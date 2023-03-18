mod avatar;
mod bone;
mod config;
mod device;
mod ik;

use std::error::Error;
use std::io::Cursor;
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{bail, ensure, Context as _, Result as AnyResult};
use async_broadcast::{Receiver as BroadcastRx, RecvError as BroadcastRxErr};
use futures::prelude::*;
use glam::Quat;
use hashbrown::HashMap;
use smol::net::UdpSocket;
use string_cache::DefaultAtom;

use avatar::AvatarState;
use bone::{Bone, BoneMask};
pub use config::{AxisOutputConfig, ButtonOutputConfig, Config};

use crate::config::MappingConfig;
use crate::output::OutputEvent;

pub async fn run(
    config: Config,
    mappings: Arc<MappingConfig>,
    mut recv: BroadcastRx<OutputEvent>,
) -> AnyResult<()> {
    log::info!("VMC task starting...");

    let socket = UdpSocket::bind(config.input.address)
        .await
        .with_context(|| format!("Failed to bind to UDP socket {}", config.input.address))?;
    let mut recv_buffer = vec![0u8; 16384];

    let mut avatar = AvatarState::new();
    let mut devices = config.device;
    let mut packets = PacketBuffer::new();
    let mut tracking = TrackingData::new();

    let mut num_received = 0u32;
    let mut processing_time_max = Duration::ZERO;
    let mut processing_time_min = Duration::MAX;
    let mut processing_time_total = Duration::ZERO;

    let mut report_timer = config
        .report_interval
        .map(Duration::from_secs_f64)
        .map(smol::Timer::interval)
        .unwrap_or_else(smol::Timer::never);

    log::info!(
        "VMC task started. Listening on {}, sending to {}.",
        config.input.address,
        config.output.address
    );

    loop {
        futures::select_biased! {
            recv = socket.recv_from(recv_buffer.as_mut_slice()).fuse() => {
                let recv_time = Instant::now();

                let (size, addr) = match recv {
                    Ok(r) => r,
                    Err(e) => {
                        log::debug!("Failed to receive VMC data: {e}");
                        continue;
                    },
                };

                log::debug!("Received packet of {size} byte(s) from {addr}.");

                let data = &recv_buffer[..size];
                let packet = match rosc::decoder::decode_udp(data) {
                    Ok((_, packet)) => packet,
                    Err(e) => {
                        log::warn!("Failed to decode {size}-byte packet from {addr}: {e}");
                        continue;
                    },
                };

                tracking.update(&packet);
                avatar.update(0.0, &devices);
                avatar.apply_to(&mut tracking);
                packets.apply_data(&tracking);

                let mut cursor = Cursor::new(&mut recv_buffer);
                let data_len = packets.encode(&mut cursor)
                    .context("Failed to encode VMC bundle")?;
                let data = &recv_buffer[..data_len];

                socket.send_to(data, config.output.address).await
                    .context("failed to send VMC bundle")?;

                let processing_time = recv_time.elapsed();
                num_received += 1;

                processing_time_max = processing_time_max.max(processing_time);
                processing_time_min = processing_time_min.min(processing_time);
                processing_time_total += processing_time;
            },

            _ = report_timer.next().fuse() => {
                let report_interval = config.report_interval.unwrap();
                let avg_rate = num_received as f64 / report_interval;
                log::info!("VMC updates sent due to received tracking data: {num_received} ({avg_rate:.1}/sec)");

                if num_received > 0 {
                    let avg_processing = 1e6 * processing_time_total.as_secs_f64() / num_received as f64;
                    let max_processing = 1e6 * processing_time_max.as_secs_f64();
                    let min_processing = 1e6 * processing_time_min.as_secs_f64();
                    log::info!("VMC processing time: avg {avg_processing:.1} us, range {min_processing:.1}-{max_processing:.1} us");
                }

                num_received = 0;
                processing_time_max = Duration::ZERO;
                processing_time_min = Duration::MAX;
                processing_time_total = Duration::ZERO;
            },

            msg = recv.recv().fuse() => match msg {
                Ok(OutputEvent::UpdateAxis(id, value)) => {
                    if let Some(axis) = mappings.axis.get(&id) {
                        for (name, range) in axis.output.vmc.on_update.blendshape.iter() {
                            let mapped_value = range[0] + value as f32 * (range[1] - range[0]);
                            tracking.update_blendshape(name, mapped_value / 100.0);
                        }

                        for (name, range) in axis.output.vmc.on_update.device.iter() {
                            if let Some(device) = devices.get_mut(name) {
                                let mapped_value = range[0] + value as f32 * (range[1] - range[0]);
                                device.set_value(mapped_value);
                            }
                        }
                    }
                },

                Ok(OutputEvent::UpdateButton(id, pressed)) => {
                    if let Some(button) = mappings.button.get(&id) {
                        for (name, range) in button.output.vmc.on_update.blendshape.iter() {
                            let mapped_value = if pressed { range[1] } else { range[0] };
                            tracking.update_blendshape(name, mapped_value / 100.0);
                        }

                        for (name, range) in button.output.vmc.on_update.device.iter() {
                            if let Some(device) = devices.get_mut(name) {
                                let mapped_value = if pressed { range[1] } else { range[0] };
                                device.set_value(mapped_value);
                            }
                        }

                        let on_state = if pressed { &button.output.vmc.on_press } else { &button.output.vmc.on_release };

                        for (name, value) in on_state.blendshape.iter() {
                            tracking.update_blendshape(name, *value);
                        }

                        for (name, value) in on_state.device.iter() {
                            if let Some(device) = devices.get_mut(name) {
                                device.set_value(*value);
                            }
                        }
                    }
                },

                Ok(_) => {},

                Err(BroadcastRxErr::Overflowed(n)) => {
                    log::warn!("VMC sender missed {n} update(s)!");
                },

                Err(BroadcastRxErr::Closed) => {
                    log::info!("VMC task stopping (no inputs remaining)...");
                    break;
                },
            },
        };
    }

    log::info!("VMC task stopped.");
    Ok(())
}

#[derive(Debug)]
struct TrackingData {
    root: TrackingPoint,
    blendshapes: HashMap<DefaultAtom, (f32, u32)>,
    devices: HashMap<(Device, DefaultAtom), (TrackingPoint, usize)>,

    local_bones: Vec<TrackingPoint>,

    global_bones: Vec<TrackingPoint>,
    global_ready: BoneMask,

    time: f32,
    tracking: bool,
}

#[derive(Clone, Copy, Debug, Default)]
struct TrackingPoint {
    pos: glam::Vec3A,
    rot: glam::Quat,
}

#[derive(Debug)]
struct PacketBuffer {
    pre_packets: Vec<rosc::OscPacket>,
    bone_packets: Vec<rosc::OscPacket>,
    device_packets: Vec<rosc::OscPacket>,
    blendshape_packets: Vec<rosc::OscPacket>,
    post_packets: Vec<rosc::OscPacket>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Device {
    Hmd,
    Controller,
    Tracker,
}

impl PacketBuffer {
    fn new() -> PacketBuffer {
        PacketBuffer {
            pre_packets: vec![rosc::OscPacket::Message(rosc::OscMessage {
                addr: String::from("/VMC/Ext/Root/Pos"),
                args: vec![
                    rosc::OscType::String(String::from("root")),
                    rosc::OscType::Float(0.0),
                    rosc::OscType::Float(0.0),
                    rosc::OscType::Float(0.0),
                    rosc::OscType::Float(0.0),
                    rosc::OscType::Float(0.0),
                    rosc::OscType::Float(0.0),
                    rosc::OscType::Float(0.0),
                ],
            })],

            bone_packets: Bone::iter()
                .map(|bone| {
                    rosc::OscPacket::Message(rosc::OscMessage {
                        addr: String::from("/VMC/Ext/Bone/Pos"),
                        args: vec![
                            rosc::OscType::String(bone.name().to_string()),
                            rosc::OscType::Float(0.0),
                            rosc::OscType::Float(0.0),
                            rosc::OscType::Float(0.0),
                            rosc::OscType::Float(0.0),
                            rosc::OscType::Float(0.0),
                            rosc::OscType::Float(0.0),
                            rosc::OscType::Float(0.0),
                        ],
                    })
                })
                .collect(),

            device_packets: vec![],

            blendshape_packets: vec![rosc::OscPacket::Message(rosc::OscMessage {
                addr: String::from("/VMC/Ext/Blend/Apply"),
                args: vec![],
            })],

            post_packets: vec![
                rosc::OscPacket::Message(rosc::OscMessage {
                    addr: String::from("/VMC/Ext/OK"),
                    args: vec![rosc::OscType::Int(0)],
                }),
                rosc::OscPacket::Message(rosc::OscMessage {
                    addr: String::from("/VMC/Ext/T"),
                    args: vec![rosc::OscType::Float(0.0)],
                }),
            ],
        }
    }

    fn apply_data(&mut self, tracking: &TrackingData) {
        update_point(&tracking.root, &mut self.pre_packets[0]);

        for (tracking, packet) in std::iter::zip(&tracking.local_bones, &mut self.bone_packets) {
            update_point(tracking, packet);
        }

        {
            let num_needed = tracking.devices.len() - self.device_packets.len();
            self.device_packets.reserve(num_needed);

            self.device_packets.resize_with(tracking.devices.len(), || {
                rosc::OscPacket::Message(rosc::OscMessage {
                    addr: String::new(),
                    args: vec![
                        rosc::OscType::Nil,
                        rosc::OscType::Float(0.0),
                        rosc::OscType::Float(0.0),
                        rosc::OscType::Float(0.0),
                        rosc::OscType::Float(0.0),
                        rosc::OscType::Float(0.0),
                        rosc::OscType::Float(0.0),
                        rosc::OscType::Float(1.0),
                    ],
                })
            });

            for ((device, name), (tracking, index)) in &tracking.devices {
                let rosc::OscPacket::Message(message) = &mut self.device_packets[*index] else { unreachable!() };
                message.args[1] = rosc::OscType::Float(tracking.pos.x);
                message.args[2] = rosc::OscType::Float(tracking.pos.y);
                message.args[3] = rosc::OscType::Float(tracking.pos.z);

                message.args[4] = rosc::OscType::Float(tracking.rot.x);
                message.args[5] = rosc::OscType::Float(tracking.rot.y);
                message.args[6] = rosc::OscType::Float(tracking.rot.z);
                message.args[7] = rosc::OscType::Float(tracking.rot.w);

                if message.addr.is_empty() {
                    message.addr = String::from(match device {
                        Device::Controller => "/VMC/Ext/Con/Pos",
                        Device::Hmd => "/VMC/Ext/Hmd/Pos",
                        Device::Tracker => "/VMC/Ext/Tra/Pos",
                    });

                    message.args[0] = rosc::OscType::String(name.to_string());
                }
            }
        }

        {
            let apply_packet = self.blendshape_packets.pop().unwrap();

            let num_needed = 1 + tracking.blendshapes.len() - self.blendshape_packets.len();
            self.blendshape_packets.reserve(num_needed);

            self.blendshape_packets
                .resize_with(tracking.blendshapes.len(), || {
                    rosc::OscPacket::Message(rosc::OscMessage {
                        addr: String::new(),
                        args: vec![rosc::OscType::Nil, rosc::OscType::Float(0.0)],
                    })
                });

            for (name, (value, index)) in &tracking.blendshapes {
                let rosc::OscPacket::Message(message) = &mut self.blendshape_packets[*index as usize] else { unreachable!() };
                message.args[1] = rosc::OscType::Float(*value);

                if message.addr.is_empty() {
                    message.addr = String::from("/VMC/Ext/Blend/Val");
                    message.args[0] = rosc::OscType::String(name.to_string());
                }
            }

            self.blendshape_packets.push(apply_packet);
        }

        let rosc::OscPacket::Message(message) = &mut self.post_packets[0] else { unreachable!() };
        message.args[0] = rosc::OscType::Int(if tracking.tracking { 1 } else { 0 });

        let rosc::OscPacket::Message(message) = &mut self.post_packets[1] else { unreachable!() };
        message.args[0] = rosc::OscType::Float(tracking.time);
    }

    fn encode<O: rosc::encoder::Output>(&mut self, out: &mut O) -> AnyResult<usize>
    where
        O::Err: 'static + Error + Send + Sync,
    {
        let mut buffer = std::mem::take(&mut self.pre_packets);
        buffer.reserve(
            self.bone_packets.len()
                + self.device_packets.len()
                + self.blendshape_packets.len()
                + self.post_packets.len(),
        );

        let bone_start = buffer.len();
        buffer.append(&mut self.bone_packets);

        let device_start = buffer.len();
        buffer.append(&mut self.device_packets);

        let blendshape_start = buffer.len();
        buffer.append(&mut self.blendshape_packets);

        let post_start = buffer.len();
        buffer.append(&mut self.post_packets);

        let packet = rosc::OscPacket::Bundle(rosc::OscBundle {
            timetag: (0, 0).into(),
            content: buffer,
        });

        let data_len =
            rosc::encoder::encode_into(&packet, out).context("Failed to encode VMC packet")?;

        let rosc::OscPacket::Bundle(bundle) = packet else { unreachable!() };
        let mut buffer = bundle.content;

        self.post_packets.extend(buffer.drain(post_start..));
        self.blendshape_packets
            .extend(buffer.drain(blendshape_start..));
        self.device_packets.extend(buffer.drain(device_start..));
        self.bone_packets.extend(buffer.drain(bone_start..));
        self.pre_packets = buffer;

        if data_len > rosc::decoder::MTU {
            static FLAGGED: AtomicBool = AtomicBool::new(false);
            if !FLAGGED.swap(true, std::sync::atomic::Ordering::Relaxed) {
                log::warn!("Encoded data ({} byte(s)) is larger than typical MTU ({} byte(s)). Data may be lost if sent via the Internet! This will only be logged once.", data_len, rosc::decoder::MTU);
            }
        }

        Ok(data_len)
    }
}

impl TrackingData {
    fn new() -> TrackingData {
        TrackingData {
            root: TrackingPoint::default(),
            blendshapes: HashMap::new(),
            devices: HashMap::new(),

            local_bones: Bone::iter().map(|_| TrackingPoint::default()).collect(),

            global_bones: Bone::iter().map(|_| TrackingPoint::default()).collect(),
            global_ready: BoneMask::all(),

            time: -1.0,
            tracking: false,
        }
    }

    fn global_bone(&mut self, bone: Bone) -> TrackingPoint {
        if self.global_ready.insert(bone) {
            let local_point = self.local_bone(bone);
            let parent_point = bone.parent().map_or(self.root, |b| self.global_bone(b));

            self.global_bones[bone as u8 as usize] = TrackingPoint {
                pos: parent_point.pos + parent_point.rot * local_point.pos,
                rot: (parent_point.rot * local_point.rot).normalize(),
            };
        }

        self.global_bones[bone as u8 as usize]
    }

    fn local_bone(&self, bone: Bone) -> TrackingPoint {
        self.local_bones[bone as u8 as usize]
    }

    fn set_global_bone_rot(&mut self, bone: Bone, new_rot: Quat) {
        self.global_ready = self.global_ready.difference(&bone.descendants());

        let parent_point = bone.parent().map_or(self.root, |b| self.global_bone(b));

        self.global_bones[bone as u8 as usize].rot = new_rot;
        self.local_bones[bone as u8 as usize].rot =
            (parent_point.rot.inverse() * new_rot).normalize();
    }

    fn set_local_bone(&mut self, bone: Bone, point: TrackingPoint) {
        self.global_ready = self.global_ready.difference(&bone.affected());
        self.local_bones[bone as u8 as usize] = point;
    }

    fn set_local_bone_rot(&mut self, bone: Bone, new_rot: Quat) {
        self.global_ready = self.global_ready.difference(&bone.affected());
        self.local_bones[bone as u8 as usize].rot = new_rot;
    }

    fn set_root(&mut self, point: TrackingPoint) {
        self.global_ready.clear();
        self.root = point;
    }

    fn update(&mut self, packet: &rosc::OscPacket) {
        let result: AnyResult<()> = (|| {
            match *packet {
                rosc::OscPacket::Bundle(ref bundle) => {
                    for child in &bundle.content {
                        self.update(child);
                    }
                }

                rosc::OscPacket::Message(ref message) => {
                    // TODO: Use address matchers instead
                    match message.addr.as_str() {
                        "/VMC/Ext/Root/Pos" => {
                            let (name, point) = message.arg_tracking()?;
                            ensure!(
                                name == "root",
                                "Unexpected name of root (expected \"root\", got \"{}\").",
                                name
                            );
                            self.set_root(point);
                        }

                        "/VMC/Ext/Bone/Pos" => {
                            let (name, point) = message.arg_tracking()?;
                            let bone = Bone::from_str(name).context("Failed to parse bone")?;
                            self.set_local_bone(bone, point);
                        }

                        "/VMC/Ext/Con/Pos" => {
                            let (name, point) = message.arg_tracking()?;
                            self.update_device(Device::Controller, name, &point);
                        }

                        "/VMC/Ext/Hmd/Pos" => {
                            let (name, point) = message.arg_tracking()?;
                            self.update_device(Device::Hmd, name, &point);
                        }

                        "/VMC/Ext/Tra/Pos" => {
                            let (name, point) = message.arg_tracking()?;
                            self.update_device(Device::Tracker, name, &point);
                        }

                        "/VMC/Ext/Blend/Val" => {
                            ensure!(
                                message.args.len() == 2,
                                "Incorrect number of arguments to {} (expected 2, got {}).",
                                message.addr,
                                message.args.len()
                            );
                            let name = message.arg_str(0)?;
                            let value = message.arg_f32(1)?;
                            self.update_blendshape(name, value);
                        }

                        "/VMC/Ext/Blend/Apply" => {}

                        "/VMC/Ext/OK" => {
                            ensure!(
                                message.args.len() == 1,
                                "Incorrect number of arguments to {} (expected 1, got {}).",
                                message.addr,
                                message.args.len()
                            );
                            self.tracking = 1 == message.arg_i32(0)?;
                        }

                        "/VMC/Ext/T" => {
                            ensure!(
                                message.args.len() == 1,
                                "Incorrect number of arguments to {} (expected 1, got {}).",
                                message.addr,
                                message.args.len()
                            );
                            self.time = message.arg_f32(0)?;
                        }

                        _ => bail!("Unrecognized VMC address: {}", message.addr),
                    }
                }
            }

            Ok(())
        })();

        if let Err(e) = result {
            log::debug!("Failed to process received VMC message: {}", e);
        }
    }

    fn update_blendshape(&mut self, name: impl Into<DefaultAtom>, value: f32) {
        let num_blendshapes = self
            .blendshapes
            .len()
            .try_into()
            .expect("Too many blendshapes!");

        let atom = name.into();
        self.blendshapes
            .entry(atom)
            .and_modify(|(v, _)| *v = value)
            .or_insert((value, num_blendshapes));
    }

    fn update_device(
        &mut self,
        device: Device,
        name: impl Into<DefaultAtom>,
        point: &TrackingPoint,
    ) {
        let atom = name.into();
        let num_devices = self.devices.len();

        self.devices
            .entry((device, atom))
            .and_modify(|(v, _)| *v = *point)
            .or_insert((*point, num_devices));
    }
}

fn update_point(tracking: &TrackingPoint, packet: &mut rosc::OscPacket) {
    let rosc::OscPacket::Message(message) = packet else { unreachable!() };
    assert_eq!(message.args.len(), 8);

    message.args[1] = rosc::OscType::Float(tracking.pos.x);
    message.args[2] = rosc::OscType::Float(tracking.pos.y);
    message.args[3] = rosc::OscType::Float(tracking.pos.z);

    message.args[4] = rosc::OscType::Float(tracking.rot.x);
    message.args[5] = rosc::OscType::Float(tracking.rot.y);
    message.args[6] = rosc::OscType::Float(tracking.rot.z);
    message.args[7] = rosc::OscType::Float(tracking.rot.w);
}

trait OscMessageExt {
    fn arg_f32(&self, i: usize) -> AnyResult<f32>;
    fn arg_i32(&self, i: usize) -> AnyResult<i32>;
    fn arg_str(&self, i: usize) -> AnyResult<&str>;

    fn arg_tracking(&self) -> AnyResult<(&str, TrackingPoint)>;
}

impl OscMessageExt for rosc::OscMessage {
    fn arg_f32(&self, i: usize) -> AnyResult<f32> {
        match self.args[i] {
            rosc::OscType::Float(x) => Ok(x),
            rosc::OscType::Double(x) => Ok(x as f32),
            _ => bail!(
                "Incorrect type for argument {} to {} (expected float, got {:?}).",
                i,
                self.addr,
                self.args[i]
            ),
        }
    }

    fn arg_i32(&self, i: usize) -> AnyResult<i32> {
        match self.args[i] {
            rosc::OscType::Int(x) => Ok(x),
            rosc::OscType::Long(x) => x.try_into().ok().with_context(|| {
                format!(
                    "Invalid value for argument {} to {} (integer out of range).",
                    i + 1,
                    self.addr
                )
            }),
            _ => bail!(
                "Incorrect type for argument {} to {} (expected int, got {:?}).",
                i + 1,
                self.addr,
                self.args[i]
            ),
        }
    }

    fn arg_str(&self, i: usize) -> AnyResult<&str> {
        match self.args[i] {
            rosc::OscType::String(ref s) => Ok(s),
            _ => bail!(
                "Incorrect type for argument {} to {} (expected string, got {:?}).",
                i + 1,
                self.addr,
                self.args[i]
            ),
        }
    }

    fn arg_tracking(&self) -> AnyResult<(&str, TrackingPoint)> {
        ensure!(
            self.args.len() == 8,
            "Incorrect number of arguments to {} (expected 8, got {}).",
            self.addr,
            self.args.len()
        );
        let name = self.arg_str(0)?;

        let px = self.arg_f32(1)?;
        let py = self.arg_f32(2)?;
        let pz = self.arg_f32(3)?;

        let rx = self.arg_f32(4)?;
        let ry = self.arg_f32(5)?;
        let rz = self.arg_f32(6)?;
        let rw = self.arg_f32(7)?;

        Ok((
            name,
            TrackingPoint {
                pos: glam::Vec3A::new(px, py, pz),
                rot: glam::Quat::from_xyzw(rx, ry, rz, rw).normalize(),
            },
        ))
    }
}

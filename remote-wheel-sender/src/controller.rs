use std::fmt::Display;
use std::num::{NonZeroU32, NonZeroU8};
use std::sync::Arc;

use anyhow::{bail, Context as _, Result as AnyResult};
use async_broadcast::Sender as BroadcastTx;
use hashbrown::HashMap;
use sdl2::event::Event as SdlEvent;
use serde::Deserialize;
use smol::channel::Receiver as ChannelRx;
use string_cache::DefaultAtom;

use crate::config::MappingConfig;
use crate::output::OutputEvent;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct AxisInputConfig {
    name: DefaultAtom,
    axis: Axis,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct ButtonInputConfig {
    name: DefaultAtom,
    button: NonZeroU32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Axis(NonZeroU8);

impl<'de> Deserialize<'de> for Axis {
    fn deserialize<D: serde::de::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        de.deserialize_any(AxisVisitor)
    }
}

impl Display for Axis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Axis #{}", self.0)
    }
}

impl TryFrom<u8> for Axis {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, ()> {
        value
            .checked_add(1)
            .and_then(NonZeroU8::new)
            .map(Axis)
            .ok_or(())
    }
}

struct AxisVisitor;

impl AxisVisitor {
    fn visit_int<I: TryInto<u8>>(self, v: I) -> Option<Axis> {
        v.try_into().ok().and_then(NonZeroU8::new).map(Axis)
    }
}

impl<'de> serde::de::Visitor<'de> for AxisVisitor {
    type Value = Axis;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "axis number (1-255)")
    }

    fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
        self.visit_int(v).ok_or_else(|| {
            E::invalid_value(serde::de::Unexpected::Signed(v), &"axis number (1-255)")
        })
    }

    fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
        self.visit_int(v).ok_or_else(|| {
            E::invalid_value(serde::de::Unexpected::Unsigned(v), &"axis number (1-255)")
        })
    }

    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
        // For backwards-compatibility
        // These names don't necessarily correspond to the correct axes!
        match v {
            "x" => Ok(Axis(NonZeroU8::new(1).unwrap())),
            "y" => Ok(Axis(NonZeroU8::new(2).unwrap())),
            "z" => Ok(Axis(NonZeroU8::new(3).unwrap())),
            "rx" => Ok(Axis(NonZeroU8::new(4).unwrap())),
            "ry" => Ok(Axis(NonZeroU8::new(5).unwrap())),
            "rz" => Ok(Axis(NonZeroU8::new(6).unwrap())),
            "slider" => Ok(Axis(NonZeroU8::new(7).unwrap())),
            "dial" => Ok(Axis(NonZeroU8::new(8).unwrap())),
            _ => Err(E::unknown_variant(
                v,
                &["x", "y", "z", "rx", "ry", "rz", "slider", "dial"],
            )),
        }
    }
}

#[derive(Debug, Default)]
pub struct ControllerMap {
    pub axes: HashMap<Axis, Vec<DefaultAtom>>,
    pub buttons: HashMap<u32, Vec<DefaultAtom>>,
}

pub async fn run(
    exec: Arc<smol::Executor<'static>>,
    mappings: Arc<MappingConfig>,
    output_tx: BroadcastTx<OutputEvent>,
    cancel_rx: ChannelRx<()>,
) -> AnyResult<()> {
    log::info!("Controller input task starting...");

    let mut controllers = HashMap::new();

    for (id, axes) in &mappings.axis {
        for axis in &axes.input {
            if let crate::config::AxisInputConfig::Controller(c) = axis {
                let controller = controllers
                    .entry(c.name.clone())
                    .or_insert_with(|| Arc::new(ControllerMap::default()));

                Arc::get_mut(controller)
                    .unwrap()
                    .axes
                    .entry(c.axis)
                    .or_insert_with(Vec::new)
                    .push(id.clone());
            }
        }
    }

    for (id, buttons) in &mappings.button {
        for button in &buttons.input {
            if let crate::config::ButtonInputConfig::Controller(c) = button {
                let controller = controllers
                    .entry(c.name.clone())
                    .or_insert_with(|| Arc::new(ControllerMap::default()));

                Arc::get_mut(controller)
                    .unwrap()
                    .buttons
                    .entry(c.button.get())
                    .or_insert_with(Vec::new)
                    .push(id.clone());
            }
        }
    }

    if controllers.is_empty() {
        log::info!("No controllers configured. Controller input task not starting.");
        return Ok(());
    }

    smol::unblock(move || run_sync(exec, controllers, output_tx, cancel_rx)).await
}

fn run_sync(
    exec: Arc<smol::Executor>,
    controllers: HashMap<DefaultAtom, Arc<ControllerMap>>,
    output_tx: BroadcastTx<OutputEvent>,
    cancel_rx: ChannelRx<()>,
) -> AnyResult<()> {
    log::info!("Controller input task started.");

    // Devices that support both DirectInput and XInput are broken when any of the following are enabled
    sdl2::hint::set("SDL_JOYSTICK_HIDAPI", "0");
    //sdl2::hint::set("SDL_JOYSTICK_HIDAPI_XBOX", "0");
    sdl2::hint::set("SDL_JOYSTICK_RAWINPUT", "0");
    sdl2::hint::set("SDL_XINPUT_ENABLED", "0");

    let sdl = sdl2::init()
        .or_else(|e| bail!(e))
        .context("Failed to initialize SDL")?;

    let sdl_event = sdl
        .event()
        .or_else(|e| bail!(e))
        .context("Failed to initialize SDL event subsystem")?;

    let sdl_joystick = sdl
        .joystick()
        .or_else(|e| bail!(e))
        .context("Failed to initialize SDL joystick subsystem")?;

    let sender = sdl_event.event_sender();
    exec.spawn(async move {
        let _ = cancel_rx.recv().await;
        sender
            .push_event(SdlEvent::Quit { timestamp: 0 })
            .expect("Failed to push Quit event to Controller input task");
    })
    .detach();

    let mut sdl_event_pump = sdl
        .event_pump()
        .or_else(|e| bail!(e))
        .context("Failed to initialize SDL event pump subsystem")?;

    let mut connected_map = HashMap::new();

    'outer: loop {
        let mut event = Some(sdl_event_pump.wait_event());
        let mut flush = false;

        while let Some(e) = event {
            match e {
                SdlEvent::JoyDeviceAdded { which, .. } => {
                    let joystick = sdl_joystick
                        .open(which)
                        .or_else(|e| bail!(e))
                        .context("Failed to open joystick")?;

                    let id = joystick.instance_id();
                    let name = DefaultAtom::from(joystick.name());
                    let controller = controllers.get(&name).cloned();

                    log::info!(
                        "Joystick detected: {} (configured axes: {}, buttons: {})",
                        name.escape_default(),
                        controller.as_ref().map(|m| m.axes.len()).unwrap_or(0),
                        controller.as_ref().map(|m| m.buttons.len()).unwrap_or(0)
                    );
                    connected_map.insert(id, (joystick, name, controller));
                }

                SdlEvent::JoyDeviceRemoved { which, .. } => {
                    if let Some((_, name, _)) = connected_map.remove(&which) {
                        log::info!("Joystick removed: {}", name.escape_default());
                    }
                }

                SdlEvent::JoyAxisMotion {
                    which,
                    axis_idx,
                    value,
                    ..
                } => {
                    if let Some((_, _, Some(controller))) = connected_map.get(&which) {
                        if let Ok(axis) = Axis::try_from(axis_idx) {
                            let value = value.wrapping_add_unsigned(32768) as u16 as f64 / 65535.0;

                            if let Some(inputs) = controller.axes.get(&axis) {
                                for input in inputs {
                                    if smol::block_on(
                                        output_tx.broadcast(OutputEvent::UpdateAxis(
                                            input.clone(),
                                            value,
                                        )),
                                    )
                                    .is_err()
                                    {
                                        log::info!("Controller input task stopping (no remaining outputs).");
                                        break 'outer;
                                    }

                                    flush = true;
                                }
                            }
                        }
                    }
                }

                SdlEvent::JoyButtonDown {
                    which, button_idx, ..
                }
                | SdlEvent::JoyButtonUp {
                    which, button_idx, ..
                } => {
                    if let Some((_, _, Some(controller))) = connected_map.get(&which) {
                        let button = button_idx as u32 + 1;

                        if let Some(inputs) = controller.buttons.get(&button) {
                            let pressed = matches!(e, SdlEvent::JoyButtonDown { .. });

                            for input in inputs {
                                if smol::block_on(
                                    output_tx.broadcast(OutputEvent::UpdateButton(
                                        input.clone(),
                                        pressed,
                                    )),
                                )
                                .is_err()
                                {
                                    log::info!(
                                        "Controller input task stopping (no remaining outputs)."
                                    );
                                    break 'outer;
                                }

                                flush = true;
                            }
                        }
                    }
                }

                SdlEvent::Quit { .. } => {
                    log::info!("Controller input task stopping (shutdown)...");
                    break 'outer;
                }

                _ => {}
            }

            event = sdl_event_pump.poll_event();
        }

        if flush && smol::block_on(output_tx.broadcast(OutputEvent::Flush)).is_err() {
            log::info!("Controller input task stopping (no remaining outputs).");
            break;
        }
    }

    log::info!("Controller input task stopped.");
    Ok(())
}

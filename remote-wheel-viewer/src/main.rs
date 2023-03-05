#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::{Cursor};
use std::path::{Path};
use std::process::{ExitCode};
use std::thread::{JoinHandle};

use anyhow::{Context as _, Result as AnyResult, anyhow};
use eframe::{CreationContext, NativeOptions};
use eframe::egui::{TextureOptions, TextureFilter, Frame};
use eframe::epaint::{TextureHandle, ImageData, ColorImage};
use futures::prelude::*;
use image::{RgbaImage, ImageFormat};
use rosc::{OscPacket, OscType, OscMessage};
use smol::channel::{Receiver, TryRecvError, Sender};
use smol::net::{SocketAddr, UdpSocket};

mod config;
use config::{AppConfig};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            let message = format!("An error has occurred: {:#}", e);

            #[cfg(debug_assertions)]
            {
                eprintln!("{}", message);
            }

            #[cfg(not(debug_assertions))]
            {
                rfd::MessageDialog::new()
                    .set_title("Error")
                    .set_description(&message)
                    .set_level(rfd::MessageLevel::Error)
                    .set_buttons(rfd::MessageButtons::Ok)
                    .show();
            }

            ExitCode::FAILURE
        },
    }

}

fn run() -> AnyResult<()> {
    let config = AppConfig::read_from("remote-wheel-viewer.yaml")
        .context("Failed to load configuration")?;

    let wheel_image = if config.wheel == Path::new("default") {
        image::io::Reader::with_format(Cursor::new(include_bytes!("default-wheel.png")), ImageFormat::Png)
            .decode()
            .context("Failed to decode wheel image")?
    } else {
        image::io::Reader::open(&config.wheel)
            .context("Failed to open wheel image")?
            .decode()
            .context("Failed to decode wheel image")?
    };

    let wheel_image = wheel_image.to_rgba8();
    let (wheel_width, wheel_height) = wheel_image.dimensions();
    let wheel_square = u32::max(wheel_width, wheel_height);

    let options = NativeOptions {
        initial_window_size: Some((wheel_square as f32, wheel_square as f32).into()),
        resizable: false,
        .. NativeOptions::default()
    };

    eframe::run_native("Remote Wheel Viewer", options, Box::new(move |cc| Box::new(App::new(cc, config, wheel_image))))
        .map_err(|e| anyhow!("{}", e))
        .context("Failed to run application")?;

    Ok(())
}

struct App {
    background: eframe::egui::Color32,
    rotation: f64,
    wheel_texture: TextureHandle,

    async_thread: Option<JoinHandle<()>>,
    event_rx: Receiver<AppEvent>,
    run_tx: Sender<()>,
}

impl App {
    fn new(cc: &CreationContext, config: AppConfig, wheel_image: RgbaImage) -> Self {
        let (wheel_width, wheel_height) = wheel_image.dimensions();
        let wheel_data = ImageData::Color(ColorImage::from_rgba_unmultiplied([wheel_width as usize, wheel_height as usize], &wheel_image));

        let (event_tx, event_rx) = smol::channel::unbounded();
        let (run_tx, run_rx) = smol::channel::unbounded();
        let egui = cc.egui_ctx.clone();

        App {
            background: config.background.into(),
            rotation: 0.0,
            wheel_texture: cc.egui_ctx.load_texture("wheel", wheel_data, TextureOptions {
                magnification: TextureFilter::Linear,
                minification: TextureFilter::Linear,
            }),

            async_thread: Some(std::thread::spawn(move || async_thread(config, egui, event_tx, run_rx))),
            event_rx, run_tx,
        }
    }
}

impl eframe::App for App {
    fn clear_color(&self, _visuals: &eframe::egui::Visuals) -> [f32; 4] {
        self.background.to_normalized_gamma_f32()
    }

    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        loop {
            match self.event_rx.try_recv() {
                Ok(AppEvent::RotationUpdate(f)) => {
                    self.rotation = f;
                },

                Err(TryRecvError::Closed) => frame.close(),
                Err(TryRecvError::Empty) => break,
            }
        }

        eframe::egui::CentralPanel::default().frame(Frame::none()).show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                ui.add(eframe::egui::widgets::Image::new(self.wheel_texture.id(), self.wheel_texture.size_vec2())
                    .rotate(self.rotation.to_radians() as f32, [0.5, 0.5].into()));
            });
        });
    }

    fn on_exit(&mut self, _: Option<&eframe::glow::Context>) {
        self.event_rx.close();
        self.run_tx.close();

        if let Some(async_thread) = self.async_thread.take() {
            async_thread.join()
                .expect("Failed to join on async thread");
        }
    }
}

enum AppEvent {
    RotationUpdate(f64),
}

fn async_thread(config: AppConfig, egui: eframe::egui::Context, event_tx: Sender<AppEvent>, run_rx: Receiver<()>) {
    let listen_fut = listen_osc(config.osc.address, egui, event_tx);

    smol::block_on(async move {
        futures::select_biased!{
            _ = run_rx.recv().fuse() => {},
            r = listen_fut.fuse() => r.expect("Failed to listen for OSC messages"),
        }
    });
}

async fn listen_osc(addr: SocketAddr, egui: eframe::egui::Context, sender: Sender<AppEvent>) -> AnyResult<()> {
    let socket = UdpSocket::bind(addr).await
        .with_context(|| format!("Failed to bind to UDP address {}", addr))?;

    let rotation_addr = rosc::address::OscAddress::new(String::from("/wheel/rotation"))
        .context("Failed to create OSC address for wheel rotation")?;

    let mut buf = [0; 4096];
    let mut messages = Vec::new();

    while let Ok(size) = socket.recv(&mut buf).await {
        let msg = &buf[..size];

        match rosc::decoder::decode_udp(msg) {
            Ok((_, packet)) => {
                fn collect_messages(packet: OscPacket, messages: &mut Vec<OscMessage>) {
                    match packet {
                        OscPacket::Bundle(bundle) => {
                            for packet in bundle.content {
                                collect_messages(packet, messages);
                            }
                        },

                        OscPacket::Message(message) => messages.push(message),
                    }
                }

                collect_messages(packet, &mut messages);

                for message in messages.drain(..) {
                    let message_matcher = match rosc::address::Matcher::new(&message.addr) {
                        Ok(m) => m,
                        Err(e) => {
                            eprintln!("Failed to parse received OSC address ({}): {}", message.addr, e);
                            continue;
                        },
                    };

                    if message_matcher.match_address(&rotation_addr) {
                        for arg in &message.args {
                            match *arg {
                                OscType::Float(f) => {
                                    let _ = sender.send(AppEvent::RotationUpdate(f as f64)).await;
                                    egui.request_repaint();
                                },

                                OscType::Double(f) => {
                                    let _ = sender.send(AppEvent::RotationUpdate(f)).await;
                                    egui.request_repaint();
                                },

                                _ => eprintln!("Ignoring unrecognized value {:?} sent to {}.", arg, message.addr),
                            }
                        }
                    }
                }
            },

            Err(e) => {
                eprintln!("Failed to decode data packet: {}", e);
                continue;
            },
        }
    }

    Ok(())
}

use std::collections::HashMap;
use std::sync::{Arc};

use anyhow::{Context as _, Result as AnyResult, bail};
use async_broadcast::{Sender as BroadcastTx};
use sdl2::event::{Event as SdlEvent};
use smol::channel::{Receiver as ChannelRx};

use crate::config::{Axis, InputConfig};
use crate::output::{OutputEvent};

pub async fn run(exec: Arc<smol::Executor<'static>>, config: Vec<Arc<InputConfig>>, output_tx: BroadcastTx<OutputEvent>, cancel_rx: ChannelRx<()>) -> AnyResult<()> {
	smol::unblock(move || run_sync(exec, config, output_tx, cancel_rx)).await
}

fn run_sync(exec: Arc<smol::Executor>, config: Vec<Arc<InputConfig>>, output_tx: BroadcastTx<OutputEvent>, cancel_rx: ChannelRx<()>) -> AnyResult<()> {
	log::info!("Input task starting...");
	let mut controller_map = HashMap::with_capacity(config.len());

	for input in &config {
		let axis_map = controller_map.entry(input.controller.clone())
			.or_insert_with(|| Arc::new(HashMap::new()));

		let axis_inputs = Arc::get_mut(axis_map).unwrap()
			.entry(input.axis)
			.or_insert(Vec::new());

		axis_inputs.push(input.clone());
	}

	log::info!("Input task started.");

    // Devices that support both DirectInput and XInput are broken when any of the following are enabled
    sdl2::hint::set("SDL_JOYSTICK_HIDAPI", "0");
    //sdl2::hint::set("SDL_JOYSTICK_HIDAPI_XBOX", "0");
    sdl2::hint::set("SDL_JOYSTICK_RAWINPUT", "0");
    sdl2::hint::set("SDL_XINPUT_ENABLED", "0");

	let sdl = sdl2::init()
		.or_else(|e| bail!(e))
		.context("Failed to initialize SDL")?;

	let sdl_event = sdl.event()
		.or_else(|e| bail!(e))
		.context("Failed to initialize SDL event subsystem")?;

	let sdl_joystick = sdl.joystick()
		.or_else(|e| bail!(e))
		.context("Failed to initialize SDL joystick subsystem")?;

	let sender = sdl_event.event_sender();
	exec.spawn(async move {
		let _ = cancel_rx.recv().await;
		sender.push_event(SdlEvent::Quit { timestamp: 0 })
			.expect("Failed to push Quit event to Input task");
	}).detach();

	let mut sdl_event_pump = sdl.event_pump()
		.or_else(|e| bail!(e))
		.context("Failed to initialize SDL event pump subsystem")?;

	let mut connected_map = HashMap::new();

	for event in sdl_event_pump.wait_iter() {
		match event {
			SdlEvent::JoyDeviceAdded { which, .. } => {
				let joystick = sdl_joystick.open(which)
					.or_else(|e| bail!(e))
					.context("Failed to open joystick")?;

				let id = joystick.instance_id();
				let name = joystick.name();
				let axis_map = controller_map.get(&name).cloned();

				log::info!("Joystick detected: {} (configured inputs: {})", name, axis_map.as_ref().map(|m| m.len()).unwrap_or(0));
				connected_map.insert(id, (joystick, name, axis_map));
			},

			SdlEvent::JoyDeviceRemoved { which, .. } => {
				if let Some((_, name, _)) = connected_map.remove(&which) {
					log::info!("Joystick removed: {}", name);
				}
			},

			SdlEvent::JoyAxisMotion { which, axis_idx, value, .. } => {
				if let Some((_, name, Some(axis_map))) = connected_map.get(&which) {
                    if let Ok(axis) = Axis::try_from(axis_idx) {
                        let value = value.wrapping_add_unsigned(32768) as u16 as f64 / 65535.0;

                        if let Some(inputs) = axis_map.get(&axis) {
                            for input in inputs {
                                let value = input.range[0] + value * (input.range[1] - input.range[0]);
                                if let Err(e) = smol::block_on(output_tx.broadcast(OutputEvent::Update(input.clone(), value))) {
                                    log::warn!("Failed to send update for controller {} axis {}: {}", name, axis, e);
                                }
                            }
                        }
					}
				}
			},

			SdlEvent::Quit { .. } => {
				log::info!("Input task stopping (shutdown)...");
				break;
			},

			_ => {},
		}
	}

	log::info!("Input task stopped.");
	Ok(())
}

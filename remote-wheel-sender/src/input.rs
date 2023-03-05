use std::collections::HashMap;
use std::sync::{Arc};

use anyhow::{Context as _, Result as AnyResult, bail};
use async_broadcast::{Sender as BroadcastTx};
use sdl2::event::{Event as SdlEvent};
use smol::channel::{Receiver as ChannelRx};

use crate::config::{Axis, InputConfig, InputAxisConfig, InputButtonConfig};
use crate::output::{OutputEvent};

pub async fn run(exec: Arc<smol::Executor<'static>>, config: Vec<InputConfig>, output_tx: BroadcastTx<OutputEvent>, cancel_rx: ChannelRx<()>) -> AnyResult<()> {
	smol::unblock(move || run_sync(exec, config, output_tx, cancel_rx)).await
}

fn run_sync(exec: Arc<smol::Executor>, config: Vec<InputConfig>, output_tx: BroadcastTx<OutputEvent>, cancel_rx: ChannelRx<()>) -> AnyResult<()> {
	log::info!("Input task starting...");
	let mut controller_map = HashMap::with_capacity(config.len());

	for input in &config {
		let controller_map = controller_map.entry(input.controller().to_string())
			.or_insert_with(|| Arc::new(ControllerMap::default()));

		match *input {
			InputConfig::Axis(ref axis) => {
				let axis_inputs = Arc::get_mut(controller_map).unwrap()
					.axis_map
					.entry(axis.axis)
					.or_insert(Vec::new());

				axis_inputs.push(axis.clone());
			},

			InputConfig::Button(ref button) => {
				let button_inputs = Arc::get_mut(controller_map).unwrap()
					.button_map
					.entry(button.button)
					.or_insert(Vec::new());

				button_inputs.push(button.clone());
			},
		}
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

    'outer: loop {
        let mut event = Some(sdl_event_pump.wait_event());
        let mut flush = false;

        while let Some(e) = event {
            match e {
                SdlEvent::JoyDeviceAdded { which, .. } => {
                    let joystick = sdl_joystick.open(which)
                        .or_else(|e| bail!(e))
                        .context("Failed to open joystick")?;

                    let id = joystick.instance_id();
                    let name = joystick.name();
                    let controller_map = controller_map.get(&name).cloned();

                    log::info!("Joystick detected: {name} (configured axes: {}, buttons: {})",
                        controller_map.as_ref().map(|m| m.axis_map.len()).unwrap_or(0),
                        controller_map.as_ref().map(|m| m.button_map.len()).unwrap_or(0));
                    connected_map.insert(id, (joystick, name, controller_map.clone()));
                },

                SdlEvent::JoyDeviceRemoved { which, .. } => {
                    if let Some((_, name, _)) = connected_map.remove(&which) {
                        log::info!("Joystick removed: {}", name);
                    }
                },

                SdlEvent::JoyAxisMotion { which, axis_idx, value, .. } => {
                    if let Some((_, name, Some(controller_map))) = connected_map.get(&which) {
                        if let Ok(axis) = Axis::try_from(axis_idx) {
                            let value = value.wrapping_add_unsigned(32768) as u16 as f64 / 65535.0;

                            if let Some(inputs) = controller_map.axis_map.get(&axis) {
                                for input in inputs {
                                    let value = input.range[0] + value * (input.range[1] - input.range[0]);
                                    if let Err(e) = smol::block_on(output_tx.broadcast(OutputEvent::UpdateAxis(input.clone(), value))) {
                                        log::warn!("Failed to send update for controller {} axis {}: {}", name, axis, e);
                                    }

									flush = true;
                                }
                            }
                        }
                    }
                },

                SdlEvent::JoyButtonDown { which, button_idx, .. } | SdlEvent::JoyButtonUp { which, button_idx, .. } => {
                    if let Some((_, name, Some(controller_map))) = connected_map.get(&which) {
                        let button = button_idx as u32 + 1;

                        if let Some(inputs) = controller_map.button_map.get(&button) {
                            let pressed = matches!(e, SdlEvent::JoyButtonDown { .. });

                            for input in inputs {
                                if let Err(e) = smol::block_on(output_tx.broadcast(OutputEvent::UpdateButton(input.clone(), pressed))) {
                                    log::warn!("Failed to send update for controller {name} button {button}: {e}");
                                }

                                flush = true;
                            }
                        }
                    }
                },

                SdlEvent::Quit { .. } => {
                    log::info!("Input task stopping (shutdown)...");
                    break 'outer;
                },

                _ => {},
            }

			event = sdl_event_pump.poll_event();
		}

		if flush {
            if let Err(e) = smol::block_on(output_tx.broadcast(OutputEvent::Flush)) {
                log::warn!("Failed to send flush after controller updates: {e}");
            }
		}
	}

	log::info!("Input task stopped.");
	Ok(())
}

#[derive(Debug, Default)]
struct ControllerMap {
	axis_map: HashMap<Axis, Vec<Arc<InputAxisConfig>>>,
	button_map: HashMap<u32, Vec<Arc<InputButtonConfig>>>,
}

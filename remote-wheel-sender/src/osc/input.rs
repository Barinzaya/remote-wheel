use std::sync::Arc;

use anyhow::{Context as _, Result as AnyResult};
use async_broadcast::Sender as BroadcastTx;
use smol::channel::Receiver as ChannelRx;
use string_cache::DefaultAtom;

use super::config::InputConfig;
use crate::config::{AxisInputConfig, ButtonInputConfig, MappingConfig};
use crate::output::OutputEvent;

pub(super) async fn run(
    _config: InputConfig,
    mappings: Arc<MappingConfig>,
    inbound_rx: ChannelRx<rosc::OscPacket>,
    value_tx: BroadcastTx<OutputEvent>,
) -> AnyResult<()> {
    log::info!("OSC input processing task starting...");

    let mut axis_mappings = Vec::new();
    let mut button_mappings = Vec::new();

    for (id, axes) in &mappings.axis {
        for axis in &axes.input {
            if let AxisInputConfig::Osc(c) = axis {
                let address = rosc::address::OscAddress::new(c.address.to_string())
                    .context("Invalid OSC address in configuration")?;
                let range = c.range;

                axis_mappings.push((id.clone(), address, range));
            }
        }
    }

    for (id, buttons) in &mappings.button {
        for button in &buttons.input {
            if let ButtonInputConfig::Osc(c) = button {
                let address = rosc::address::OscAddress::new(c.address.to_string())
                    .context("Invalid OSC address in configuration")?;
                button_mappings.push((id.clone(), address));
            }
        }
    }

    if axis_mappings.is_empty() && button_mappings.is_empty() {
        log::info!("OSC input processing task stopped (no OSC inputs configured).");
        return Ok(());
    }

    log::info!("OSC input processing task started.");
    let mut events = Vec::new();

    loop {
        let Ok(packet) = inbound_rx.recv().await else {
			log::info!("OSC input processing task stopping (OSC receive task has stopped).");
			break;
		};

        collect_values(&packet, &axis_mappings, &button_mappings, &mut events);

        if !events.is_empty() {
            events.push(OutputEvent::Flush);
        }

        for event in events.drain(..) {
            if value_tx.broadcast(event).await.is_err() {
                log::info!("OSC input processing stopping (no remaining outputs).");
                break;
            }
        }
    }

    log::info!("OSC input processing task stopped.");
    Ok(())
}

fn collect_values(
    packet: &rosc::OscPacket,
    axis_mappings: &[(DefaultAtom, rosc::address::OscAddress, [f64; 2])],
    button_mappings: &[(DefaultAtom, rosc::address::OscAddress)],
    into: &mut Vec<OutputEvent>,
) {
    match packet {
        rosc::OscPacket::Bundle(b) => {
            for subpacket in &b.content {
                collect_values(subpacket, axis_mappings, button_mappings, into);
            }
        }

        rosc::OscPacket::Message(m) => {
            let axis_value = m.args.get(0).and_then(|a| match a {
                rosc::OscType::Double(f) => Some(*f),
                rosc::OscType::Float(f) => Some(*f as f64),
                rosc::OscType::Int(i) => Some(*i as f64),
                rosc::OscType::Long(i) => Some(*i as f64),
                _ => None,
            });

            let button_value = m.args.get(0).and_then(|a| match a {
                rosc::OscType::Bool(b) => Some(*b),
                _ => None,
            });

            if axis_value.is_some() || button_value.is_some() {
                let Ok(matcher) = rosc::address::Matcher::new(&m.addr) else { return };

                if let Some(axis_value) = axis_value {
                    for (id, address, range) in axis_mappings {
                        if matcher.match_address(address) {
                            let span = range[1] - range[0];
                            let mapped_value = if span != 0.0 {
                                (axis_value - range[0]) / span
                            } else {
                                0.0
                            };

                            into.push(OutputEvent::UpdateAxis(id.clone(), mapped_value));
                        }
                    }
                }

                if let Some(button_value) = button_value {
                    for (id, address) in button_mappings {
                        if matcher.match_address(address) {
                            into.push(OutputEvent::UpdateButton(id.clone(), button_value));
                        }
                    }
                }
            }
        }
    }
}

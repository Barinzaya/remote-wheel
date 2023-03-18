use std::sync::Arc;

use anyhow::{Context as _, Result as AnyResult};
use async_broadcast::{Receiver as BroadcastRx, RecvError as BroadcastRxErr};
use smol::channel::Sender as ChannelTx;

use crate::{config::MappingConfig, output::OutputEvent};

use super::config::OutputConfig;

pub(super) async fn run(
    config: OutputConfig,
    mappings: Arc<MappingConfig>,
    mut output_rx: BroadcastRx<OutputEvent>,
    outbound_tx: ChannelTx<Vec<u8>>,
) -> AnyResult<()> {
    log::info!("OSC output processing task starting...");

    let mut packet = rosc::OscPacket::Bundle(rosc::OscBundle {
        timetag: (0, 0).into(),
        content: config
            .pre_bundle
            .to_messages(&())
            .map(rosc::OscPacket::Message)
            .collect(),
    });

    let mut post_packets = config
        .post_bundle
        .to_messages(&())
        .map(rosc::OscPacket::Message)
        .collect::<Vec<_>>();

    let num_pre_packets = config.pre_bundle.len();

    log::info!("OSC output processing task started.");

    loop {
        match output_rx.recv().await {
            Ok(OutputEvent::UpdateAxis(id, value)) => {
                if let Some(mapping) = mappings.axis.get(&id) {
                    let rosc::OscPacket::Bundle(ref mut bundle) = packet else { unreachable!() };

                    bundle.content.extend(
                        mapping
                            .output
                            .osc
                            .on_update
                            .to_messages(&(value as f32))
                            .map(rosc::OscPacket::Message),
                    );
                }
            }

            Ok(OutputEvent::UpdateButton(id, pressed)) => {
                if let Some(mapping) = mappings.button.get(&id) {
                    let rosc::OscPacket::Bundle(ref mut bundle) = packet else { unreachable!() };

                    let specific_messages = if pressed {
                        &mapping.output.osc.on_press
                    } else {
                        &mapping.output.osc.on_release
                    };

                    bundle.content.extend(
                        specific_messages
                            .to_messages(&pressed)
                            .map(rosc::OscPacket::Message),
                    );

                    bundle.content.extend(
                        mapping
                            .output
                            .osc
                            .on_update
                            .to_messages(&pressed)
                            .map(rosc::OscPacket::Message),
                    );
                }
            }

            Ok(OutputEvent::Flush) => {
                let rosc::OscPacket::Bundle(ref mut bundle) = packet else { unreachable!() };
                if bundle.content.len() > num_pre_packets {
                    let post_start = bundle.content.len();
                    bundle.content.append(&mut post_packets);

                    let bytes =
                        rosc::encoder::encode(&packet).context("Failed to encode OSC packet")?;

                    if let Err(e) = outbound_tx.send(bytes).await {
                        log::warn!("Failed to transfer OSC packet data for sending: {e}");
                    }

                    let rosc::OscPacket::Bundle(ref mut bundle) = packet else { unreachable!() };
                    post_packets.extend(bundle.content.drain(post_start..));
                    bundle.content.truncate(num_pre_packets);
                }
            }

            Err(BroadcastRxErr::Overflowed(n)) => {
                log::warn!("OSC output processing task missed {} update(s)!", n);
            }

            Err(BroadcastRxErr::Closed) => {
                log::info!("OSC output processing task stopping (no inputs remaining)...");
                break;
            }
        }
    }

    log::info!("OSC output processing task stopped.");
    Ok(())
}

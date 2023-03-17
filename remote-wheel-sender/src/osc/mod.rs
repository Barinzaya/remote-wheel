mod config;
mod input;
mod output;

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{Context as _, Result as AnyResult};
use async_broadcast::{Receiver as BroadcastRx, Sender as BroadcastTx};
use futures::prelude::*;
use smol::channel::{Receiver as ChannelRx, Sender as ChannelTx};
use smol::net::UdpSocket;

use crate::config::MappingConfig;
use crate::output::OutputEvent;
pub use config::{
    AxisInputConfig, AxisOutputConfig, ButtonInputConfig, ButtonOutputConfig, Config,
};

pub async fn run(
    exec: Arc<smol::Executor<'static>>,
    config: Config,
    mappings: Arc<MappingConfig>,
    cancel_rx: ChannelRx<()>,
    value_tx: BroadcastTx<OutputEvent>,
    value_rx: BroadcastRx<OutputEvent>,
) -> AnyResult<()> {
    log::info!("OSC task starting...");

    let socket = UdpSocket::bind(config.input.address)
        .await
        .with_context(|| {
            format!(
                "Failed to bind to UDP input address {}",
                config.input.address
            )
        })?;

    let local_addr = socket
        .local_addr()
        .expect("Failed to get local address of UdpSocket");
    let remote_addr = config.output.address;

    let socket = Arc::new(socket);
    let (inbound_tx, inbound_rx) = smol::channel::bounded(16);
    let (outbound_tx, outbound_rx) = smol::channel::bounded(16);

    let mut recv_task = exec
        .spawn(run_recv(socket.clone(), inbound_tx, cancel_rx))
        .fuse();
    let mut input_task = exec
        .spawn(input::run(
            config.input,
            mappings.clone(),
            inbound_rx,
            value_tx,
        ))
        .fuse();
    let mut output_task = exec
        .spawn(output::run(
            config.output,
            mappings.clone(),
            value_rx,
            outbound_tx,
        ))
        .fuse();
    let mut send_task = exec
        .spawn(run_send(socket, remote_addr, outbound_rx))
        .fuse();

    log::info!("OSC task has started. Listening for input on {local_addr}, sending output to {remote_addr}.");

    let mut result = Ok(());
    loop {
        futures::select! {
            task_result = recv_task => {
                if let Err(e) = task_result {
                    log::error!("OSC receive task has stopped with an error: {e}");
                    result = result.and(Err(e));
                }
            },

            task_result = input_task => {
                if let Err(e) = task_result {
                    log::error!("OSC input processing task has stopped with an error: {e}");
                    result = result.and(Err(e));
                }
            },

            task_result = output_task => {
                if let Err(e) = task_result {
                    log::error!("OSC output processing task has stopped with an error: {e}");
                    result = result.and(Err(e));
                }
            },

            task_result = send_task => {
                if let Err(e) = task_result {
                    log::error!("OSC send task has stopped with an error: {e}");
                    result = result.and(Err(e));
                }
            },

            complete => break result,
        }
    }
}

async fn run_recv(
    socket: Arc<UdpSocket>,
    inbound_tx: ChannelTx<rosc::OscPacket>,
    cancel_rx: ChannelRx<()>,
) -> AnyResult<()> {
    let mut buffer = vec![0u8; 16384];
    log::info!("OSC receive task started.");

    loop {
        let Ok((len, addr)) = futures::select_biased!{
			_ = cancel_rx.recv().fuse() => {
				log::info!("OSC receive task stopping (shutdown).");
				break Ok(());
			},
			result = socket.recv_from(&mut buffer).fuse() => result,
		} else { continue };

        log::debug!("Received {len} bytes of data from {addr}.");

        let data = &buffer[..len];
        let Ok((_, packet)) = rosc::decoder::decode_udp(data) else {
			log::warn!("Received {len} bytes of invalid data from {addr}.");
			continue;
		};

        if inbound_tx.send(packet).await.is_err() {
            log::info!("OSC receive task stopping (input processing task has stopped.).");
            break Ok(());
        }
    }
}

async fn run_send(
    socket: Arc<UdpSocket>,
    addr: SocketAddr,
    outbound_rx: ChannelRx<Vec<u8>>,
) -> AnyResult<()> {
    log::info!("OSC send task started.");
    loop {
        let Ok(data) = outbound_rx.recv().await else {
			log::info!("OSC send task stopping (output processing task has stopped).");
			break Ok(());
		};

        log::debug!("Sending {} bytes of data to {}.", data.len(), addr);

        if let Err(e) = socket.send_to(&data, addr).await {
            log::error!("Failed to send OSC data to {addr}: {e}");
        }
    }
}

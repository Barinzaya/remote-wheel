use std::net::{SocketAddr, SocketAddrV4, Ipv4Addr, Ipv6Addr, SocketAddrV6};

use anyhow::{Context as _, Result as AnyResult};
use async_broadcast::{Receiver as BroadcastRx, RecvError as BroadcastRxErr};
use linear_map::{LinearMap};
use serde::{Deserialize};
use smol::net::{UdpSocket};

use super::{OutputEvent};

#[derive(Clone, Debug, Deserialize)]
pub struct OscConfig {
	address: SocketAddr,
}

#[derive(Clone, Debug, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct OscInputConfig {
	paths: LinearMap<String, Vec<OscParameter>>,
}

impl Default for OscConfig {
	fn default() -> Self {
		OscConfig {
			address: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 19794)),
		}
	}
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum OscParameter {
	Int(i32),
	Long(i64),

	Float(f32),
	Double(f64),

	String(String),
	Bool(bool),

	Nil,
	Inf,

	Input,
}

impl OscConfig {
	pub fn validate(&self) -> AnyResult<()> {
		Ok(())
	}
}

impl OscInputConfig {
	pub fn validate(&self) -> AnyResult<()> {
		Ok(())
	}
}

impl OscParameter {
	fn to_rosc(&self, input: f32) -> rosc::OscType {
		match *self {
			OscParameter::Int(i) => rosc::OscType::Int(i),
			OscParameter::Long(i) => rosc::OscType::Long(i),

			OscParameter::Float(f) => rosc::OscType::Float(f),
			OscParameter::Double(f) => rosc::OscType::Double(f),

			OscParameter::String(ref s) => rosc::OscType::String(s.clone()),
			OscParameter::Bool(b) => rosc::OscType::Bool(b),

			OscParameter::Nil => rosc::OscType::Nil,
			OscParameter::Inf => rosc::OscType::Inf,

			OscParameter::Input => rosc::OscType::Float(input),
		}
	}
}

pub async fn run(config: OscConfig, mut recv: BroadcastRx<OutputEvent>) -> AnyResult<()> {
	log::info!("OSC task starting...");

	let socket_addr = match config.address {
		SocketAddr::V4(_) => SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)),
		SocketAddr::V6(_) => SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, 0, 0, 0)),
	};

	let socket = UdpSocket::bind(socket_addr).await
		.with_context(|| format!("Failed to bind to UDP socket {}", socket_addr))?;
	socket.connect(&config.address).await
		.with_context(|| format!("Failed to connect to UDP address {}", config.address))?;

	let mut packet = rosc::OscPacket::Bundle(rosc::OscBundle {
		timetag: (0, 0).into(),
		content: Vec::new(),
	});

	log::info!("OSC task started.");

	loop {
		match recv.recv().await {
			Ok(OutputEvent::Update(input, value)) => {
				if let Some(ref config) = input.osc {
					let rosc::OscPacket::Bundle(ref mut bundle) = packet else { unreachable!() };

					bundle.content.clear();
					bundle.content.extend(config.paths.iter()
						.map(|(path, params)| rosc::OscMessage {
							addr: path.clone(),
							args: params.iter()
								.map(|p| p.to_rosc(value as f32))
								.collect(),
						})
						.map(rosc::OscPacket::Message));

					let bytes = rosc::encoder::encode(&packet)
						.context("Failed to encode OSC packet")?;
					socket.send(&bytes).await
						.context("Failed to send OSc packet")?;
				}
			},

			Err(BroadcastRxErr::Overflowed(n)) => {
				log::warn!("OSC sender missed {} update(s)!", n);
			},

			Err(BroadcastRxErr::Closed) => {
				log::info!("OSC task stopping (shutdown)...");
				break;
			},
		}
	}

	log::info!("OSC task stopped.");
	Ok(())
}

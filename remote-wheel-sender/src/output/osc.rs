use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

use anyhow::{Context as _, Result as AnyResult};
use async_broadcast::{Receiver as BroadcastRx, RecvError as BroadcastRxErr};
use linear_map::{LinearMap};
use serde::{Deserialize};
use smol::net::{UdpSocket};

use super::{OutputEvent};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct OscConfig {
	address: SocketAddr,

	#[serde(default)]
	pre_bundle: OscMessages,

	#[serde(default)]
	post_bundle: OscMessages,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct OscInputAxisConfig {
	#[serde(flatten)]
	on_change: OscMessages,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct OscInputButtonConfig {
	#[serde(default)]
	on_press: OscMessages,

	#[serde(default)]
	on_release: OscMessages,

	#[serde(flatten)]
	on_change: OscMessages,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct OscMessages(LinearMap<String, Vec<OscParameter>>);

impl Default for OscConfig {
	fn default() -> Self {
		OscConfig {
			address: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 19794)),

			pre_bundle: OscMessages::default(),
			post_bundle: OscMessages::default(),
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

	Bool(bool),
	String(String),

	Nil,
	Inf,

	Input,
}

impl OscConfig {
	pub fn validate(&self) -> AnyResult<()> {
		Ok(())
	}
}

impl OscInputAxisConfig {
	pub fn validate(&self) -> AnyResult<()> {
		Ok(())
	}
}

impl OscInputButtonConfig {
	pub fn validate(&self) -> AnyResult<()> {
		Ok(())
	}
}

impl OscParameter {
	fn to_rosc(&self, input: &rosc::OscType) -> rosc::OscType {
		match *self {
			OscParameter::Int(i) => rosc::OscType::Int(i),
			OscParameter::Long(i) => rosc::OscType::Long(i),

			OscParameter::Float(f) => rosc::OscType::Float(f),
			OscParameter::Double(f) => rosc::OscType::Double(f),

			OscParameter::String(ref s) => rosc::OscType::String(s.clone()),
			OscParameter::Bool(b) => rosc::OscType::Bool(b),

			OscParameter::Nil => rosc::OscType::Nil,
			OscParameter::Inf => rosc::OscType::Inf,

			OscParameter::Input => input.clone(),
		}
	}
}

impl OscMessages {
	fn to_messages<'m>(&'m self, input: &'m rosc::OscType) -> impl 'm + Iterator<Item = rosc::OscMessage> + DoubleEndedIterator + ExactSizeIterator {
		self.0.iter().map(|(address, params)| rosc::OscMessage {
			addr: address.clone(),
			args: params.iter()
				.map(|p| p.to_rosc(input))
				.collect(),
		})
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
		content: config.pre_bundle.to_messages(&rosc::OscType::Nil)
			.map(rosc::OscPacket::Message)
			.collect(),
	});

	let num_pre_packets = config.pre_bundle.0.len();
	let mut post_packets = config.post_bundle.to_messages(&rosc::OscType::Nil)
		.map(rosc::OscPacket::Message)
		.collect::<Vec<_>>();

	log::info!("OSC task started.");

	loop {
		match recv.recv().await {
			Ok(OutputEvent::UpdateAxis(input, value)) => {
				let rosc::OscPacket::Bundle(ref mut bundle) = packet else { unreachable!() };

				bundle.content.extend(input.osc.on_change.to_messages(&rosc::OscType::Float(value as f32))
					.map(rosc::OscPacket::Message));
			},

			Ok(OutputEvent::UpdateButton(input, pressed)) => {
				let rosc::OscPacket::Bundle(ref mut bundle) = packet else { unreachable!() };

				let specific_messages = if pressed { &input.osc.on_press } else { &input.osc.on_release };
				bundle.content.extend(specific_messages.to_messages(&rosc::OscType::Bool(pressed))
					.map(rosc::OscPacket::Message));

				bundle.content.extend(input.osc.on_change.to_messages(&rosc::OscType::Bool(pressed))
					.map(rosc::OscPacket::Message));
			},

			Ok(OutputEvent::Flush) => {
				let rosc::OscPacket::Bundle(ref mut bundle) = packet else { unreachable!() };
				if bundle.content.len() > num_pre_packets {
					let post_start = bundle.content.len();
					bundle.content.append(&mut post_packets);

					let bytes = rosc::encoder::encode(&packet)
						.context("Failed to encode OSC packet")?;
					socket.send(&bytes).await
						.context("Failed to send OSC packet")?;

					let rosc::OscPacket::Bundle(ref mut bundle) = packet else { unreachable!() };
					post_packets.extend(bundle.content.drain(post_start..));
					bundle.content.truncate(num_pre_packets);
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

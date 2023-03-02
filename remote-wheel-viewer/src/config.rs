use std::borrow::{Cow};
use std::io::{ErrorKind};
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result as AnyResult, bail};
use serde::{Deserialize};
use smol::net::{SocketAddr};

#[derive(Debug, Deserialize)]
pub struct AppConfig {
	#[serde(default)]
	pub background: Color,
	pub wheel: PathBuf,

	pub osc: OscConfig,
}

#[derive(Debug, Deserialize)]
pub struct OscConfig {
	pub address: SocketAddr,
}

impl AppConfig {
	pub fn read_from(path: impl AsRef<Path>) -> AnyResult<AppConfig> {
		Self::read_from_path(path.as_ref())
	}

	fn read_from_path(path: &Path) -> AnyResult<AppConfig> {
		let raw: Cow<[u8]> = match std::fs::read(path) {
			Ok(s) => Cow::Owned(s),
			Err(e) if e.kind() == ErrorKind::NotFound => {
				let default = include_bytes!("default-config.yaml");

				std::fs::write(path, default)
					.with_context(|| format!("Failed to write default configuration to <{}>", path.display()))?;

				Cow::Borrowed(default)
			},
			Err(e) => Err(e)
				.with_context(|| format!("Failed to read configuration from <{}>", path.display()))?,
		};

		let config = serde_yaml::from_slice(&raw)
			.with_context(|| format!("Failed to parse configuration from <{}>", path.display()))?;
		Ok(config)
	}
}

#[derive(Clone, Copy, Debug, Default, serde_with::DeserializeFromStr)]
pub struct Color(u8, u8, u8, u8);

impl std::str::FromStr for Color {
	type Err = anyhow::Error;
	fn from_str(s: &str) -> AnyResult<Self> {
		match s.len() {
			3 => Ok(Color(
				17 * u8::from_str_radix(&s[0..1], 16).context("Invalid red component")?,
				17 * u8::from_str_radix(&s[1..2], 16).context("Invalid green component")?,
				17 * u8::from_str_radix(&s[2..3], 16).context("Invalid blue component")?,
				255,
			)),

			4 => Ok(Color(
				17 * u8::from_str_radix(&s[0..1], 16).context("Invalid red component")?,
				17 * u8::from_str_radix(&s[1..2], 16).context("Invalid green component")?,
				17 * u8::from_str_radix(&s[2..3], 16).context("Invalid blue component")?,
				17 * u8::from_str_radix(&s[3..4], 16).context("Invalid alpha component")?,
			)),

			6 => Ok(Color(
				u8::from_str_radix(&s[0..2], 16).context("Invalid red component")?,
				u8::from_str_radix(&s[2..4], 16).context("Invalid green component")?,
				u8::from_str_radix(&s[4..6], 16).context("Invalid blue component")?,
				255,
			)),

			8 => Ok(Color(
				u8::from_str_radix(&s[0..2], 16).context("Invalid red component")?,
				u8::from_str_radix(&s[2..4], 16).context("Invalid green component")?,
				u8::from_str_radix(&s[4..6], 16).context("Invalid blue component")?,
				u8::from_str_radix(&s[6..8], 16).context("Invalid alpha component")?,
			)),

			_ => bail!("Invalid color string (must be 3, 4, 6, or 8 hex characters)."),
		}
	}
}

impl From<Color> for eframe::egui::Color32 {
	fn from(value: Color) -> Self {
		Self::from_rgba_unmultiplied(value.0, value.1, value.2, value.3)
	}
}

use std::borrow::Cow;
use std::fmt::Display;
use std::io::ErrorKind;
use std::path::{Path};
use std::sync::{Arc};

use anyhow::{Context as _, Result as AnyResult, ensure};
use serde::{Deserialize};

#[derive(Debug, Deserialize)]
pub struct AppConfig {
	#[cfg(feature = "osc")]
    pub osc: crate::output::osc::OscConfig,

	#[serde(default)]
	pub inputs: Vec<InputConfig>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum InputConfig {
	Axis(Arc<InputAxisConfig>),
	Button(Arc<InputButtonConfig>),
}

#[derive(Debug, Deserialize)]
pub struct InputAxisConfig {
	pub controller: String,
	pub axis: Axis,

	#[serde(default = "default_range")]
	pub range: [f64; 2],

	#[cfg(feature = "osc")]
	#[serde(default)]
	pub osc: crate::output::osc::OscInputAxisConfig,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct InputButtonConfig {
	pub controller: String,
	pub button: u32,

	#[cfg(feature = "osc")]
	#[serde(default)]
	pub osc: crate::output::osc::OscInputButtonConfig,
}

fn default_range() -> [f64; 2] {
	[0.0, 1.0]
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "kebab-case")]
pub enum Axis {
	X, Y, Z,
	Rx, Ry, Rz,
	Slider, Dial,
}

impl AppConfig {
	pub async fn read_from(path: impl AsRef<Path>) -> AnyResult<AppConfig> {
		Self::read_from_path(path.as_ref()).await
	}

	async fn read_from_path(path: &Path) -> AnyResult<AppConfig> {
		let raw: Cow<[u8]> = match smol::fs::read(path).await {
			Ok(s) => Cow::Owned(s),
			Err(e) if e.kind() == ErrorKind::NotFound => {
				let default = include_bytes!("default-config.yaml");

				smol::fs::write(path, default).await
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

	pub fn validate(&self) -> AnyResult<()> {
		#[cfg(feature = "osc")]
		{
			self.osc.validate()?;
		}

		for input in &self.inputs {
			input.validate()?;
		}

		Ok(())
	}
}

impl InputConfig {
	pub fn controller(&self) -> &str {
		match *self {
			InputConfig::Axis(ref axis) => &axis.controller,
			InputConfig::Button(ref button) => &button.controller,
		}
	}

	pub fn validate(&self) -> AnyResult<()> {
		match *self {
			InputConfig::Axis(ref axis) => axis.validate(),
			InputConfig::Button(ref button) => button.validate(),
		}
	}
}

impl InputAxisConfig {
	pub fn validate(&self) -> AnyResult<()> {
		#[cfg(feature = "osc")]
		self.osc.validate()?;

		Ok(())
	}
}

impl InputButtonConfig {
	pub fn validate(&self) -> AnyResult<()> {
		ensure!(self.button > 0, "Input button must be a number starting at 1.");

		#[cfg(feature = "osc")]
		self.osc.validate()?;

		Ok(())
	}
}

impl Display for Axis {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let name = match *self {
			Axis::X => "X",
			Axis::Y => "Y",
			Axis::Z => "Z",
			Axis::Rx => "Rx",
			Axis::Ry => "Ry",
			Axis::Rz => "Rz",
			Axis::Slider => "Slider",
			Axis::Dial => "Dial",
		};

		write!(f, "{}", name)
	}
}

impl TryFrom<u8> for Axis {
	type Error = ();

	fn try_from(value: u8) -> Result<Self, ()> {
        match value {
			0 => Ok(Axis::X),
			1 => Ok(Axis::Y),
			2 => Ok(Axis::Z),
			3 => Ok(Axis::Rx),
			4 => Ok(Axis::Ry),
			5 => Ok(Axis::Rz),
			6 => Ok(Axis::Slider),
			7 => Ok(Axis::Dial),
            _ => Err(()),
		}
	}
}

use std::borrow::Cow;
use std::fmt::Debug;
use std::io::ErrorKind;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context as _, Result as AnyResult};
use hashbrown::HashMap;
use serde::Deserialize;
use string_cache::DefaultAtom;

use crate::controller;

#[cfg(feature = "osc")]
use crate::osc::{
    AxisInputConfig as OscAxisInputConfig, AxisOutputConfig as OscAxisOutputConfig,
    ButtonInputConfig as OscButtonInputConfig, ButtonOutputConfig as OscButtonOutputConfig,
    Config as OscConfig,
};
#[cfg(not(feature = "osc"))]
use serde::de::{
    IgnoredAny as OscConfig, IgnoredAny as OscAxisInputConfig, IgnoredAny as OscAxisOutputConfig,
    IgnoredAny as OscButtonInputConfig, IgnoredAny as OscButtonOutputConfig,
};

#[cfg(feature = "vmc")]
use crate::vmc::{
    AxisOutputConfig as VmcAxisOutputConfig, ButtonOutputConfig as VmcButtonOutputConfig,
    Config as VmcConfig,
};
#[cfg(not(feature = "vmc"))]
use serde::de::{
    IgnoredAny as VmcConfig, IgnoredAny as VmcAxisOutputConfig, IgnoredAny as VmcButtonOutputConfig,
};

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct AppConfig {
    pub osc: OscConfig,
    pub vmc: VmcConfig,

    #[serde(flatten)]
    pub mappings: Arc<MappingConfig>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct MappingConfig {
    pub axis: HashMap<DefaultAtom, AxisConfig>,
    pub button: HashMap<DefaultAtom, ButtonConfig>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct AxisConfig {
    pub input: Vec<AxisInputConfig>,
    pub output: AxisOutputConfig,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AxisInputConfig {
    Controller(controller::AxisInputConfig),
    Osc(OscAxisInputConfig),
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct AxisOutputConfig {
    pub osc: OscAxisOutputConfig,
    pub vmc: VmcAxisOutputConfig,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct ButtonConfig {
    pub input: Vec<ButtonInputConfig>,
    pub output: ButtonOutputConfig,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum ButtonInputConfig {
    Controller(controller::ButtonInputConfig),
    Osc(OscButtonInputConfig),
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct ButtonOutputConfig {
    pub osc: OscButtonOutputConfig,
    pub vmc: VmcButtonOutputConfig,
}

impl AppConfig {
    pub async fn read_from(path: impl AsRef<Path>) -> AnyResult<AppConfig> {
        Self::read_from_path(path.as_ref()).await
    }

    async fn read_from_path(path: &Path) -> AnyResult<AppConfig> {
        let raw: Cow<str> = match smol::fs::read_to_string(path).await {
            Ok(s) => Cow::Owned(s),
            Err(e) if e.kind() == ErrorKind::NotFound => {
                log::info!("No configuration file found at <{}>. Prompting for which configuration to write.", path.display());
                log::logger().flush();

                let theme = dialoguer::theme::ColorfulTheme::default();
                let mut select = dialoguer::Select::with_theme(&theme);
                select.with_prompt("Which best describes how you want to use this application?");
                select.report(false);

                for sample in CONFIGS {
                    select.item(sample.desc);
                }

                let index = select.interact()
                    .context("Failed to prompt for default configuration")?;
                let config = &CONFIGS[index];

                log::info!(
                    "{} configuration was selected. Opening editor for configuration.",
                    config.name
                );

                let contents = dialoguer::Editor::new()
                    .extension(".toml")
                    .trim_newlines(false)
                    .edit(config.contents)
                    .context("Failed to open editor for initial configuration")?;

                let (edited, contents) = (
                    contents.is_some(),
                    contents
                        .map(Cow::Owned)
                        .unwrap_or(Cow::Borrowed(config.contents)),
                );

                if edited {
                    log::info!(
                        "Initial configuration was edited. Saving edited configuration to <{}>.",
                        path.display()
                    );
                } else {
                    log::info!(
                        "Initial configuration was not edited. Saving base configuration to <{}>.",
                        path.display()
                    );
                }

                smol::fs::write(path, contents.as_ref())
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to write default configuration to <{}>",
                            path.display()
                        )
                    })?;

                contents
            }
            Err(e) => Err(e).with_context(|| {
                format!("Failed to read configuration from <{}>", path.display())
            })?,
        };

        let config = toml::from_str(&raw)
            .with_context(|| format!("Failed to parse configuration from <{}>", path.display()))?;
        Ok(config)
    }
}

struct SampleConfig {
    name: &'static str,
    desc: &'static str,
    contents: &'static str,
}

static CONFIGS: &[SampleConfig] = &[
    SampleConfig {
        name: "Viewer",
        desc: "2D wheel overlay with the Viewer application",
        contents: include_str!("with-viewer.toml"),
    },

    SampleConfig {
        name: "VNyan (Single PC)",
        desc: "3D wheel overlay with VNyan (this PC has the controller connected and is also running VNyan)",
        contents: include_str!("with-vnyan-single.toml"),
    },

    SampleConfig {
        name: "VNyan (Game PC)",
        desc: "3D wheel overlay with VNyan (this PC has the controller connected; VNyan is on another PC)",
        contents: include_str!("with-vnyan-game.toml"),
    },

    SampleConfig {
        name: "VNyan (Stream PC)",
        desc: "3D wheel overlay with VNyan (this PC is running VNyan; the controller is connected to another PC)",
        contents: include_str!("with-vnyan-stream.toml"),
    },

    SampleConfig {
        name: "Reference",
        desc: "Other (manual configuration)",
        contents: include_str!("reference.toml"),
    },
];

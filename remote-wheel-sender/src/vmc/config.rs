use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use hashbrown::HashMap;
use linear_map::LinearMap;
use serde::Deserialize;
use string_cache::DefaultAtom;

use super::device::Device;

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct Config {
    enabled: bool,
    pub(super) report_interval: Option<f64>,

    pub(super) receive_addr: SocketAddr,
    pub(super) send_addr: SocketAddr,

    pub(super) device: HashMap<DefaultAtom, Device>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct AxisOutputConfig {
    pub(super) on_change: EventConfig<[f32; 2]>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct ButtonOutputConfig {
    #[serde(default)]
    pub(super) on_change: EventConfig<[f32; 2]>,

    #[serde(default)]
    pub(super) on_press: EventConfig<f32>,

    #[serde(default)]
    pub(super) on_release: EventConfig<f32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct EventConfig<T> {
    pub(super) blendshape: LinearMap<DefaultAtom, T>,
    pub(super) device: LinearMap<DefaultAtom, T>,
}

impl Config {
    pub fn enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            enabled: false,
            receive_addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 3332)),
            send_addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 3333)),
            report_interval: Some(60.0),

            device: HashMap::new(),
        }
    }
}

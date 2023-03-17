use std::net::{Ipv4Addr, SocketAddr};

use linear_map::LinearMap;
use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct Config {
    enabled: bool,
    pub(super) input: InputConfig,
    pub(super) output: OutputConfig,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub(super) struct InputConfig {
    pub(super) address: SocketAddr,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub(super) struct OutputConfig {
    pub(super) address: SocketAddr,

    pub(super) pre_bundle: BundleConfig<NullInput>,
    pub(super) post_bundle: BundleConfig<NullInput>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct AxisInputConfig {
    pub(super) address: String,
    pub(super) range: [f64; 2],
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct AxisOutputConfig {
    pub(super) on_change: BundleConfig<FloatRangeInput>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct ButtonInputConfig {
    pub(super) address: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct ButtonOutputConfig {
    pub(super) on_change: BundleConfig<BoolInput>,
    pub(super) on_press: BundleConfig<BoolInput>,
    pub(super) on_release: BundleConfig<BoolInput>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub(super) struct BundleConfig<I> {
    messages: LinearMap<String, Vec<OscParameter<I>>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum OscParameter<I> {
    Int(i32),
    Long(i64),

    Float(f32),
    Double(f64),

    Bool(bool),
    String(String),

    Nil,
    Inf,

    Input(I),
}

impl<I: OscInput> OscParameter<I> {
    fn to_rosc(&self, raw: &I::Param) -> rosc::OscType {
        match *self {
            OscParameter::Int(i) => rosc::OscType::Int(i),
            OscParameter::Long(i) => rosc::OscType::Long(i),

            OscParameter::Float(f) => rosc::OscType::Float(f),
            OscParameter::Double(f) => rosc::OscType::Double(f),

            OscParameter::String(ref s) => rosc::OscType::String(s.clone()),
            OscParameter::Bool(b) => rosc::OscType::Bool(b),

            OscParameter::Nil => rosc::OscType::Nil,
            OscParameter::Inf => rosc::OscType::Inf,

            OscParameter::Input(ref i) => i.to_rosc(raw),
        }
    }
}

pub trait OscInput {
    type Param;
    fn to_rosc(&self, raw: &Self::Param) -> rosc::OscType;
}

impl<I: OscInput> BundleConfig<I> {
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn to_messages<'m>(
        &'m self,
        raw: &'m I::Param,
    ) -> impl 'm + Iterator<Item = rosc::OscMessage> + DoubleEndedIterator + ExactSizeIterator {
        self.messages
            .iter()
            .map(|(address, params)| rosc::OscMessage {
                addr: address.to_string(),
                args: params.iter().map(|p| p.to_rosc(raw)).collect(),
            })
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct BoolInput;

impl OscInput for BoolInput {
    type Param = bool;
    fn to_rosc(&self, raw: &Self::Param) -> rosc::OscType {
        rosc::OscType::Bool(*raw)
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(from = "(f32, f32)")]
pub struct FloatRangeInput(f32, f32);

impl From<(f32, f32)> for FloatRangeInput {
    fn from((a, b): (f32, f32)) -> Self {
        FloatRangeInput(a, b - a)
    }
}

impl OscInput for FloatRangeInput {
    type Param = f32;
    fn to_rosc(&self, raw: &Self::Param) -> rosc::OscType {
        rosc::OscType::Float(self.0 + *raw * self.1)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct NullInput;

impl OscInput for NullInput {
    type Param = ();
    fn to_rosc(&self, _: &Self::Param) -> rosc::OscType {
        rosc::OscType::Nil
    }
}

impl Config {
    pub fn enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            address: SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0),
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            address: SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 19794),

            pre_bundle: BundleConfig::default(),
            post_bundle: BundleConfig::default(),
        }
    }
}

impl<I> Default for BundleConfig<I> {
    fn default() -> Self {
        Self {
            messages: LinearMap::new(),
        }
    }
}

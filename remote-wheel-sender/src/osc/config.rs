use std::borrow::Cow;
use std::marker::PhantomData;
use std::net::{Ipv4Addr, SocketAddr};

use linear_map::LinearMap;
use serde::{Deserialize, Deserializer, de::Error as _};

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
    pub(super) on_update: BundleConfig<FloatRangeInput>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct ButtonInputConfig {
    pub(super) address: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct ButtonOutputConfig {
    pub(super) on_press: BundleConfig<BoolInput>,
    pub(super) on_release: BundleConfig<BoolInput>,
    pub(super) on_update: BundleConfig<BoolInput>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub(super) struct BundleConfig<I> {
    messages: LinearMap<String, Vec<OscParameter<I>>>,
}

#[derive(Clone, Debug)]
pub(super) enum OscParameter<I> {
    Int(i32),
    Long(i64),

    Float(f32),
    Double(f64),

    Bool(bool),
    String(String),

    Input(I),
}

impl<I: OscInput> OscParameter<I> {
    fn to_rosc(&self, raw: &I::Param) -> rosc::OscType {
        match *self {
            OscParameter::Int(i) => rosc::OscType::Int(i),
            OscParameter::Long(i) => rosc::OscType::Long(i),

            OscParameter::Float(f) => rosc::OscType::Float(f),
            OscParameter::Double(f) => rosc::OscType::Double(f),

            OscParameter::Bool(b) => rosc::OscType::Bool(b),
            OscParameter::String(ref s) => rosc::OscType::String(s.clone()),

            OscParameter::Input(ref i) => i.to_rosc(raw),
        }
    }
}

struct OscParameterVisitor<'de, I: Deserialize<'de>>(PhantomData<(&'de (), I)>);

impl<'de, I: Deserialize<'de>> Deserialize<'de> for OscParameter<I> {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        de.deserialize_any(OscParameterVisitor::<'de, I>(PhantomData))
    }
}

impl<'de, I: Deserialize<'de>> serde::de::Visitor<'de> for OscParameterVisitor<'de, I> {
    type Value = OscParameter<I>;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "OSC parameter value (int, float, bool, or tagged value)")
    }

    fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<Self::Value, E> {
        Ok(OscParameter::Bool(v))
    }

    fn visit_f32<E: serde::de::Error>(self, v: f32) -> Result<Self::Value, E> {
        Ok(OscParameter::Float(v))
    }

    fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Self::Value, E> {
        Ok(OscParameter::Float(v as f32))
    }

    fn visit_i8<E: serde::de::Error>(self, v: i8) -> Result<Self::Value, E> {
        self.visit_i32(v as i32)
    }

    fn visit_i16<E: serde::de::Error>(self, v: i16) -> Result<Self::Value, E> {
        self.visit_i32(v as i32)
    }

    fn visit_i32<E: serde::de::Error>(self, v: i32) -> Result<Self::Value, E> {
        Ok(OscParameter::Int(v))
    }

    fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
        if let Ok(v) = i32::try_from(v) {
            Ok(OscParameter::Int(v))
        } else {
            Ok(OscParameter::Long(v))
        }
    }

    fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let key: Cow<'de, str> = map.next_key()?
            .ok_or_else(|| A::Error::invalid_length(0, &"a single entry with a key of bool/int/long/float/double/string/input"))?;

        let value = match key.as_ref() {
            "bool" => OscParameter::Bool(map.next_value()?),
            "int" => OscParameter::Int(map.next_value()?),
            "long" => OscParameter::Long(map.next_value()?),
            "float" => OscParameter::Float(map.next_value()?),
            "double" => OscParameter::Double(map.next_value()?),
            "string" => OscParameter::String(map.next_value()?),
            "input" => OscParameter::Input(map.next_value()?),

            _ => return Err(A::Error::unknown_field(key.as_ref(), &["bool", "int", "long", "float", "double", "string", "input"])),
        };

        if map.next_key::<serde::de::IgnoredAny>()?.is_some() {
            return Err(A::Error::invalid_length(2, &"a single entry with a key of bool/int/long/float/double/string/input"));
        }

        Ok(value)
    }

    fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Self::Value, E> {
        Ok(OscParameter::String(v))
    }

    fn visit_u8<E: serde::de::Error>(self, v: u8) -> Result<Self::Value, E> {
        self.visit_i32(v as i32)
    }

    fn visit_u16<E: serde::de::Error>(self, v: u16) -> Result<Self::Value, E> {
        self.visit_i32(v as i32)
    }

    fn visit_u32<E: serde::de::Error>(self, v: u32) -> Result<Self::Value, E> {
        if let Ok(v) = i32::try_from(v) {
            Ok(OscParameter::Int(v))
        } else {
            Ok(OscParameter::Long(v as i64))
        }
    }

    fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
        if let Ok(v) = i32::try_from(v) {
            Ok(OscParameter::Int(v))
        } else if let Ok(v) = i64::try_from(v) {
            Ok(OscParameter::Long(v))
        } else {
            Err(E::invalid_value(serde::de::Unexpected::Unsigned(v), &"integer out of range"))
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

use std::sync::{Arc};

use crate::config::{InputAxisConfig, InputButtonConfig};

#[cfg(feature = "osc")]
pub mod osc;

#[derive(Clone, Debug)]
pub enum OutputEvent {
	UpdateAxis(Arc<InputAxisConfig>, f64),
	UpdateButton(Arc<InputButtonConfig>, bool),
    Flush,
}

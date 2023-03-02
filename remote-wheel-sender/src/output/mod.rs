use std::sync::{Arc};

use crate::config::InputConfig;

#[cfg(feature = "osc")]
pub mod osc;

#[derive(Clone, Debug)]
pub enum OutputEvent {
	Update(Arc<InputConfig>, f64),
}

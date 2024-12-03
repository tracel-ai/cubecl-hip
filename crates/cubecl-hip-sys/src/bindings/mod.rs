#[cfg(feature = "rocm_622")]
mod bindings_622;
#[cfg(feature = "rocm_622")]
pub use bindings_622::*;
#[cfg(feature = "rocm_624")]
mod bindings_624;
#[cfg(feature = "rocm_624")]
pub use bindings_624::*;

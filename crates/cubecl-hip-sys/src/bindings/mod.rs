#[cfg(feature = "rocm__6_2_2")]
mod bindings_622;
#[cfg(feature = "rocm__6_2_2")]
pub use bindings_622::*;
#[cfg(feature = "rocm__6_2_4")]
mod bindings_624;
#[cfg(feature = "rocm__6_2_4")]
pub use bindings_624::*;

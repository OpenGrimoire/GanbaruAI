//! Provider driver contracts and built-in implementations.

mod claude;
mod codex;
mod cursor;
mod driver;
mod opencode;
mod registry;
mod unsupported;

pub use driver::*;
pub use registry::ProviderDriverRegistry;
pub use unsupported::*;

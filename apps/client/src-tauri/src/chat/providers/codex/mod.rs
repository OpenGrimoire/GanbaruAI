//! Codex app-server provider.

mod driver;
mod home;
mod normalizer;
mod protocol;
mod session;
mod transport;

pub use driver::CodexProviderDriver;

#[cfg(test)]
mod tests;

//! Provider processes, transports, drivers, event sinks, and registry.

#![allow(dead_code)]

pub mod chat;

pub use chat::{process, providers};

#[cfg(test)]
pub(crate) fn test_block_on<F>(future: F) -> F::Output
where
    F: std::future::Future,
{
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("provider test runtime must start")
        .block_on(future)
}

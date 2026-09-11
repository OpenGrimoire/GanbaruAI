//! Notes domain, persistence, transfers, history, assets, and validation.

pub mod image_metadata;
pub mod notes;

pub use notes::models::*;

#[cfg(test)]
pub(crate) fn test_block_on<F>(future: F) -> F::Output
where
    F: std::future::Future,
{
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Notes test runtime must start")
        .block_on(future)
}

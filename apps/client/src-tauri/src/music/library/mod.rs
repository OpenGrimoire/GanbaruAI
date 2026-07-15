pub(crate) mod commands;
mod error;
pub(crate) mod fixtures;
mod models;
mod queries;
mod rows;
mod search;
mod validation;
mod writes;

#[cfg(test)]
pub use error::MusicLibraryErrorCode;
pub use error::{MusicLibraryError, MusicLibraryResult};
pub use models::*;
pub(crate) use rows::*;
pub(crate) use validation::*;

#[cfg(test)]
mod tests;

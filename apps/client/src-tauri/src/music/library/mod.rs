pub(crate) mod commands;
mod error;
pub(crate) mod fixtures;
mod item_repair;
pub(crate) mod local_refresh;
mod models;
mod playlist_edits;
mod queries;
mod relink;
mod rows;
mod search;
mod source_lifecycle;
mod validation;
mod writes;
mod youtube;

#[cfg(test)]
pub use error::MusicLibraryErrorCode;
pub use error::{MusicLibraryError, MusicLibraryResult};
pub use models::*;
pub(crate) use rows::*;
pub(crate) use validation::*;

#[cfg(test)]
mod query_tests;
#[cfg(test)]
mod relink_tests;
#[cfg(test)]
mod source_lifecycle_tests;
#[cfg(test)]
mod tests;

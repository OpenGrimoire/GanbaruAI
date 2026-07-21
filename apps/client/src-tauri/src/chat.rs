//! Local coding-agent Chat contracts.
//!
//! The module freezes provider, event, lifecycle, credential, configuration,
//! and workspace authorization boundaries before runtime processes exist.

pub mod config;
pub mod credentials;
pub mod device_state;
pub mod events;
pub mod ingestion;
pub mod models;
pub mod process;
pub mod providers;
pub mod repository;
pub mod runtime;
pub mod settings_commands;
pub mod state;
pub mod thread_commands;
pub mod workspace;
pub mod workspace_commands;

#[cfg(test)]
mod tests;

//! Local coding-agent Chat contracts.
//!
//! The module freezes provider, event, lifecycle, credential, configuration,
//! and workspace authorization boundaries before runtime processes exist.

pub mod benchmark;
pub mod checkpoint_commands;
pub mod checkpoints;
pub mod config;
pub mod credentials;
pub mod device_state;
pub mod diagnostics_commands;
pub mod draft_commands;
pub mod events;
pub mod ingestion;
pub mod interaction_commands;
pub mod models;
pub mod process;
pub mod provider_files;
pub mod providers;
pub mod repository;
pub mod restore_commands;
pub mod runtime;
pub mod send_commands;
pub mod settings_commands;
pub mod state;
pub mod terminal;
pub mod terminal_commands;
pub mod thread_commands;
pub mod workspace;
pub mod workspace_commands;
pub mod workspace_files;

#[cfg(test)]
mod tests;

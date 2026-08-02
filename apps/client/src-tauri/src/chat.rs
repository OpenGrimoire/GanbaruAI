//! Local coding-agent Chat contracts.
//!
//! The module freezes provider, event, lifecycle, credential, configuration,
//! and workspace authorization boundaries before runtime processes exist.

pub mod benchmark;
pub mod channel_commands;
pub mod checkpoint_commands;
pub mod checkpoints;
pub mod config;
pub mod coordination_commands;
pub mod credentials;
pub mod device_state;
pub mod diagnostics_commands;
pub mod draft_commands;
pub mod events;
pub mod execution_environment;
pub mod git_commands;
pub mod git_service;
pub mod ingestion;
pub mod interaction_commands;
pub mod internal_mcp;
pub mod models;
pub mod preview;
pub mod process;
pub mod provider_files;
pub mod providers;
pub mod repository;
pub mod resource_commands;
pub mod restore_commands;
pub mod review_commands;
pub mod review_engine;
pub mod runtime;
pub mod send_commands;
pub mod settings_commands;
pub mod source_control;
pub mod state;
pub mod terminal;
pub mod terminal_commands;
pub mod thread_commands;
pub mod workspace;
pub mod workspace_commands;
pub mod workspace_files;
pub mod workspace_mutation;
pub mod workspace_observer;

#[cfg(test)]
mod tests;

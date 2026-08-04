//! Local coding-agent Chat contracts.
//!
//! The module freezes provider, event, lifecycle, credential, configuration,
//! and workspace authorization boundaries before runtime processes exist.

pub(crate) mod agent_runs;
pub mod benchmark;
pub mod channel_commands;
pub mod checkpoint_commands;
pub mod checkpoints;
pub(crate) mod command_support;
pub(crate) mod composer;
pub mod config;
pub(crate) mod coordination;
pub mod coordination_commands;
pub mod credentials;
pub mod device_state;
pub mod diagnostics_commands;
pub mod draft_commands;
pub(crate) mod driver_operations;
pub mod events;
pub mod execution_environment;
pub mod git_commands;
pub mod git_service;
pub mod ingestion;
pub(crate) mod interaction;
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
pub(crate) mod send;
pub mod send_commands;
pub(crate) mod settings;
pub mod settings_commands;
pub mod source_control;
pub mod state;
pub mod terminal;
pub mod terminal_commands;
pub mod thread_commands;
pub(crate) mod turns;
pub mod workspace;
pub mod workspace_commands;
pub mod workspace_files;
pub mod workspace_mutation;
pub mod workspace_observer;

#[cfg(test)]
mod tests;

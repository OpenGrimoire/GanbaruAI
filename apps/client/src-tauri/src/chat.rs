//! Local coding-agent Chat contracts.
//!
//! The module freezes provider, event, lifecycle, credential, configuration,
//! and workspace authorization boundaries before runtime processes exist.

pub mod config;
pub mod credentials;
pub mod device_state;
pub mod events;
pub mod models;
pub mod providers;
pub mod repository;
pub mod state;
pub mod workspace;
pub mod workspace_commands;

#[cfg(test)]
mod tests;

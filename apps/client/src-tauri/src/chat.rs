//! Local coding-agent Chat contracts.
//!
//! Phase 1 intentionally exposes no Tauri commands. The module freezes the
//! provider, event, and lifecycle boundaries before runtime processes exist.

pub mod events;
pub mod models;
pub mod providers;
pub mod state;

#[cfg(test)]
mod tests;

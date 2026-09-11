//! Provider-free Chat commands and configuration shared by mobile clients.

#[path = "chat/channel_commands.rs"]
pub mod channel_commands;
pub(crate) use ganbaru_chat::chat::coordination;
#[path = "chat/coordination_commands.rs"]
pub mod coordination_commands;

#[path = "chat/config.rs"]
pub mod config;
#[path = "chat/device_state.rs"]
pub mod device_state;
#[path = "chat/models.rs"]
pub mod models;
#[path = "chat/settings_commands_mobile.rs"]
pub mod settings_commands;

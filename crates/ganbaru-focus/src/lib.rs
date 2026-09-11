//! Durable focus history, validation, and recovery independent of Tauri and the WebView.
//!
//! Callers supply an authorized local SQLite pool. Notification schedules are reminders,
//! never evidence of an accepted run or authority to start a later focus interval.

pub mod admission;
mod pomodoro;
pub use pomodoro::*;

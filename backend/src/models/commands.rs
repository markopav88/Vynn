use serde::{Deserialize, Serialize};

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct Command {
    pub command_id: i32,
    pub command_name: String,
    pub command_description: String,
    pub default_keybinding: String,
}

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct UserKeybinding {
    pub user_id: i32,
    pub command_id: i32,
    pub keybinding: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateKeybindingPayload {
    pub keybinding: String,
} 
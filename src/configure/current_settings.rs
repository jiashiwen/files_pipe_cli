use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CurrentSettings {
    pub current_server_id: String,
}

impl Default for CurrentSettings {
    fn default() -> Self {
        Self {
            current_server_id: "".to_string(),
        }
    }
}

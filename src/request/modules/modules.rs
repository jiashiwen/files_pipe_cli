use serde::{Deserialize, Serialize};

use super::moduile_task::Task;

#[derive(Debug, Deserialize, Serialize)]
pub struct RespListTask {
    pub cf_id: String,
    pub task: Task,
}

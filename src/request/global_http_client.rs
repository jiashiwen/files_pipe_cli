use anyhow::Result;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::runtime::{self, Runtime};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub struct TaskServer {
    pub name: String,
    pub url: String,
}

impl Default for TaskServer {
    fn default() -> Self {
        Self {
            name: "default_name".to_string(),
            url: "http://127.0.0.1:3000".to_string(),
        }
    }
}

pub static GLOBAL_RUNTIME: Lazy<Arc<Runtime>> = Lazy::new(|| {
    let runtime = match init_task_runtime() {
        Ok(db) => db,
        Err(err) => panic!("{}", err),
    };
    Arc::new(runtime)
});

pub static GLOBAL_HTTP_CLIENT: Lazy<Arc<Client>> = Lazy::new(|| {
    let client = reqwest::Client::new();
    Arc::new(client)
});

fn init_task_runtime() -> Result<Runtime> {
    let rt = runtime::Builder::new_multi_thread()
        .worker_threads(num_cpus::get())
        .enable_all()
        .max_io_events_per_tick(32)
        .build()?;
    Ok(rt)
}

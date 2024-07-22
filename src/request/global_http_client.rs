use anyhow::Result;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use tokio::runtime::{self, Runtime};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub struct TaskServer {
    pub name: String,
    pub url: String,
}

impl TaskServer {
    pub fn set_default(&self) -> Self {
        Self::default()
    }
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }
    pub fn set_url(&mut self, url: &str) {
        self.url = url.to_string();
    }
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
    // let client = reqwest::Client::new();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
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

use super::TaskServer;
use crate::{
    configure::CurrentSettings,
    resources::{get_current_settings, get_task_server_from_cf, save_current_settings},
};
use anyhow::Result;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::{runtime, sync::RwLock};

pub static GLOBAL_CURRENT_SERVER: Lazy<Arc<RwLock<TaskServer>>> = Lazy::new(|| {
    let task_server = match get_current_settings() {
        Ok(c) => match get_task_server_from_cf(&c.current_server_id) {
            Ok(t) => t,
            Err(_) => TaskServer::default(),
        },
        Err(_) => TaskServer::default(),
    };

    let t_s = Arc::new(RwLock::new(task_server));
    t_s
});

pub async fn set_current_server(server_id: &str) -> Result<String> {
    let task_server = get_task_server_from_cf(server_id)?;
    let mut current_server = GLOBAL_CURRENT_SERVER.write().await;
    *current_server = task_server.clone();
    let mut current_settings = match get_current_settings() {
        Ok(c) => c,
        Err(_) => CurrentSettings::default(),
    };

    current_settings.current_server_id = server_id.to_string();
    let _ = save_current_settings(&current_settings)?;

    Ok(task_server.url)
}

pub fn test_reqwest() {
    let rt = runtime::Builder::new_multi_thread()
        .worker_threads(num_cpus::get())
        .enable_all()
        .max_io_events_per_tick(32)
        .build()
        .unwrap();

    let async_req = async {
        let resp = reqwest::get("https://www.baidu.com").await.unwrap();
        println!("{resp:#?}");
    };

    rt.block_on(async_req);
}

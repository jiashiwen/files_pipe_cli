use super::{ReqTaskId, RespListTask, Task, TaskServer, GLOBAL_HTTP_CLIENT};
use crate::{
    configure::CurrentSettings,
    request::modules::Response,
    resources::{get_current_settings, get_task_server_from_cf, save_current_settings},
};
use anyhow::Result;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::{runtime, sync::RwLock};

pub const API_TASK_ALL: &'static str = "/api/v1/task/all";
pub const API_TASK_SHOW: &'static str = "/api/v1/task/show";
pub const API_TASK_CREATE: &'static str = "/api/v1/task/create";
pub const API_TASK_UPDATE: &'static str = "/api/v1/task/update";
pub const API_TASK_REMOVE: &'static str = "/api/v1/task/remove";
pub const API_TASK_CLEAN: &'static str = "/api/v1/task/clean";
pub const API_TASK_START: &'static str = "/api/v1/task/start";
pub const API_TASK_STOP: &'static str = "/api/v1/task/stop";
pub const API_TASK_CHECKPOINT: &'static str = "#/api/v1/task/checkpoint";
pub const API_TASK_STATUS: &'static str = "/api/v1/task/status";
pub const API_TASK_ANALYZE: &'static str = "/api/v1/task/analyze";
pub const API_TASK_ALL_LIVING: &'static str = "/api/v1/task/all_living";
pub const API_TEMPLATE_TRANSFER_OSS2OSS: &'static str = "/template/transfer/oss2oss";
pub const API_TEMPLATE_TRANSFER_LOCAL2OSS: &'static str = "/template/transfer/local2oss";
pub const API_TEMPLATE_TRANSFER_OSS2LOCAL: &'static str = "/template/transfer/oss2local";
pub const API_TEMPLATE_TRANSFER_LOCAL2LOCAL: &'static str = "/template/transfer/local2local";

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

pub async fn list_all_tasks() -> Result<Response<Vec<RespListTask>>> {
    let mut url = GLOBAL_CURRENT_SERVER
        .read()
        .await
        .url
        .parse::<reqwest::Url>()?;
    url.set_path(API_TASK_ALL);

    let resp = GLOBAL_HTTP_CLIENT
        .post(url)
        .send()
        .await?
        .json::<Response<Vec<RespListTask>>>()
        .await?;
    Ok(resp)
}

pub async fn task_show(id: &ReqTaskId) -> Result<Response<Task>> {
    let mut url = GLOBAL_CURRENT_SERVER
        .read()
        .await
        .url
        .parse::<reqwest::Url>()?;
    url.set_path(API_TASK_SHOW);
    let client = reqwest::Client::new();
    let resp = client
        .post(url)
        .json(id)
        .send()
        .await?
        .json::<Response<Task>>()
        .await?;
    Ok(resp)
}

pub fn template_transfer_oss2oss() {}

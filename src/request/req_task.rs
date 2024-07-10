use super::{ReqTaskId, RespListTask, Task, TaskStatus, GLOBAL_CURRENT_SERVER, GLOBAL_HTTP_CLIENT};
use crate::request::modules::Response;
use anyhow::Result;

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

pub async fn task_show(id: &ReqTaskId) -> Result<Response<Task>> {
    let mut url = GLOBAL_CURRENT_SERVER
        .read()
        .await
        .url
        .parse::<reqwest::Url>()?;
    url.set_path(API_TASK_SHOW);

    let resp = GLOBAL_HTTP_CLIENT
        .post(url)
        .json(id)
        .send()
        .await?
        .json::<Response<Task>>()
        .await?;
    Ok(resp)
}

pub async fn task_status(id: &ReqTaskId) -> Result<Response<TaskStatus>> {
    let mut url = GLOBAL_CURRENT_SERVER
        .read()
        .await
        .url
        .parse::<reqwest::Url>()?;
    url.set_path(API_TASK_STATUS);

    let resp = GLOBAL_HTTP_CLIENT
        .post(url)
        .json(id)
        .send()
        .await?
        .json::<Response<TaskStatus>>()
        .await?;
    Ok(resp)
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

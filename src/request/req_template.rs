use anyhow::Result;

use super::{Response, Task, GLOBAL_CURRENT_SERVER, GLOBAL_HTTP_CLIENT};

pub const API_TEMPLATE_TRANSFER_OSS2OSS: &'static str = "/api/v1/task/template/transfer/oss2oss";
pub const API_TEMPLATE_TRANSFER_LOCAL2OSS: &'static str =
    "/api/v1/task/template/transfer/local2oss";
pub const API_TEMPLATE_TRANSFER_OSS2LOCAL: &'static str =
    "/api/v1/task/template/transfer/oss2local";
pub const API_TEMPLATE_TRANSFER_LOCAL2LOCAL: &'static str =
    "/api/v1/task/template/transfer/local2local";

pub async fn template_transfer_oss2oss() -> Result<Response<Task>> {
    let mut url = GLOBAL_CURRENT_SERVER
        .read()
        .await
        .url
        .parse::<reqwest::Url>()?;
    url.set_path(API_TEMPLATE_TRANSFER_OSS2OSS);

    let resp = GLOBAL_HTTP_CLIENT
        .get(url)
        .send()
        .await?
        .json::<Response<Task>>()
        .await?;
    Ok(resp)
}

pub async fn template_transfer_local2oss() -> Result<Response<Task>> {
    let mut url = GLOBAL_CURRENT_SERVER
        .read()
        .await
        .url
        .parse::<reqwest::Url>()?;
    url.set_path(API_TEMPLATE_TRANSFER_LOCAL2OSS);

    let resp = GLOBAL_HTTP_CLIENT
        .get(url)
        .send()
        .await?
        .json::<Response<Task>>()
        .await?;
    Ok(resp)
}

pub async fn template_transfer_oss2local() -> Result<Response<Task>> {
    let mut url = GLOBAL_CURRENT_SERVER
        .read()
        .await
        .url
        .parse::<reqwest::Url>()?;
    url.set_path(API_TEMPLATE_TRANSFER_OSS2LOCAL);

    let resp = GLOBAL_HTTP_CLIENT
        .get(url)
        .send()
        .await?
        .json::<Response<Task>>()
        .await?;
    Ok(resp)
}

pub async fn template_transfer_local2local() -> Result<Response<Task>> {
    let mut url = GLOBAL_CURRENT_SERVER
        .read()
        .await
        .url
        .parse::<reqwest::Url>()?;
    url.set_path(API_TEMPLATE_TRANSFER_LOCAL2LOCAL);

    let resp = GLOBAL_HTTP_CLIENT
        .get(url)
        .send()
        .await?
        .json::<Response<Task>>()
        .await?;
    Ok(resp)
}

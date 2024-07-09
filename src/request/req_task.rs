use super::RespListTask;
use crate::request::modules::Response;
use anyhow::{Ok, Result};
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tokio::runtime;

pub const API_TASK_ALL: &'static str = "/api/v1/task/all";

pub static GLOBAL_CURRENT_SERVER: Lazy<Arc<RwLock<String>>> = Lazy::new(|| {
    let lock_str = Arc::new(RwLock::new("".to_string()));
    lock_str
});

pub fn test_reqwest() {
    let rt = runtime::Builder::new_multi_thread()
        .worker_threads(num_cpus::get())
        .enable_all()
        .max_io_events_per_tick(32)
        .build()
        .unwrap();
    // rt.block_on(async_http_server);

    let async_req = async {
        // let resp = reqwest::get("https://httpbin.org/ip")
        //     .await
        //     .unwrap()
        //     .json::<HashMap<String, String>>()
        //     .await
        //     .unwrap();

        let resp = reqwest::get("https://www.baidu.com")
            .await
            // .unwrap()
            // .json::<HashMap<String, String>>()
            // .await
            .unwrap();
        println!("{resp:#?}");
    };

    rt.block_on(async_req);
}

pub async fn list_all_tasks() -> Result<Response<Vec<RespListTask>>> {
    let client = reqwest::Client::new();
    let resp = client
        .post("http://114.67.87.223:3000/api/v1/task/all")
        .send()
        .await?
        .json::<Response<Vec<RespListTask>>>()
        .await?;
    Ok(resp)
}

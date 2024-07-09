// use hyper::client::connect::HttpConnector;
// use hyper::client::conn::http1
// use hyper::Body;
// use hyper::Client;
// use hyper_tls::HttpsConnector;
// use lazy_static::lazy_static;

// pub struct HttpClient {
//     pub http: hyper::Client<HttpConnector, Body>,
//     pub https: hyper::Client<HttpsConnector<HttpConnector>, Body>,
// }

// impl HttpClients {
//     pub fn default() -> Self {
//         let client = Client::new();
//         let https = HttpsConnector::new();
//         let client_https = Client::builder().build::<_, hyper::Body>(https);
//         Self {
//             http: client,
//             https: client_https,
//         }
//     }
// }

// lazy_static! {
//     pub static ref GLOBAL_HTTP_CLIENT: HttpClients = HttpClients::default();
// }

use anyhow::Result;
use std::sync::Arc;

use once_cell::sync::Lazy;
use tokio::runtime::{self, Runtime};

pub static GLOBAL_RUNTIME: Lazy<Arc<Runtime>> = Lazy::new(|| {
    let runtime = match init_task_runtime() {
        Ok(db) => db,
        Err(err) => panic!("{}", err),
    };
    Arc::new(runtime)
});

fn init_task_runtime() -> Result<Runtime> {
    let rt = runtime::Builder::new_multi_thread()
        .worker_threads(num_cpus::get())
        .enable_all()
        .max_io_events_per_tick(32)
        .build()?;
    Ok(rt)
}

use crate::configure::CurrentSettings;
use crate::request::TaskServer;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use once_cell::sync::Lazy;
use rocksdb::IteratorMode;
use rocksdb::{DBWithThreadMode, MultiThreaded, Options};
use snowflake::SnowflakeIdGenerator;
use std::sync::Arc;
use url::Url;

pub const CF_SERVERS: &'static str = "cf_servers";
pub const CF_CURRENT_SETTITNGS: &'static str = "cf_current_settings";

pub const CURRENT_SETTITNGS_KEY: &'static str = "current_settings";

pub static GLOBAL_ROCKSDB: Lazy<Arc<DBWithThreadMode<MultiThreaded>>> = Lazy::new(|| {
    let rocksdb = match init_rocksdb("files_pipe_cli_rocksdb") {
        Ok(db) => db,
        Err(err) => panic!("{}", err),
    };
    Arc::new(rocksdb)
});

pub fn init_rocksdb(db_path: &str) -> Result<DBWithThreadMode<MultiThreaded>> {
    let mut cf_opts = Options::default();
    cf_opts.set_allow_concurrent_memtable_write(true);
    cf_opts.set_max_write_buffer_number(16);
    cf_opts.set_write_buffer_size(128 * 1024 * 1024);
    cf_opts.set_disable_auto_compactions(true);

    let mut db_opts = Options::default();
    // db_opts.set_disable_auto_compactions(true);
    db_opts.create_missing_column_families(true);
    db_opts.create_if_missing(true);

    let db = DBWithThreadMode::<MultiThreaded>::open_cf_with_opts(
        &db_opts,
        db_path,
        vec![
            (CF_SERVERS, cf_opts.clone()),
            (CF_CURRENT_SETTITNGS, cf_opts.clone()),
        ],
    )?;
    Ok(db)
}

pub fn save_current_settings(current_settings: &CurrentSettings) -> Result<()> {
    let cf = match GLOBAL_ROCKSDB.cf_handle(CF_SERVERS) {
        Some(cf) => cf,
        None => return Err(anyhow!("column family not exist")),
    };

    let encoded = bincode::serialize(current_settings)?;
    let _ = GLOBAL_ROCKSDB.put_cf(&cf, CURRENT_SETTITNGS_KEY, encoded)?;
    Ok(())
}
pub fn get_current_settings() -> Result<CurrentSettings> {
    let cf = match GLOBAL_ROCKSDB.cf_handle(CF_SERVERS) {
        Some(cf) => cf,
        None => return Err(anyhow!("column family not exist")),
    };

    let current_settings_bytes = match GLOBAL_ROCKSDB.get_cf(&cf, CURRENT_SETTITNGS_KEY)? {
        Some(v) => v,
        None => return Err(anyhow!("server not exists")),
    };
    let current_settings = bincode::deserialize::<CurrentSettings>(&current_settings_bytes)?;
    Ok(current_settings)
}

pub fn save_task_server_to_cf(task_server: &TaskServer) -> Result<i64> {
    let cf = match GLOBAL_ROCKSDB.cf_handle(CF_SERVERS) {
        Some(cf) => cf,
        None => return Err(anyhow!("column family not exist")),
    };

    if task_server_exist(task_server).context(format!("{}:{}", file!(), line!()))? {
        return Err(anyhow!("server is already saved"));
    }

    let mut id_generator = SnowflakeIdGenerator::new(1, 1);
    let id = id_generator.real_time_generate();
    let encoded: Vec<u8> =
        bincode::serialize(task_server).context(format!("{}:{}", file!(), line!()))?;
    GLOBAL_ROCKSDB
        .put_cf(&cf, id.to_string().as_bytes(), encoded)
        .context(format!("{}:{}", file!(), line!()))?;
    Ok(id)
}

pub fn task_server_exist(task_server: &TaskServer) -> Result<bool> {
    let mut saved = false;
    let cf = match GLOBAL_ROCKSDB.cf_handle(CF_SERVERS) {
        Some(cf) => cf,
        None => return Err(anyhow!("column family not exist")),
    };

    let url = Url::parse(task_server.url.as_str())?;
    let sever_cf_iter = GLOBAL_ROCKSDB.iterator_cf(&cf, IteratorMode::Start);

    for item in sever_cf_iter {
        if let Ok(kv) = item {
            let task_server = bincode::deserialize::<TaskServer>(&kv.1).context(format!(
                "{}:{}",
                file!(),
                line!()
            ))?;
            let saved_url =
                Url::parse(&task_server.url).context(format!("{}:{}", file!(), line!()))?;

            if url.scheme().eq(saved_url.scheme())
                && url.host().eq(&saved_url.host())
                && url.port().eq(&saved_url.port())
            {
                saved = true;
                break;
            }
        }
    }

    Ok(saved)
}

pub fn get_task_server_from_cf(server_id: &str) -> Result<TaskServer> {
    let cf = match GLOBAL_ROCKSDB.cf_handle(CF_SERVERS) {
        Some(cf) => cf,
        None => return Err(anyhow!("column family not exist")),
    };

    let task_server_bytes = match GLOBAL_ROCKSDB.get_cf(&cf, server_id)? {
        Some(v) => v,
        None => return Err(anyhow!("server not exists")),
    };
    let task_server = bincode::deserialize::<TaskServer>(&task_server_bytes)?;
    Ok(task_server)
}

pub fn remove_server_from_cf(server_id: &str) -> Result<()> {
    let cf = match GLOBAL_ROCKSDB.cf_handle(CF_SERVERS) {
        Some(cf) => cf,
        None => return Err(anyhow!("column family not exist")),
    };
    GLOBAL_ROCKSDB.delete_cf(&cf, server_id)?;
    Ok(())
}

pub fn list_servers_from_cf() -> Result<Vec<(String, TaskServer)>> {
    let mut vec_servers = vec![];
    let cf = match GLOBAL_ROCKSDB.cf_handle(CF_SERVERS) {
        Some(cf) => cf,
        None => return Err(anyhow!("column family not exist")),
    };

    let sever_cf_iter = GLOBAL_ROCKSDB.iterator_cf(&cf, IteratorMode::Start);

    for item in sever_cf_iter {
        if let Ok(kv) = item {
            let key = String::from_utf8(kv.0.to_vec())?;
            let task_server = bincode::deserialize::<TaskServer>(&kv.1)?;

            vec_servers.push((key, task_server))
        }
    }

    Ok(vec_servers)
}

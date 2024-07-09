use anyhow::anyhow;
use anyhow::Result;
use once_cell::sync::Lazy;
use rocksdb::IteratorMode;
use rocksdb::{DBWithThreadMode, MultiThreaded, Options};
use snowflake::SnowflakeIdGenerator;
use std::sync::Arc;
use url::Url;

pub const CF_SERVERS: &'static str = "cf_servers";

pub static GLOBAL_ROCKSDB: Lazy<Arc<DBWithThreadMode<MultiThreaded>>> = Lazy::new(|| {
    let rocksdb = match init_rocksdb("oss_pipe_rocksdb") {
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
        vec![(CF_SERVERS, cf_opts)],
    )?;
    Ok(db)
}

pub fn save_server_to_cf(server: &str) -> Result<()> {
    let cf = match GLOBAL_ROCKSDB.cf_handle(CF_SERVERS) {
        Some(cf) => cf,
        None => return Err(anyhow!("column family not exist")),
    };

    if server_is_saved(server)? {
        return Err(anyhow!("server is already saved"));
    }
    let mut id_generator = SnowflakeIdGenerator::new(1, 1);
    let id = id_generator.real_time_generate();

    GLOBAL_ROCKSDB.put_cf(
        &cf,
        id.to_string().as_bytes(),
        server.to_string().as_bytes(),
    )?;
    Ok(())
}

pub fn server_is_saved(server: &str) -> Result<bool> {
    let mut saved = false;
    let cf = match GLOBAL_ROCKSDB.cf_handle(CF_SERVERS) {
        Some(cf) => cf,
        None => return Err(anyhow!("column family not exist")),
    };

    let url = Url::parse(server)?;
    let sever_cf_iter = GLOBAL_ROCKSDB.iterator_cf(&cf, IteratorMode::Start);

    for item in sever_cf_iter {
        if let Ok(kv) = item {
            let url_str = String::from_utf8(kv.1.to_vec())?;
            let saved_url = Url::parse(&url_str)?;

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

pub fn get_server_url_str_from_cf(server_id: &str) -> Result<String> {
    let cf = match GLOBAL_ROCKSDB.cf_handle(CF_SERVERS) {
        Some(cf) => cf,
        None => return Err(anyhow!("column family not exist")),
    };

    let server = match GLOBAL_ROCKSDB.get_cf(&cf, server_id)? {
        Some(v) => v,
        None => return Err(anyhow!("server not exists")),
    };
    let server_str = String::from_utf8(server)?;
    Ok(server_str)
}

pub fn remove_server_from_cf(server_id: &str) -> Result<()> {
    let cf = match GLOBAL_ROCKSDB.cf_handle(CF_SERVERS) {
        Some(cf) => cf,
        None => return Err(anyhow!("column family not exist")),
    };
    GLOBAL_ROCKSDB.delete_cf(&cf, server_id)?;
    Ok(())
}

pub fn list_servers_from_cf() -> Result<Vec<(String, String)>> {
    let mut vec_servers = vec![];
    let cf = match GLOBAL_ROCKSDB.cf_handle(CF_SERVERS) {
        Some(cf) => cf,
        None => return Err(anyhow!("column family not exist")),
    };

    let sever_cf_iter = GLOBAL_ROCKSDB.iterator_cf(&cf, IteratorMode::Start);

    for item in sever_cf_iter {
        if let Ok(kv) = item {
            let key = String::from_utf8(kv.0.to_vec())?;
            let url_str = String::from_utf8(kv.1.to_vec())?;

            vec_servers.push((key, url_str))
        }
    }

    Ok(vec_servers)
}

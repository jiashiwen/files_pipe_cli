use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

pub static GLOBAL_CURRENT_SERVER: Lazy<Arc<RwLock<String>>> = Lazy::new(|| {
    let lock_str = Arc::new(RwLock::new("".to_string()));
    lock_str
});

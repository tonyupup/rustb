use std::{
    collections::HashMap,
    sync::RwLock,
    path::Path,
};

use config::{Config, File, Value};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref GLOBAL_CONFIG: RwLock<Config> = {
        let mut conf = Config::default();
        conf.merge(File::from(Path::new("src/conf/config.yaml")))
            .unwrap();
        RwLock::new(conf)
    };
}
pub fn get_config(section: &str) -> Option<HashMap<String, Value>> {
    return match GLOBAL_CONFIG.read().unwrap().get_table(section) {
        Ok(t) => Some(t),
        Err(_) => None,
    };
}

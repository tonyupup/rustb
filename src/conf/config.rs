use serde::{self, Deserialize, Serialize};
use std::{collections::HashMap, net, path::Path, sync::RwLock, thread::sleep, time};

use config::{Config, File, Value};
use lazy_static::lazy_static;

use crate::dnspod::{self, RecordType};

pub static CONFIG_PATH: &'static str = "src/conf/config.yaml";
lazy_static! {
    pub static ref GLOBAL_CONFIG: RwLock<Config> = {
        let mut conf = Config::default();
        conf.merge(File::from(Path::new(CONFIG_PATH))).unwrap();
        RwLock::new(conf)
    };
}
pub fn get_config(section: &str) -> Option<HashMap<String, Value>> {
    return match GLOBAL_CONFIG.read().unwrap().get_table(section) {
        Ok(t) => Some(t),
        Err(_) => None,
    };
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct host_config {
    pub host: Option<String>,
    #[serde(default = "default_rtype")]
    pub rtype: Option<dnspod::RecordType>,
}

fn default_rtype() -> Option<dnspod::RecordType> {
    Some(RecordType::AAAA)
}

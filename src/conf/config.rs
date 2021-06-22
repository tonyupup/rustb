use std::{
    collections::HashMap,
    rc::Rc,
    sync::{Mutex, RwLock},
};
#[macro_use]
use std::{
    cell::{Ref, RefCell},
    path::Path,
};
use config::{Config, File, Value};
use lazy_static::lazy_static;
use serde_json::Deserializer;

lazy_static! {
    pub static ref GLOBAL_CONFIG: RwLock<Config> = {
        let mut conf = Config::default();
        conf.merge(File::from(Path::new("src/conf/config.yaml")))
            .unwrap();
        RwLock::new(conf)
    };
}
pub fn get_config(Section: &str) -> Option<HashMap<String, Value>> {
    return match GLOBAL_CONFIG.read().unwrap().get_table(Section) {
        Ok(t) => Some(t),
        Err(_) => None,
    };
}

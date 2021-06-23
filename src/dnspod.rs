use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    convert::TryInto,
    io::{stdout, Read, Write},
    iter::FromIterator,
    ops::{Add, IndexMut},
    sync::RwLock,
    time, usize,
};

use crate::{arp::DhcpV4Record, conf::config::get_config};
use curl::{easy::Easy, Error};
use serde::{self, Deserialize, Serialize, __private::de::IdentifierDeserializer};

use serde_json::Deserializer;

#[derive(Debug, Default, Clone)]
struct DnspodConfig {
    token: String,
    domain: String,
    domain_id: u32,
}

impl DnspodConfig {
    pub fn get() -> Self {
        let dnsconf = get_config("dnspod").unwrap();

        return Self {
            token: dnsconf.get("token").unwrap().to_string(),
            domain: dnsconf.get("domain").unwrap().to_string(),
            domain_id: dnsconf
                .get("domain_id")
                .unwrap()
                .clone()
                .into_int()
                .unwrap_or(0) as u32,
        };
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct DnsPodglobalBody {
    login_token: String,
    format: &'static str,
    domain: String,
}

impl Default for DnsPodglobalBody {
    fn default() -> Self {
        DnsPodglobalBody {
            login_token: DnspodConfig::get().token,
            format: "json",
            domain: DnspodConfig::get().domain,
        }
    }
}
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Hash)]
pub enum record_type {
    A,
    AAAA,
    Text,
    NS,
    CNAME,
    TXT,
}
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Hash)]
pub struct Record {
    pub id: String,
    pub name: String,
    pub line: String,
    pub line_id: String,

    #[serde(rename = "type")]
    pub rtype: record_type,
    pub ttl: String,
    pub value: String,
    pub weight: Option<String>,
    pub mx: String,
    pub enabled: String,
    pub status: String,
    pub monitor_status: String,
    pub remark: String,
    pub updated_on: String,
    pub use_aqb: String,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct record_statue {
    pub code: String,
    pub message: String,
    pub created_at: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct record_domain {
    pub id: String,
    pub name: String,
    pub punycode: String,
    pub grade: String,
    pub owner: String,
    pub ext_status: String,
    pub ttl: u8,
    pub min_ttl: u8,
    pub dnspod_ns: Vec<String>,
    pub status: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RecordResp {
    pub status: record_statue,
    pub domain: record_domain,
    pub records: Vec<Record>,
}
pub struct DnsPod {
    lastupdate: RefCell<time::SystemTime>,
    dnsrecord: RefCell<HashMap<String, Record>>,
}
impl DnsPod {
    pub fn new() -> Self {
        let result = Self::get_record_list();

        return Self {
            lastupdate: RefCell::new(result.0),
            dnsrecord: RefCell::new(result.1),
        };
    }

    fn get_record_list() -> (time::SystemTime, HashMap<String, Record>) {
        // client.post()
        let req_body = serde_urlencoded::to_string(&DnsPodglobalBody::default()).unwrap();
        // let req_body = serde_json::to_string(&DnsPodglobalBody::default()).unwrap();
        let resp =
            http_request("https://dnsapi.cn/Record.List", "POST", req_body.as_bytes()).unwrap();
        // println!("{}",String::from_utf8(resp.clone()).unwrap());
        let mut result = serde_json::from_slice::<RecordResp>(&resp[..]).unwrap();

        let mut hn = HashMap::new();

        let mut count = result.records.len();
        while count > 0 {
            let recrod = result.records.pop().unwrap();
            hn.insert(recrod.name.clone(), recrod);
            count -= 1;
        }

        (time::SystemTime::now(), hn)
    }
    pub fn update(&self, r: &Record) -> Result<(), Error> {
        Ok(())
    }
    pub fn delete(&self, r: &Record) -> Result<(), Error> {
        Ok(())
    }
    pub fn add(&self, r: &Record) -> Result<(), Error> {
        Ok(())
    }

    fn lazy_update(&self, timeout: time::Duration) -> Result<(), Error> {
        if self.lastupdate.borrow().add(timeout) < time::SystemTime::now() {
            println!("update");
            let new_record = Self::get_record_list();
            *self.lastupdate.borrow_mut() = new_record.0;
            *self.dnsrecord.borrow_mut() = new_record.1;
        }
        Ok(())
    }
    pub fn handle(&self, _t: DhcpV4Record) -> Result<(), Error> {
        // self.lastRecord.borrow().
        self.lazy_update(time::Duration::from_secs(5));
        Ok(())
    }
}

fn http_request<'a>(url: &'a str, method: &'a str, body: &[u8]) -> Result<Vec<u8>, Error> {
    let mut rawresp = Vec::new();
    let mut easy = Easy::new();
    easy.url(url).unwrap();

    if method == "POST" {
        easy.post(true).unwrap();
        easy.post_field_size(body.len() as u64);
    }

    {
        let mut trans = easy.transfer();
        trans
            .write_function(|data| {
                rawresp.extend_from_slice(data);
                Ok(data.len())
            })
            .unwrap();

        trans
            .read_function(|mut buf| Ok(buf.write(body).unwrap_or(0)))
            .unwrap();

        trans.perform().unwrap();
    }
    Ok(rawresp)
}

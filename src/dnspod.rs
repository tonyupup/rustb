use crate::{
    arp::DhcpV4Record,
    conf::{self, config::get_config},
};
use crate::{enum_error, impl_display, impl_error, impl_from};
use conf::config::host_config;
use curl::{easy::Easy, Error as CurlError};
use serde::{self, Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::HashMap,
    error::Error,
    fmt::Display,
    io::{Error as IOError, Write},
    net::{IpAddr, Ipv4Addr},
    num::ParseIntError,
    ops::Add,
    time, usize,
};

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
struct DnsPodGlobalBody {
    login_token: String,
    format: &'static str,
    domain: String,
}
#[derive(Serialize, Debug)]
struct DnsPodUpdateBody {
    #[serde(flatten)]
    pub global: DnsPodGlobalBody,
    pub record_id: u32,
    pub sub_domain: String,
    pub record_type: RecordType,
    pub record_line: String,
    pub value: IpAddr,
    pub max: u8,
}

impl Default for DnsPodUpdateBody {
    fn default() -> Self {
        Self {
            global: DnsPodGlobalBody::default(),
            record_id: 0,
            sub_domain: "you".to_string(),
            record_type: RecordType::default(),
            record_line: "xx".to_string(),
            value: IpAddr::V4(Ipv4Addr::LOCALHOST),
            max: 0,
        }
    }
}

impl Default for DnsPodGlobalBody {
    fn default() -> Self {
        DnsPodGlobalBody {
            login_token: DnspodConfig::get().token,
            format: "json",
            domain: DnspodConfig::get().domain,
        }
    }
}
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Hash, Clone)]
pub enum RecordType {
    A,
    AAAA,
    Text,
    NS,
    CNAME,
    TXT,
}

impl Default for RecordType {
    fn default() -> Self {
        Self::AAAA
    }
}
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Hash)]
pub struct Record {
    pub id: String,
    pub name: String,
    pub line: String,
    pub line_id: String,

    #[serde(rename = "type")]
    pub rtype: RecordType,
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

self::enum_error!(
    DnsPodHandleError,
    DnsPodHandleError::CurlError=>CurlError,
    DnsPodHandleError::IOError=>IOError,
    DnsPodHandleError::ParseIntError=>ParseIntError);

impl DnsPod {
    pub fn new() -> Self {
        let result = Self::get_record_list().unwrap();

        return Self {
            lastupdate: RefCell::new(result.0),
            dnsrecord: RefCell::new(result.1),
        };
    }

    fn get_record_list() -> Result<(time::SystemTime, HashMap<String, Record>), DnsPodHandleError> {
        // client.post()
        let req_body = serde_urlencoded::to_string(&DnsPodGlobalBody::default()).unwrap();
        let resp = http_request("https://dnsapi.cn/Record.List", "POST", req_body.as_bytes())?;
        let mut result = serde_json::from_slice::<RecordResp>(&resp[..]).unwrap();

        let mut hn = HashMap::new();

        let mut count = result.records.len();
        while count > 0 {
            let recrod = result.records.pop().unwrap();
            hn.insert(recrod.name.clone(), recrod);
            count -= 1;
        }

        Ok((time::SystemTime::now(), hn))
    }
    pub fn add_or_update(
        &self,
        r: &DhcpV4Record,
        c: &host_config,
    ) -> Result<(), DnsPodHandleError> {
        if let Some(record) = self
            .dnsrecord
            .borrow()
            .get(c.host.as_ref().unwrap_or(&r.host))
        {
            //update dnspod records
            if let Some(x) = &c.rtype {
                if let RecordType::AAAA = x {
                    let x = DnsPodUpdateBody {
                        global: DnsPodGlobalBody::default(),
                        record_id: record.id.parse()?,
                        sub_domain: c.host.clone().unwrap_or(r.host.clone()),
                        record_line: "??????".to_string(),
                        record_type: x.clone(),
                        value: r.get_global_ipv6().unwrap(),
                        max: 1,
                    };
                    let resp = http_request(
                        "https://dnsapi.cn/Record.Modify",
                        "POST",
                        serde_urlencoded::to_string(x).unwrap().as_bytes(),
                    )?;
                    println!("{:?}", String::from_utf8(resp));
                }
            }
        } else {
            //add record
        }

        Ok(())
    }

    pub fn delete(&self, r: &Record) -> Result<(), DnsPodHandleError> {
        Ok(())
    }
    pub fn add(&self, r: &Record) -> Result<(), DnsPodHandleError> {
        Ok(())
    }

    fn lazy_update(&self, timeout: time::Duration) -> Result<(), DnsPodHandleError> {
        if self.lastupdate.borrow().add(timeout) < time::SystemTime::now() {
            println!("update");
            let new_record = Self::get_record_list()?;
            *self.lastupdate.borrow_mut() = new_record.0;
            *self.dnsrecord.borrow_mut() = new_record.1;
        }
        Ok(())
    }
    pub fn handle(&self, r: DhcpV4Record) -> Result<(), DnsPodHandleError> {
        // self.lastRecord.borrow().
        self.lazy_update(time::Duration::from_secs(5))?;
        if let Some(c) = r.need() {
            self.add_or_update(&r, &c)?;
        }
        Ok(())
    }
}

fn http_request<'a>(url: &'a str, method: &'a str, body: &[u8]) -> Result<Vec<u8>, CurlError> {
    let mut rawresp = Vec::new();
    let mut easy = Easy::new();
    easy.url(url).unwrap();

    if method == "POST" {
        easy.post(true)?;
        easy.post_field_size(body.len() as u64)?;
    }

    {
        let mut trans = easy.transfer();
        trans.write_function(|data| {
            rawresp.extend_from_slice(data);
            Ok(data.len())
        })?;

        trans.read_function(|mut buf| Ok(buf.write(body).unwrap_or(0)))?;

        trans.perform()?;
    }
    Ok(rawresp)
}

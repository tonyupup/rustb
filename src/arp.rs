use std::{
    error::Error,
    fmt::Display,
    net::{AddrParseError, Ipv4Addr, Ipv6Addr},
    num::ParseIntError,
    process,
    str::FromStr,
};

use crate::conf::{self, config::host_config};
use crate::{enum_error, impl_display, impl_error, impl_from};
self::enum_error!(DhcpV4RecordParseError,DhcpV4RecordParseError::AddrParseError=>AddrParseError,DhcpV4RecordParseError::ParseIntError=>ParseIntError);

#[derive(Debug)]
pub struct DhcpV4Record {
    pub time: u32,
    pub mac: String,
    pub ipv4: Ipv4Addr,
    pub host: String,
    pub other: String,
}

impl DhcpV4Record {
    pub(crate) fn get_global_ipv6(&self) -> Option<Ipv6Addr> {
        // let m = process::Command::new("ip")
        //     .args(vec!["-6", "neigh", "show"])
        //     .output()
        //     .unwrap();
        let m = process::Command::new("cat")
            .args(vec!["ip6.cat"])
            .output()
            .unwrap();
        String::from_utf8(m.stdout).map_or_else(
            |_| None,
            |out| {
                for x in out.split_terminator('\n') {
                    let line: Vec<&str> = x.split_terminator(' ').collect();
                    if line.len() >= 5 && line[4] == self.mac {
                        if let Ok(c) = Ipv6Addr::from_str(line[0]) {
                            if !(c.is_loopback()
                                || c.is_multicast()
                                || (c.segments()[0] & 0xffc0) == 0xfe80)
                            {
                                return Some(c);
                            }
                        }
                    }
                }
                None
            },
        )
    }

    pub fn need(&self) -> Option<host_config> {
        if let Some(fc) = conf::config::get_config("client") {
            if let Some(conf) = fc.get(&self.mac[..]) {
                let c: host_config = conf.clone().try_into().unwrap();
                return Some(c);
            }
        }
        None
        // get_config("client").map_or_else(|| false, |x| x.get(&self.mac[..]).is_some())
    }
}

impl FromStr for DhcpV4Record {
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let o: Vec<&str> = s.split_terminator(' ').collect();
        assert_eq!(o.len(), 5, "must size 5");
        Ok(Self {
            time: o[0].parse()?,
            mac: String::from(o[1]),
            ipv4: Ipv4Addr::from_str(o[2])?,
            host: String::from(o[3]),
            other: String::from(o[4]),
        })
    }

    type Err = DhcpV4RecordParseError;
}

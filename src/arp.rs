use std::{
    error::Error,
    io::Error,
    net::{Ipv4Addr, Ipv6Addr},
    process,
    str::FromStr,
};

#[derive(Debug)]
pub struct DhcpRecord {
    pub(crate) time: u32,
    pub(crate) mac: String,
    pub(crate) ipv4: Ipv4Addr,
    pub(crate) host: String,
    pub(crate) other: String,
}

impl DhcpRecord {
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
}

impl From<&str> for DhcpRecord {
    fn from(s: &str) -> Self {
        let o: Vec<&str> = s.split_terminator(' ').collect();
        assert_eq!(o.len(), 5, "must size 5");
        Self {
            time: o[0].parse().unwrap(),
            mac: String::from(o[1]),
            ipv4: Ipv4Addr::from_str(o[2]).unwrap(),
            host: String::from(o[3]),
            other: String::from(o[4]),
        }
    }
}
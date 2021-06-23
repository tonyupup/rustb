use std::{
    error::Error,
    fmt::Display,
    net::{AddrParseError, Ipv4Addr, Ipv6Addr},
    num::ParseIntError,
    process,
    str::FromStr,
};

use conf::config::{get_config, GLOBAL_CONFIG};

use crate::conf;

#[derive(Debug)]
pub struct DhcpV4Record {
    pub(crate) time: u32,
    pub(crate) mac: String,
    pub(crate) ipv4: Ipv4Addr,
    pub(crate) host: String,
    pub(crate) other: String,
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
    
    pub fn need(&self) -> bool {
        get_config("client").map_or_else(|| false, |x| x.get(&self.mac[..]).is_none())
    }

    pub fn gethost(&self) -> bool {
        false
    }
}

#[macro_export]
macro_rules! impl_from {
    ($t:ty, $($y:path=>$x:path),+) =>{
        $(
            impl From<$x> for $t {
                fn from(args:$x) -> Self {
                    $y(args)
                }
            }
        )+
    }
}

#[macro_export]
macro_rules! impl_display {
    ($t:ty,$($y:path=>$x:path),+) => {

        impl Display for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        $y(p)=>p.fmt(f)?,
                    )+
                };
                Ok(())
            }
        }
    };
}

#[macro_export]
macro_rules! impl_error {
    ($t:ty,$($y:path=>$x:path),+) => {
        impl Error for $t {
            fn cause(&self) -> Option<&dyn Error> {
                match *self {
                    $(
                        $y(ref p)=>Some(p),
                    )+
                }
            }
        }
    }
}
#[macro_export]
macro_rules! enum_error {
    ($t:ident,$($fullerr:path=>$err:ident),+) =>{
        #[derive(Debug)]
        pub enum $t {
            $(
                $err($err),
            )+
        }

        impl_display!($t,$($fullerr=>$err),+);
        impl_error!($t,$($fullerr=>$err),+);
        impl_from!($t,$($fullerr=>$err),+);

    }
}

enum_error!(DhcpV4RecordParseError,DhcpV4RecordParseError::AddrParseError=>AddrParseError,DhcpV4RecordParseError::ParseIntError=>ParseIntError);

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

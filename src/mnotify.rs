extern crate notify;

use crate::arp;
use crate::arp::DhcpV4Record;
use crate::conf;
use notify::DebouncedEvent;
use notify::{watcher, RecursiveMode, Watcher};
use std::sync::mpsc;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;
use std::{env, fs, thread};
use DebouncedEvent::{Chmod, Create, NoticeRemove, NoticeWrite, Remove, Rename, Rescan, Write};

pub fn not() -> Receiver<arp::DhcpV4Record> {
    let dhcppfile = env::var("nppath").unwrap_or(String::from("dhcp.lease"));
    // Create a channel to receive the events.
    let (tx, rx) = mpsc::channel();
    let (ttx, rrx) = mpsc::channel();
    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.
    thread::spawn(move || {
        let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();
        // Add a path to be watched. All files and directories at that path and
        // below will be monitored for changes.
        watcher
            .watch(&dhcppfile[..], RecursiveMode::NonRecursive)
            .unwrap();
        //watch config
        watcher
            .watch(conf::config::CONFIG_PATH, RecursiveMode::NonRecursive)
            .unwrap();

        loop {
            if let Ok(e) = rx.recv() {
                match e {
                    NoticeWrite(p) | NoticeRemove(p) | Create(p) | Write(p) | Chmod(p)
                    | Remove(p) => {
                        let fname = p.file_name().unwrap().to_str().unwrap();
                        if conf::config::CONFIG_PATH.ends_with(fname) {
                            if let Ok(mut c) = conf::config::GLOBAL_CONFIG.try_write() {
                                c.refresh().unwrap();
                            }
                        } else {
                            let rs = fs::read_to_string(&dhcppfile[..]).unwrap();
                            for p in rs.split_terminator('\n') {
                                if let Ok(p) = p.parse::<DhcpV4Record>() {
                                    ttx.send(p).unwrap();
                                }
                            }
                        }
                    }
                    _=>()
                }
            }
        }
    });
    return rrx;
}

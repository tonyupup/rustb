extern crate notify;

use notify::DebouncedEvent;
use notify::Error::Generic;
use notify::{watcher, RecursiveMode, Watcher};
use std::str::FromStr;
use std::sync::mpsc;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;
use std::{env, fs, thread};
use DebouncedEvent::Error;
use DebouncedEvent::{Chmod, Create, NoticeRemove, NoticeWrite, Remove, Rename, Rescan, Write};

use crate::arp;
use crate::arp::DhcpV4Record;

pub fn not() -> Receiver<arp::DhcpV4Record> {
    let dhcp_file = env::var("n_path").unwrap_or(String::from("dhcp.lease"));
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
            .watch(&dhcp_file[..], RecursiveMode::Recursive)
            .unwrap();
        loop {
            match rx.recv() {
                Ok(_) => {
                    let rs = fs::read_to_string(&dhcp_file[..]).unwrap();
                    for p in rs.split_terminator('\n') {
                        if let Ok(p) = p.parse::<DhcpV4Record>() {
                            ttx.send(p).unwrap();
                        }
                        
                    }
                }
                Err(e) => {
                    println!("{}", e);
                }
            }
        }
    });
    return rrx;
}

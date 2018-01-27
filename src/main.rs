extern crate notify;

use notify::{Watcher, RecursiveMode, RawEvent, raw_watcher};
use notify::op::WRITE;
use std::sync::mpsc::channel;
use std::path::PathBuf;

fn main() {
    // Create a channel to receive the events.
    let (tx, rx) = channel();

    // Create a watcher object, delivering raw events.
    // The notification back-end is selected based on the platform.
    let mut watcher = raw_watcher(tx).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(".", RecursiveMode::Recursive).unwrap();

    loop {
        match rx.recv() {
            Ok(RawEvent{path: Some(path), op: Ok(WRITE), ..}) => echo_file(path),
            _ => {}
        }
    }
}

fn echo_file(path_buf: PathBuf) {
    println!("WRITE to {:?}", path_buf);
}
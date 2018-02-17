extern crate notify;
extern crate core;

use notify::{Watcher, RecursiveMode, RawEvent, raw_watcher};
use notify::op::WRITE;
use std::sync::mpsc::channel;
use std::path::PathBuf;
use std::env;
use std::path::Path;
use std::fs;

use std::collections::HashMap;
use core::cmp::Ordering;

fn main() {
    let mut file_map = HashMap::new();

    let args: Vec<String> = env::args().collect();
    let path_to_watch =
        if args.len() < 2 { env::current_dir().unwrap() } else { PathBuf::from(args[1].clone()) };

    println!("Tailing files in directory [{:?}]...", path_to_watch);

    let rd = fs::read_dir(&path_to_watch);
    match rd {
        Ok(rd) => read_directory(&mut file_map, rd),
        Err(err) => println!("Dir no good: {:?}", err),
    }

    // start tailing
    tail(&mut file_map, &path_to_watch);

    println!("Tailing ended.");
}

fn tail(file_map: &mut HashMap<String, u64>, path_to_watch: &PathBuf) {
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
            Ok(RawEvent{path: Some(path), op: Ok(WRITE), ..}) => echo_file(file_map, path, path_to_watch),
            _ => {},
        }
    }
}

fn read_directory(file_map: &mut HashMap<String, u64>, rd: fs::ReadDir) {
//    println!("Value = {:?}", rd);
    for entry in rd {
        match entry {
            Ok(de) => process_entry(file_map, de),
            Err(err) => println!("Entry no good: {}", err),
        }
    }
}

fn process_entry(file_map: &mut HashMap<String, u64>, de: fs::DirEntry) {
//    println!("Dir entry = [{:?}]", de);
    let p = de.path();
    let p = p.as_path();
    if p.is_file() {
        process_file(file_map, p);
    }
}

fn process_file(file_map: &mut HashMap<String, u64>, p: &Path) {
    let pb = p.canonicalize().expect("Cannot get full absolute path.");
    let name = String::from(pb.to_str().expect("Cannot get file name from path."));

    let meta = p.metadata().expect("Cannnot get file length from path.");
    let length = meta.len();

    println!("File  = [{}]=[{}]", name, length);

    file_map.insert(name, length);
}

fn echo_file(file_map: &mut HashMap<String, u64>, path_buf: PathBuf, path_to_watch: &PathBuf) {
    let parent_path = path_buf.parent().unwrap().to_path_buf();
//    println!("path_to_watch=[{:?}], parent_path=[{:?}]", path_to_watch, parent_path);
    match path_to_watch.cmp(&parent_path) {
        Ordering::Equal => {
            let pb = path_buf.canonicalize().expect("Cannot get full absolute path.");
            let name = String::from(pb.to_str().expect("Cannot get file name from path."));

            println!("WRITE to {:?} name = {}", path_buf, name);
            match file_map.get("jan") {
                Some(fp) => println!("File pointer for file [{:?}] is [{}].", path_buf, fp),
                None => println!("File [{:?}] is new, echo complete file.", path_buf),
            }
        },
        _ => (),
    }
}
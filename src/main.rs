extern crate notify;
extern crate core;

use notify::{Watcher, RecursiveMode, RawEvent, raw_watcher};
use notify::op::WRITE;
use std::sync::mpsc::channel;
use std::path::PathBuf;
use std::env;
use std::path::Path;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::collections::HashMap;
use core::cmp::Ordering;

///
/// Usage: one or no arguments to specify a directory to watch or use the current directory.
///
/// Does not cope with rewriting files (so their size is reduced).
///
fn main() {
    let mut file_map: HashMap<String, u64> = HashMap::new();

    let path_to_watch: PathBuf = get_path_to_watch();
    let path_to_watch: &Path = path_to_watch.as_path();
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

fn get_path_to_watch() -> PathBuf {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        env::current_dir().unwrap()
    } else {
        let pb = PathBuf::from(&args[1]);
        env::current_dir().unwrap().join(pb)
    }
}

fn tail(file_map: &mut HashMap<String, u64>, path_to_watch: &Path) {
    // Create a channel to receive the events.
    let (tx, rx) = channel();

    // Create a watcher object, delivering raw events.
    // The notification back-end is selected based on the platform.
    let mut watcher = raw_watcher(tx).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path_to_watch, RecursiveMode::Recursive).unwrap();

    loop {
        match rx.recv() {
            Ok(RawEvent{path: Some(file), op: Ok(WRITE), ..}) =>
                echo_file(file_map, file.as_path(), path_to_watch),
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
    let name = String::from(p.to_str().expect("Cannot get file name from path."));
    let meta = p.metadata().expect("Cannnot get file length from path.");
    let length = meta.len();

//    println!("File  = [{}]=[{}]", name, length);

    file_map.insert(name, length);
}

fn echo_file(file_map: &mut HashMap<String, u64>, file: &Path, path_to_watch: &Path) {
    let parent_path = file.parent().unwrap().to_path_buf();
//    println!("path_to_watch=[{:?}], parent_path=[{:?}]", path_to_watch, parent_path);
    match path_to_watch.cmp(&parent_path) {
        Ordering::Equal => {
            let name = String::from(file.to_str().expect("Cannot get file name from path."));

//            println!("WRITE to {:?} name = {}", file, name);
            match file_map.get(&name) {
                Some(fp) => {
                    echo_file_from(&name, *fp);
                },
                None => {
                    echo_whole_file(&name);
                },
            }
            process_file(file_map, file);
        },
        _ => (),
    }
}

fn echo_whole_file(file_name: &String) {
//    println!("File [{:?}] is new, echo complete file.", file_name);
    echo_file_from(file_name, 0);
}

fn echo_file_from(file_name: &String, fp: u64) {
//    println!("File pointer for file [{:?}] is [{}].", file_name, fp);
    let mut file = File::open(file_name).expect("Could not open file for reading.");
    file.seek(SeekFrom::Start(fp)).expect("Could no seek in open file.");
    let mut content = String::new();
    file.read_to_string(&mut content).expect("Could not read file.");
    print!("{}", content);
}

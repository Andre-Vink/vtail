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
use std::io;
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

    match get_path_to_watch() {
        Err(e) => eprintln!("Cannot access current directory. Error: [{}]", e),
        Ok(path_to_watch) => {
            let path_to_watch: &Path = path_to_watch.as_path();
            println!("Tailing files in directory [{:?}]...", path_to_watch);

            match fs::read_dir(&path_to_watch) {
                Ok(rd) => read_directory(&mut file_map, rd),
                Err(err) => eprintln!("Dir no good: {:?}", err),
            }

            // start tailing
            tail(&mut file_map, &path_to_watch);

            println!("Tailing ended.");
        }
    }
}

fn get_path_to_watch() -> io::Result<PathBuf> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        env::current_dir()
    } else {
        let pb = PathBuf::from(&args[1]);
        match env::current_dir() {
            Ok(cd) => Ok(cd.join(pb)),
            Err(x) => Err(x),
        }
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
        if let Ok(RawEvent{path: Some(file), op: Ok(WRITE), ..}) = rx.recv() {
            echo_file(file_map, file.as_path(), path_to_watch);
        }
    }
}

fn read_directory(file_map: &mut HashMap<String, u64>, rd: fs::ReadDir) {
//    println!("Value = {:?}", rd);
    for entry in rd {
        match entry {
            Ok(de) => process_entry(file_map, de),
            Err(e) => eprintln!("Cannot read directory. Error: [{}]", e),
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
    match p.to_str() {
        None => eprintln!("Cannot get file name from path [{:?}].", p),
        Some(s) => {
            let name = String::from(s);
            match p.metadata() {
                Err(e) => eprintln!("Cannnot get the length from path [{:?}]. Error: [{}].", p, e),
                Ok(meta) => {
                    let length = meta.len();
//                    println!("File  = [{}]=[{}]", name, length);
                    file_map.insert(name, length);
                }
            }
        }
    }
}

fn echo_file(file_map: &mut HashMap<String, u64>, file: &Path, path_to_watch: &Path) {
    let parent_path = file.parent().unwrap().to_path_buf();
//    println!("path_to_watch=[{:?}], parent_path=[{:?}]", path_to_watch, parent_path);

    // Don't do files in sub folders
    if let Ordering::Equal = path_to_watch.cmp(&parent_path) {
        match file.to_str() {
            None => eprintln!("Cannot get file name from path [{:?}].", file),
            Some(s) => {
                let name = String::from(s);

//                    println!("WRITE to {:?} name = {}", file, name);
                match file_map.get(&name) {
                    Some(fp) => echo_file_from(&name, *fp),
                    None     => echo_whole_file(&name),
                }
                process_file(file_map, file);
            }
        }
    }
}

fn echo_whole_file(file_name: &String) {
//    println!("File [{:?}] is new, echo complete file.", file_name);
    echo_file_from(file_name, 0);
}

fn echo_file_from(file_name: &String, fp: u64) {
//    println!("File pointer for file [{:?}] is [{}].", file_name, fp);
    let file_result = File::open(file_name);
    match file_result {
        Ok(mut file) => {
            match file.seek(SeekFrom::Start(fp)) {
                Err(e) => eprintln!("Could no seek in open file [{}]. Error: [{}]", file_name, e),
                Ok(_) => {
                    let mut content = String::new();
                    match file.read_to_string(&mut content) {
                        Err(e) => eprintln!("Could not read file [{}]. Error: [{}]", file_name, e),
                        Ok(_) => print!("{}", content),
                    }
                }
            }
        },
        Err(x) => println!(
            "PTAIL ERROR => Could not open file [{}] for reading. Error: [{}].", file_name, x),
    }
}

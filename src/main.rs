extern crate notify;
extern crate core;

use notify::{Watcher, RecursiveMode, DebouncedEvent, watcher};
use std::sync::mpsc::channel;
use std::path::PathBuf;
use std::env;
use std::path::Path;
use std::fs;
use std::fs::File;
//use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::collections::HashMap;
use core::cmp::Ordering;
use std::time::Duration;

///
/// Usage: ptail [-r] [<directory to watch>]
/// -r:                 to watch subdirectories as well,
/// directory to watch: the directory (with optional subdirectories) to watch for changes.
///
/// Does not cope with rewriting files (so their size is reduced).
/// But handles file renaming and removing.
///

#[derive(Debug)]
struct Arguments {
    recursive: bool,
    path_to_watch: PathBuf,
}

fn main() {
    let mut file_map: HashMap<String, u64> = HashMap::new();

    let args: Arguments = parse_arguments();
//    println!("Parsed arguments: {:?}", args);

    let path_to_watch;
    if let Ok(normalized_path) = args.path_to_watch.canonicalize() {
        path_to_watch = normalized_path;
    } else {
        path_to_watch = args.path_to_watch;
    }
    println!("Tailing files in directory [{:?}]...", path_to_watch);

    match fs::read_dir(&path_to_watch) {
        Ok(rd) => read_directory(&mut file_map, rd),
        Err(err) => eprintln!("Dir no good: {:?}", err),
    }

    // start tailing
    tail(&mut file_map, &path_to_watch);

    println!("Tailing ended.");
}

fn parse_arguments() -> Arguments {
    let args: Vec<String> = env::args().collect();
//    println!("Arguments: [{:?}]", args);
    let mut usefull_args = &args[1..];
//    println!("Usefull arguments: [{:?}]", usefull_args);

    let mut recursive = false;
    // Test for recursive (-r) flag
    if usefull_args.len() > 0 {
        recursive = usefull_args[0].eq("-r");
        if recursive {
            usefull_args = &usefull_args[1..];
//            println!("Usefull arguments 2: [{:?}]", usefull_args);
        }
    }

    // Only one path supported at this time
    let cur_dir = env::current_dir().unwrap();
    if usefull_args.len() > 0 {
        let absolute_path = cur_dir.join(&usefull_args[0]);
        Arguments { recursive: recursive, path_to_watch: absolute_path }
    } else {
        Arguments { recursive: recursive, path_to_watch: cur_dir }
    }
}

fn tail(file_map: &mut HashMap<String, u64>, path_to_watch: &Path) {
    // Create a channel to receive the events.
    let (tx, rx) = channel();

    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.
    let mut watcher = watcher(tx, Duration::from_millis(100)).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path_to_watch, RecursiveMode::Recursive).unwrap();

    loop {
        let r = rx.recv();
        if let Ok(event) = r {
            match event {
                DebouncedEvent::Create(file) => echo_file(file_map, file.as_path(), path_to_watch),
                DebouncedEvent::Write(file) => echo_file(file_map, file.as_path(), path_to_watch),
                DebouncedEvent::Remove(path_buf) => println!("Remove[{:?}]", path_buf),
                DebouncedEvent::Rename(path_buf_from, path_buf_to) => println!("Rename[{:?}]->[{:?}]", path_buf_from, path_buf_to),

                DebouncedEvent::NoticeWrite(_) => (),
                DebouncedEvent::NoticeRemove(_) => (),
                DebouncedEvent::Chmod(_) => (),
                DebouncedEvent::Rescan => (),

                DebouncedEvent::Error(error, Some(path_buf)) => eprintln!("Error: [{}] for path [{:?}]", error, path_buf),
                DebouncedEvent::Error(error, None) => eprintln!("Error: [{}]", error),
            }
        } else {
            println!("Event: [{:?}]", r);
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
//    println!("echo_file file=[{:?}], path_to_watch=[{:?}], parent_path=[{:?}]", file, path_to_watch, parent_path);

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
        Err(x) => println!("PTAIL ERROR => Could not open file [{}] for reading. Error: [{}].", file_name, x),
    }
}

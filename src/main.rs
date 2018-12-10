use notify::{Watcher, RecursiveMode, DebouncedEvent, watcher};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::sync::mpsc::channel;
use std::time::Duration;
use core::cmp::Ordering;

mod arguments;
use crate::arguments::Arguments;

///
/// Usage: vtail [-r] [<directory to watch>] [<directory to watch>] ...
/// -r:                 to watch subdirectories as well,
/// directory to watch: the directories (with optional subdirectories) to watch for changes.
///
/// Does not cope with rewriting files (so their size is reduced).
/// But handles file renaming and removing.
///

fn main() {
    let mut file_map: HashMap<String, u64> = HashMap::new();
    let mut paths_to_watch: Vec<PathBuf> = Vec::new();

    let args: Arguments = Arguments::parse_arguments();
//    println!("Parsed arguments: {:?}", args);

    for path in args.get_paths().iter() {
        match path.canonicalize() {
            Ok(normalized_path) => paths_to_watch.push(normalized_path),
            _ => paths_to_watch.push(path.clone()),
        }
    }
    println!("VTAIL: Tailing files in directory [{:?}]...", paths_to_watch);

    for path in paths_to_watch.iter() {
        match fs::read_dir(path) {
            Ok(rd) => read_directory(&mut file_map, rd),
            Err(err) => eprintln!("VTAIL: Cannot read directory [{:?}]: {:?}", path, err),
        }
    }

    // start tailing
    tail(&mut file_map, &paths_to_watch);

    println!("VTAIL: Tailing ended.");
}

fn tail(file_map: &mut HashMap<String, u64>, paths_to_watch: &Vec<PathBuf>) {
    // Create a channel to receive the events.
    let (tx, rx) = channel();

    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.
    let mut watcher = watcher(tx, Duration::from_millis(5)).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    for path in paths_to_watch.iter() {
        watcher.watch(path, RecursiveMode::Recursive).unwrap();
    }

    loop {
        let r = rx.recv();
        if let Ok(event) = r {
            match event {
                DebouncedEvent::Create(file) => echo_file(file_map, file.as_path(), paths_to_watch),
                DebouncedEvent::Write(file) => echo_file(file_map, file.as_path(), paths_to_watch),

//                DebouncedEvent::Remove(path_buf) => println!("Remove[{:?}]", path_buf),
//                DebouncedEvent::Rename(path_buf_from, path_buf_to) => println!("Rename[{:?}]->[{:?}]", path_buf_from, path_buf_to),
                DebouncedEvent::Remove(_) => (),
                DebouncedEvent::Rename(_, _) => (),

                DebouncedEvent::NoticeWrite(_) => (),
                DebouncedEvent::NoticeRemove(_) => (),
                DebouncedEvent::Chmod(_) => (),
                DebouncedEvent::Rescan => (),

//                DebouncedEvent::Error(error, Some(path_buf)) => eprintln!("Error: [{}] for path [{:?}]", error, path_buf),
//                DebouncedEvent::Error(error, None) => eprintln!("Error: [{}]", error),
                DebouncedEvent::Error(_, _) => (),
            }
        } else {
            println!("VTAIL: Event: [{:?}]", r);
        }
    }
}

fn read_directory(file_map: &mut HashMap<String, u64>, rd: fs::ReadDir) {
//    println!("Value = {:?}", rd);
    for entry in rd {
        match entry {
            Ok(de) => process_entry(file_map, de),
            Err(e) => eprintln!("VTAIL: Cannot read directory. Error: [{}]", e),
        }
    }
}

fn process_entry(file_map: &mut HashMap<String, u64>, de: fs::DirEntry) {
//    println!("Dir entry = [{:?}]", de);
    let p = de.path();
    let p = p.as_path();
    if p.is_file() {
        match p.metadata() {
            Err(e) => eprintln!("VTAIL: Cannnot get the length from path [{:?}]. Error: [{}].", p, e),
            Ok(meta) => {
                let length = meta.len();
                process_file(file_map, p, length);
            }

        }
    }
}

fn process_file(file_map: &mut HashMap<String, u64>, p: &Path, last_fp: u64) {
    match p.to_str() {
        None => eprintln!("VTAIL: Cannot get file name from path [{:?}].", p),
        Some(s) => {
            let name = String::from(s);
//            println!("File  = [{}]=[{}]", name, last_fp);
            file_map.insert(name, last_fp);
        }
    }
}

fn echo_file(file_map: &mut HashMap<String, u64>, file: &Path, paths_to_watch: &Vec<PathBuf>) {
    let parent_path = file.parent().unwrap().to_path_buf();
//    println!("echo_file file=[{:?}], paths_to_watch=[{:?}], parent_path=[{:?}]", file, paths_to_watch, parent_path);

    // Don't do files in sub folders
    for path in paths_to_watch.iter() {
        if Ordering::Equal == path.cmp(&parent_path) {
            match file.to_str() {
                None => eprintln!("VTAIL: Cannot get file name from path [{:?}].", file),
                Some(s) => {
                    let name = String::from(s);
                    // println!("WRITE to {:?} name = {}", file, name);

                    let last_fp;
                    match file_map.get(&name) {
                        Some(fp) => {
                            last_fp = echo_file_from(&name, *fp);
                        },
                        None => {
                            last_fp = echo_whole_file(&name);
                        },
                    }
                    process_file(file_map, file, last_fp);
                }
            }
        }
    }
}

fn echo_whole_file(file_name: &String) -> u64 {
//    println!("File [{:?}] is new, echo complete file.", file_name);
    echo_file_from(file_name, 0)
}

fn echo_file_from(file_name: &String, fp: u64) -> u64 {
    let mut last_fp = fp;
//    println!("File pointer for file [{:?}] is [{}].", file_name, fp);
    let file_result = File::open(file_name);
    match file_result {
        Ok(mut file) => {
            match file.seek(SeekFrom::Start(fp)) {
                Err(e) => eprintln!("VTAIL: Could no seek in open file [{}]. Error: [{}]", file_name, e),
                Ok(_) => {
                    let mut content = String::new();
                    match file.read_to_string(&mut content) {
                        Err(e) => eprintln!("VTAIL: Could not read file [{}]. Error: [{}]", file_name, e),
                        Ok(_) => {
                            let path = Path::new(file_name);
                            let parent = path.parent().unwrap();
                            let file_name_os = parent.file_name().unwrap();
                            let file_name = file_name_os.to_str().unwrap();

                            let mut remaining_content = &content[..];
                            while let Some(next_newline) = remaining_content.find('\n') {
                                let line = &remaining_content[..next_newline];
                                let mut rest = &remaining_content[next_newline..];

                                // skip the newline
                                if rest.len() > 0 {
                                    rest = &rest[1..];
                                }
//                                println!("File: {}, Line: {}, Remaining: {}", file_name, line, rest);
                                println!("\x1b[1;34m{}\x1b[0m: {}", file_name, line);
                                last_fp = last_fp + (line.len() as u64) + 1;
                                remaining_content = rest;
                            }
                        },
                    }
                }
            }
        },
        Err(x) => eprintln!("VTAIL: Could not open file [{}] for reading. Error: [{}].", file_name, x),
    }

    last_fp
}

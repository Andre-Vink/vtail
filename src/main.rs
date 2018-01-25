use std::path::Path;
use std::fs;
use std::env;

fn main() {
    let args: env::Args = env::args();
    let args: Vec<String> = args.collect();

    if args.len() < 2 {
        panic!("USAGE: ptail <extension>. EXAMPLE: ptail txt");
    }

    let extension: &str = &args[1];

    println!("PTailing files ending with {}...", extension);

    let path: &Path = Path::new(".");
    println!("Path = {:?}", path);

    let cur_dir: fs::ReadDir = fs::read_dir(path).expect("Cannot read current directory");
    println!("CurDir = {:?}", cur_dir);

    for entry in cur_dir {
        if let Ok(entry) = entry {
            println!("Entry {:?}", entry.path());
        }
    }
}

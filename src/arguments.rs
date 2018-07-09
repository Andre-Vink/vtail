use std::env;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Arguments {
    recursive: bool,
    paths_to_watch: Vec<PathBuf>,
}

impl Arguments {
    pub fn parse_arguments() -> Arguments {
        let args: Vec<String> = env::args().collect();
//    println!("Arguments: [{:?}]", args);
        let mut usefull_args = &args[1..];
//    println!("Usefull arguments: [{:?}]", usefull_args);

        let mut recursive = false;
        // Test for recursive (-r) flag
        if usefull_args.len() > 0 {
            recursive = usefull_args[0].eq(&String::from("-r"));
            if recursive {
                usefull_args = &usefull_args[1..];
//            println!("Usefull arguments 2: [{:?}]", usefull_args);
            }
        }

        // Multiple paths supported
        let cur_dir = env::current_dir().unwrap();
        let mut result: Arguments = Arguments::new(recursive, Vec::new());

        if usefull_args.len() == 0 {
            result.add_path(cur_dir);
        } else {
            for arg in usefull_args.iter() {
                let absolute_path = cur_dir.join(arg);
                result.add_path(absolute_path);
            }
        }

        return result;
    }

//    pub fn is_recursive(&self) -> bool {
//        self.recursive
//    }

    pub fn get_paths(&self) -> &Vec<PathBuf> {
        &self.paths_to_watch
    }

    pub fn add_path(&mut self, path: PathBuf) {
        self.paths_to_watch.push(path);
    }

    fn new(recursive: bool, paths_to_watch: Vec<PathBuf>) -> Arguments {
        Arguments { recursive, paths_to_watch }
    }
}

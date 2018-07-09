use std::path::PathBuf;

#[derive(Debug)]
pub struct Arguments {
    recursive: bool,
    paths_to_watch: Vec<PathBuf>,
}

impl Arguments {
    pub fn new(recursive: bool, paths_to_watch: Vec<PathBuf>) -> Arguments {
        Arguments { recursive, paths_to_watch }
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
}

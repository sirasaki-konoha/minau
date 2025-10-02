use rodio::Decoder;
use std::io::BufReader;
use std::path::Path;
use std::{fs::File, process::exit};

use crate::err;

pub struct Player {
    pub decoder: Decoder<BufReader<File>>,
    pub file: File,
    pub path: String,
}

impl Clone for Player {
    fn clone(&self) -> Self {
        let decoder = Decoder::new(BufReader::new(self.file.try_clone().unwrap())).unwrap();

        Self {
            decoder,
            file: self.file.try_clone().unwrap(),
            path: self.path.clone(),
        }
    }
}

impl Player {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref().to_str().unwrap().to_string();

        let file = File::open(&path).unwrap_or_else(|e| {
            err!("Failed to open {}: {}", path, e);
            exit(1);
        });

        let decoder = Decoder::new(BufReader::new(file.try_clone().unwrap())).unwrap_or_else(|e| {
            err!("Failed to decode {}: {}", path, e);
            exit(1);
        });

        Player {
            decoder,
            file,
            path,
        }
    }
}

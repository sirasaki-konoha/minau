use rodio::Decoder;
use rodio::decoder::DecoderBuilder;
use std::io::BufReader;
use std::path::Path;
use std::{fs::File, process::exit};

use crate::err;

pub struct Player {
    pub decoder: Decoder<BufReader<File>>,
    pub path: String,
}

impl Clone for Player {
    fn clone(&self) -> Self {
        Player::new(&self.path)
    }
}

impl Player {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref().to_str().unwrap().to_string();

        let file = File::open(&path).unwrap_or_else(|e| {
            err!("Failed to open {}: {}", path, e);
            exit(1);
        });

        let buff = BufReader::new(file.try_clone().unwrap());
        let len = file
            .metadata()
            .unwrap_or_else(|e| {
                err!("Failed to get metadata from {}: {}", &path, e);
                exit(1);
            })
            .len();

        let decoder = DecoderBuilder::new()
            .with_seekable(true)
            .with_data(buff)
            .with_byte_len(len)
            .build()
            .unwrap_or_else(|e| {
                err!("Failed to build decoder: {}", e);
                exit(1);
            });

        // let decoder = Decoder::new(BufReader::new(file)).unwrap_or_else(|e| {
        //     err!("Failed to decode {}: {}", path, e);
        //     exit(1);
        // });

        Player { decoder, path }
    }
}

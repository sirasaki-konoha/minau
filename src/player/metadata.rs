use lofty::file::{AudioFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::properties::FileProperties;
use lofty::tag::{Accessor, Tag};
use std::fs::File;
use std::io::BufReader;
use std::process::exit;
use std::time::Duration;

use crate::err;

#[derive(Clone)]
pub struct MetaData {
    pub tag: Tag,
    pub prop: FileProperties,
}

impl MetaData {
    pub fn new(probe: Probe<BufReader<File>>) -> Self {
        let bind = probe.read().unwrap_or_else(|e| {
            err!("Failed to read metadata: {}", e);
            exit(1);
        });

        let Some(s) = bind.primary_tag() else {
            err!("Failed to get file tag");
            exit(1);
        };

        Self {
            tag: s.clone(),
            prop: bind.properties().clone(),
        }
    }

    pub fn title(&self) -> Option<String> {
        self.tag.title().as_ref().map(|title| title.to_string())
    }

    pub fn artist(&self) -> Option<String> {
        self.tag.artist().as_ref().map(|artist| artist.to_string())
    }

    pub fn album(&self) -> Option<String> {
        self.tag.album().as_ref().map(|album| album.to_string())
    }

    pub fn duration(&self) -> Duration {
        self.prop.duration()
    }

    /// returns first of picture data
    pub fn picture(&self) -> Option<Vec<u8>> {
        if let Some(s) = self.tag.pictures().first() {
            return Some(s.data().to_vec());
        }
        None
    }
}

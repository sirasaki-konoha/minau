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
    pub tag: Option<Tag>,
    pub prop: FileProperties,
}

impl MetaData {
    pub fn new(probe: Probe<BufReader<File>>) -> Self {
        let Ok(bind) = probe.read() else {
            err!("Failed to read metadata");
            exit(1);
        };

        let Some(s) = bind.primary_tag() else {
            err!("Failed to get file tag");
            return Self {
                tag: None,
                prop: bind.properties().clone(),
            };
        };

        Self {
            tag: Some(s.clone()),
            prop: bind.properties().clone(),
        }
    }

    pub fn title(&self) -> Option<String> {
        if let Some(tag) = &self.tag.clone() {
            tag.title().as_ref().map(|title| title.to_string())
        } else {
            return None;
        }
    }

    pub fn artist(&self) -> Option<String> {
        if let Some(tag) = &self.tag.clone() {
            tag.title().as_ref().map(|artist| artist.to_string())
        } else {
            return None;
        }
    }

    pub fn album(&self) -> Option<String> {
        if let Some(tag) = &self.tag.clone() {
            tag.title().as_ref().map(|album| album.to_string())
        } else {
            return None;
        }
    }

    pub fn duration(&self) -> Duration {
        self.prop.duration()
    }

    /// returns first of picture data
    pub fn picture(&self) -> Option<Vec<u8>> {
        if let Some(s) = self.tag.clone().unwrap().pictures().first() {
            return Some(s.data().to_vec());
        }
        None
    }
}

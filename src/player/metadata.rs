use lofty::file::{AudioFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::properties::FileProperties;
use lofty::tag::{Accessor, Tag};
use std::f32::consts::E;
use std::fs::File;
use std::io::BufReader;
use std::ops::Deref;
use std::process::exit;
use std::time::Duration;

use crate::err;

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
        if let Some(title) = &self.tag.title() {
            Some(title.to_string())
        } else {
            None
        }
    }

    pub fn artist(&self) -> Option<String> {
        if let Some(artist) = &self.tag.artist() {
            Some(artist.to_string())
        } else {
            None
        }
    }

    pub fn album(&self) -> Option<String> {
        if let Some(album) = &self.tag.album() {
            Some(album.to_string())
        } else {
            None
        }
    }

    pub fn track(&self) -> Option<u32> {
        if let Some(track) = &self.tag.track() {
            Some(track.deref().clone())
        } else {
            None
        }
    }

    pub fn duration(&self) -> Duration {
        let duration = self.prop.duration();
        duration
    }

    /// returns first of picture data
    pub fn picture(&self) -> Vec<u8> {
        let picture = self.tag.pictures()[0].clone();

        picture.data().to_vec()
    }
}

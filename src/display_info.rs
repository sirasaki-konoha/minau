use crate::player::metadata::MetaData;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Infos {
    None,
    Album,
    Artist,
    Title,
}

pub fn display_info(filename: &str, metadata: &MetaData) {
    let mut current = String::new();
    let mut infos = vec![Infos::None];

    if let Some(album) = metadata.album() {
        current.push_str(&format!("[{}] ", album));
        infos.push(Infos::Album);
    }

    if let Some(artist) = metadata.artist() {
        current.push_str(&format!("{} - ", artist));
        infos.push(Infos::Artist);
    }

    if let Some(title) = metadata.title() {
        current.push_str(&title.to_string());
        infos.push(Infos::Title);
    }

    if !infos.contains(&Infos::Title) {
        println!("Playing: {}", filename);
    } else {
        println!("{}", current);
    }
}

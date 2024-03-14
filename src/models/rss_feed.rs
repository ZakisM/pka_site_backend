use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename = "feed")]
pub struct YoutubeRssFeed {
    entry: Vec<Entry>,
}

impl YoutubeRssFeed {
    pub fn entry(&self) -> &Vec<Entry> {
        &self.entry
    }
}

#[derive(Debug, Deserialize)]
pub struct Entry {
    #[serde(rename = "videoId")]
    video_id: CompactString,
    title: CompactString,
    published: CompactString,
}

impl Entry {
    pub fn video_id(&self) -> &str {
        &self.video_id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn published(&self) -> &str {
        &self.published
    }
}

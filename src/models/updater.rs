use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct EpisodesFileRoot {
    pub episodes: Vec<EpisodeFromFile>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct EpisodeFromFile {
    pub identifier: String,
    pub number: f32,
    pub date_time: String,
    date: String,
    pub you_tube: String,
    timelined: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PkaInfoRoot {
    #[serde(rename = "Timeline")]
    pub timeline: PkaInfoTimeline,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PkaInfoTimeline {
    #[serde(rename = "Timestamps")]
    pub timestamps: Vec<PkaInfoTimestamp>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PkaInfoTimestamp {
    #[serde(rename = "HMS")]
    pub hms: String,
    #[serde(rename = "Value")]
    pub value: String,
}

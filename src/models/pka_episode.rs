use std::cmp::Ordering;

use float_ord::FloatOrd;
use serde::Serialize;

use crate::models::diesel_f32::DieselF32;
use crate::schema::pka_episode;
use crate::updater::pka::YOUTUBE_WATCH_URL;

#[derive(Debug, Serialize, Insertable, Queryable, Identifiable)]
#[serde(rename_all = "camelCase")]
#[primary_key(number)]
#[table_name = "pka_episode"]
pub struct PkaEpisode {
    number: DieselF32,
    name: String,
    #[serde(skip_serializing)]
    youtube_link: String,
    upload_date: i64,
}

impl PkaEpisode {
    pub fn new(number: f32, name: String, youtube_link: String, upload_date: i64) -> Self {
        Self {
            number: DieselF32(number),
            name,
            youtube_link,
            upload_date,
        }
    }

    pub fn number(&self) -> f32 {
        self.number.0
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn youtube_endpoint(&self) -> &str {
        self.youtube_link.as_str()
    }

    pub fn youtube_link(&self) -> String {
        format!("{}{}", YOUTUBE_WATCH_URL, self.youtube_link)
    }
}

impl std::cmp::Ord for PkaEpisode {
    fn cmp(&self, other: &Self) -> Ordering {
        FloatOrd(self.number.0).cmp(&FloatOrd(other.number.0))
    }
}

impl std::cmp::PartialOrd for PkaEpisode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        FloatOrd(self.number.0).partial_cmp(&FloatOrd(other.number.0))
    }
}

impl std::cmp::PartialEq for PkaEpisode {
    fn eq(&self, other: &Self) -> bool {
        FloatOrd(self.number.0).eq(&FloatOrd(other.number.0))
    }
}

impl std::cmp::Eq for PkaEpisode {}

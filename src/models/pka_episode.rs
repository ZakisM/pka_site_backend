use std::cmp::Ordering;

use compact_str::CompactString;
use float_ord::FloatOrd;
use serde::Serialize;
use sqlx::FromRow;

#[derive(Clone, Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PkaEpisode {
    pub number: f32,
    pub name: CompactString,
    #[serde(skip_serializing)]
    pub youtube_link: CompactString,
    pub upload_date: i64,
}

impl PkaEpisode {
    pub fn new(
        number: f32,
        name: CompactString,
        youtube_link: CompactString,
        upload_date: i64,
    ) -> Self {
        Self {
            number,
            name,
            youtube_link,
            upload_date,
        }
    }

    pub fn number(&self) -> f32 {
        self.number
    }
}

impl std::cmp::Ord for PkaEpisode {
    fn cmp(&self, other: &Self) -> Ordering {
        FloatOrd(self.number).cmp(&FloatOrd(other.number))
    }
}

impl std::cmp::PartialOrd for PkaEpisode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::PartialEq for PkaEpisode {
    fn eq(&self, other: &Self) -> bool {
        FloatOrd(self.number).eq(&FloatOrd(other.number))
    }
}

impl std::cmp::Eq for PkaEpisode {}

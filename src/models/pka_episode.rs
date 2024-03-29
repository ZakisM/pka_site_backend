use std::cmp::Ordering;

use compact_str::CompactString;
use float_ord::FloatOrd;
use serde::Serialize;

use crate::models::diesel_f32::DieselF32;
use crate::schema::pka_episode;

#[derive(Debug, Serialize, Insertable, Queryable, Identifiable)]
#[serde(rename_all = "camelCase")]
#[diesel(primary_key(number), table_name = pka_episode)]
pub struct PkaEpisode {
    number: DieselF32,
    name: CompactString,
    #[serde(skip_serializing)]
    youtube_link: CompactString,
    upload_date: i64,
}

impl PkaEpisode {
    pub fn new(
        number: f32,
        name: CompactString,
        youtube_link: CompactString,
        upload_date: i64,
    ) -> Self {
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

    pub fn youtube_link(&self) -> &str {
        self.youtube_link.as_str()
    }
}

impl std::cmp::Ord for PkaEpisode {
    fn cmp(&self, other: &Self) -> Ordering {
        FloatOrd(self.number.0).cmp(&FloatOrd(other.number.0))
    }
}

impl std::cmp::PartialOrd for PkaEpisode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::PartialEq for PkaEpisode {
    fn eq(&self, other: &Self) -> bool {
        FloatOrd(self.number.0).eq(&FloatOrd(other.number.0))
    }
}

impl std::cmp::Eq for PkaEpisode {}

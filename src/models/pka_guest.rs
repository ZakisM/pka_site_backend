use serde::Serialize;

use crate::models::pka_episode::PkaEpisode;
use crate::schema::pka_guest;

#[derive(Debug, Serialize, Insertable, Queryable, Associations, Identifiable)]
#[diesel(primary_key(name), belongs_to(PkaEpisode, foreign_key = episode_number), table_name = pka_guest)]
pub struct PkaGuest {
    name: String,
    episode_number: f32,
}

impl PkaGuest {
    pub fn new(name: String, episode_number: f32) -> Self {
        PkaGuest {
            name,
            episode_number,
        }
    }
}

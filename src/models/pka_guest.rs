use compact_str::CompactString;
use serde::Serialize;

use sqlx::FromRow;

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize, FromRow)]
pub struct PkaGuest {
    pub name: CompactString,
    pub episode_number: f32,
}

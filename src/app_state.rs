use std::sync::Arc;

use crate::{redis_db::RedisDb, Repo};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Repo>,
    pub redis: Arc<RedisDb>,
}

impl AppState {
    pub fn new(db: Arc<Repo>, redis: Arc<RedisDb>) -> Self {
        Self { db, redis }
    }
}

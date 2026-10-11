use std::sync::Arc;

use crate::Repo;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Repo>,
}

impl AppState {
    pub fn new(db: Arc<Repo>) -> Self {
        Self { db }
    }
}

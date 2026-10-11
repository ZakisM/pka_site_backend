use crate::catalog::SharedCatalog;

#[derive(Clone)]
pub struct AppState {
    pub catalog: SharedCatalog,
}

impl AppState {
    pub fn new(catalog: SharedCatalog) -> Self {
        Self { catalog }
    }
}

use std::sync::Arc;

use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct YtApiKey {
    inner: Arc<RwLock<String>>,
}

impl YtApiKey {
    pub async fn set(&self, value: impl Into<String>) {
        *self.inner.write().await = value.into();
    }

    pub async fn get(&self) -> String {
        self.inner.read().await.clone()
    }
}

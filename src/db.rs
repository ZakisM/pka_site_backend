use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::{r2d2, Connection};

pub struct SqDatabase<T>
where
    T: Connection + 'static,
{
    connection_pool: Pool<ConnectionManager<T>>,
}

impl<T> SqDatabase<T>
where
    T: Connection + 'static,
{
    pub fn new(database_url: &str) -> Self {
        let manager = ConnectionManager::new(database_url);
        let pool = r2d2::Builder::default()
            .build(manager)
            .expect("Failed to build connection pool for database.");

        SqDatabase {
            connection_pool: pool,
        }
    }

    pub async fn run<F, R>(&self, f: F) -> R
    where
        F: 'static
            + FnOnce(PooledConnection<ConnectionManager<T>>) -> R
            + Send
            + std::marker::Unpin,
        T: Send,
        R: 'static + Send,
    {
        let pool = self.connection_pool.clone();
        tokio::task::spawn_blocking(move || {
            let connection = pool.get().expect("Failed to get connection from pool");
            (f)(connection)
        })
        .await
        .expect("Failed to run diesel query on threadpool")
    }
}

use diesel::r2d2::{ConnectionManager, Pool, PooledConnection, R2D2Connection};
use diesel::{r2d2, Connection};

pub struct SqDatabase<T>
where
    T: Connection + 'static + R2D2Connection,
{
    connection_pool: Pool<ConnectionManager<T>>,
}

impl<T> SqDatabase<T>
where
    T: Connection + 'static + R2D2Connection,
{
    pub fn new(database_url: &str) -> Self {
        let manager = ConnectionManager::new(database_url);
        let pool = r2d2::Builder::new()
            .max_size(15)
            .build(manager)
            .expect("Failed to build connection pool for database.");

        SqDatabase {
            connection_pool: pool,
        }
    }

    pub async fn run<F, R>(&self, f: F) -> R
    where
        F: 'static
            + FnOnce(&mut PooledConnection<ConnectionManager<T>>) -> R
            + Send
            + std::marker::Unpin,
        T: Send,
        R: 'static + Send,
    {
        let pool = self.connection_pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut connection = pool.get().expect("Failed to get connection from pool");
            (f)(&mut connection)
        })
        .await
        .expect("Failed to run diesel query on threadpool")
    }
}

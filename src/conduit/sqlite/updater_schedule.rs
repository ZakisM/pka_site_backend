use sqlx::SqliteExecutor;

/// When `job` may next ask YouTube anything, in Unix seconds; `None` if it
/// Never has.
pub async fn next_at<'e>(
    repo: impl SqliteExecutor<'e>,
    job: &str,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT next_at AS "next_at: i64" FROM updater_schedule WHERE job = ?"#,
        job
    )
    .fetch_optional(repo)
    .await
}

/// Saves `next_at` for `job` only if what's saved is still `expected` (`None`
/// For nothing yet), and says whether it did: false means another process
/// Sharing the database changed it first.
pub async fn replace_next_at<'e>(
    repo: impl SqliteExecutor<'e>,
    job: &str,
    expected: Option<i64>,
    next_at: i64,
) -> Result<bool, sqlx::Error> {
    let result = match expected {
        None => {
            sqlx::query!(
                r#"INSERT INTO updater_schedule (job, next_at) VALUES (?, ?)
                   ON CONFLICT (job) DO NOTHING"#,
                job,
                next_at
            )
            .execute(repo)
            .await?
        }
        Some(expected) => {
            sqlx::query!(
                r#"UPDATE updater_schedule SET next_at = ? WHERE job = ? AND next_at = ?"#,
                next_at,
                job,
                expected
            )
            .execute(repo)
            .await?
        }
    };

    Ok(result.rows_affected() == 1)
}

pub async fn set_next_at<'e>(
    repo: impl SqliteExecutor<'e>,
    job: &str,
    at: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO updater_schedule (job, next_at) VALUES (?, ?)
           ON CONFLICT (job) DO UPDATE SET next_at = excluded.next_at"#,
        job,
        at
    )
    .execute(repo)
    .await?;

    Ok(())
}

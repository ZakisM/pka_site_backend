//! When each of the updater's jobs may next ask YouTube anything. Saved in
//! the database, so a restart (a deploy, or a crash loop under `restart:
//! always`) waits out the same schedule rather than spending quota straight
//! away. Read only at startup: while running, jobs wait on the monotonic
//! clock, which setting the system time doesn't move.

use std::time::Duration;

use tokio::time::Instant;
use tracing::{error, info, warn};

use crate::Repo;
use crate::conduit::sqlite::updater_schedule;

/// The time now on both clocks: the wall clock's is what's saved, as it means
/// The same after a restart; the monotonic clock's is what's waited on.
#[derive(Clone, Copy, Debug)]
pub struct Now {
    pub wall: i64,
    pub instant: Instant,
}

impl Now {
    pub fn read() -> Self {
        Now {
            wall: chrono::Utc::now().timestamp(),
            instant: Instant::now(),
        }
    }
}

fn seconds(duration: Duration) -> i64 {
    i64::try_from(duration.as_secs()).unwrap_or(i64::MAX)
}

// How long until a saved time, capped at the longest the job is ever told to
// Wait: a time saved while the clock ran ahead would otherwise stall it until
// The clock caught up again, months later perhaps. The cap never makes a job
// Run early.
fn wait_until(next_at: i64, now: i64, longest_wait: Duration) -> Duration {
    let wait = u64::try_from(next_at.saturating_sub(now)).unwrap_or(0);

    Duration::from_secs(wait).min(longest_wait)
}

pub struct Job {
    name: &'static str,
    // From a claimed run to the next, unless rescheduled.
    retry_after: Duration,
    // The longest the job is ever told to wait.
    longest_wait: Duration,
    // The saved time, as this process last read or wrote it.
    saved: Option<i64>,
    next: Instant,
}

impl Job {
    /// The job as last scheduled; due now if it has never run. If the
    /// Schedule can't be read, it waits `retry_after` as if it had just run:
    /// Not knowing is no reason to ask YouTube.
    pub async fn load(
        repo: &Repo,
        name: &'static str,
        retry_after: Duration,
        longest_wait: Duration,
        now: Now,
    ) -> Self {
        let mut job = Job {
            name,
            retry_after,
            longest_wait,
            saved: None,
            next: now.instant,
        };

        job.read(repo, now).await;

        info!(
            job = name,
            in_seconds = job.next.saturating_duration_since(now.instant).as_secs(),
            "Next run"
        );

        job
    }

    // Takes up whatever is saved.
    async fn read(&mut self, repo: &Repo, now: Now) {
        match updater_schedule::next_at(repo, self.name).await {
            Ok(saved) => {
                self.saved = saved;
                self.next = now.instant
                    + saved.map_or(Duration::ZERO, |at| {
                        wait_until(at, now.wall, self.longest_wait)
                    });
            }
            Err(e) => {
                error!("Reading when {} next runs failed: {e:#}", self.name);
                self.next = now.instant + self.retry_after;
            }
        }
    }

    pub fn is_due(&self, now: Now) -> bool {
        now.instant >= self.next
    }

    /// Claims a due run before it asks YouTube anything: records that the
    /// Next is `retry_after` from now unless rescheduled, so a crash partway
    /// Through can't make it run again early. False if it mustn't run: that
    /// Couldn't be recorded, or another process sharing the database changed
    /// The schedule first (and this one follows it from now on).
    pub async fn claim(&mut self, repo: &Repo, now: Now) -> bool {
        let next_at = now.wall.saturating_add(seconds(self.retry_after));

        self.next = now.instant + self.retry_after;

        match updater_schedule::replace_next_at(repo, self.name, self.saved, next_at).await {
            Ok(true) => {
                self.saved = Some(next_at);
                true
            }
            Ok(false) => {
                warn!("Another process rescheduled {}; following it", self.name);
                self.read(repo, now).await;
                false
            }
            Err(e) => {
                error!(
                    "Recording that {} runs failed, so it doesn't: {e:#}",
                    self.name
                );
                false
            }
        }
    }

    /// Moves the next run to `after` from now. If that can't be saved, this
    /// Process still keeps to it; only a restart falls back to the claimed
    /// Time.
    pub async fn reschedule(&mut self, repo: &Repo, now: Now, after: Duration) {
        let next_at = now.wall.saturating_add(seconds(after));

        self.next = now.instant + after;

        match updater_schedule::set_next_at(repo, self.name, next_at).await {
            Ok(()) => self.saved = Some(next_at),
            Err(e) => error!("Rescheduling {} failed: {e:#}", self.name),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

    use super::*;

    const WALL: i64 = 1_800_000_000;
    const SECOND: Duration = Duration::from_secs(1);
    const MINUTE: Duration = Duration::from_secs(60);
    const HOUR: Duration = Duration::from_secs(60 * 60);
    const DAY: Duration = Duration::from_secs(24 * 60 * 60);

    async fn database() -> Repo {
        // One connection that never closes: each connection to `:memory:` is
        // A database of its own.
        let repo = SqlitePoolOptions::new()
            .max_connections(1)
            .min_connections(1)
            .idle_timeout(None)
            .max_lifetime(None)
            .connect_with(SqliteConnectOptions::from_str("sqlite::memory:").unwrap())
            .await
            .unwrap();

        sqlx::migrate!().run(&repo).await.unwrap();

        repo
    }

    // `after` the test began, on clocks that agree.
    fn at(start: Instant, after: Duration) -> Now {
        Now {
            wall: WALL + seconds(after),
            instant: start + after,
        }
    }

    // Retried hourly, at most a day apart: the sync's schedule.
    async fn sync(repo: &Repo, now: Now) -> Job {
        Job::load(repo, "sync", HOUR, DAY, now).await
    }

    // Every five minutes, at most three hours apart: the new-episode check's.
    async fn check(repo: &Repo, now: Now) -> Job {
        Job::load(repo, "new_episodes", 5 * MINUTE, 3 * HOUR, now).await
    }

    async fn drop_schedule(repo: &Repo) {
        sqlx::query("DROP TABLE updater_schedule")
            .execute(repo)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_a_job_that_never_ran_is_due() {
        let repo = database().await;
        let start = Instant::now();
        let job = sync(&repo, at(start, Duration::ZERO)).await;

        assert!(job.is_due(at(start, Duration::ZERO)));
    }

    #[tokio::test]
    async fn test_a_restart_waits_out_a_claimed_run() {
        let repo = database().await;
        let start = Instant::now();
        let mut job = sync(&repo, at(start, Duration::ZERO)).await;

        assert!(job.claim(&repo, at(start, Duration::ZERO)).await);
        assert!(!job.is_due(at(start, HOUR - SECOND)));

        // As if the process restarted a minute in, before the run finished.
        let restarted = sync(&repo, at(start, MINUTE)).await;

        assert!(!restarted.is_due(at(start, MINUTE)));
        assert!(!restarted.is_due(at(start, HOUR - SECOND)));
        assert!(restarted.is_due(at(start, HOUR)));
    }

    #[tokio::test]
    async fn test_a_rescheduled_run_survives_a_restart() {
        let repo = database().await;
        let start = Instant::now();
        let mut job = check(&repo, at(start, Duration::ZERO)).await;

        assert!(job.claim(&repo, at(start, Duration::ZERO)).await);
        job.reschedule(&repo, at(start, Duration::ZERO), 3 * HOUR)
            .await;

        let restarted = check(&repo, at(start, MINUTE)).await;

        assert!(!restarted.is_due(at(start, 3 * HOUR - SECOND)));
        assert!(restarted.is_due(at(start, 3 * HOUR)));
        // Each job keeps its own schedule.
        assert!(
            sync(&repo, at(start, MINUTE))
                .await
                .is_due(at(start, MINUTE))
        );
    }

    #[tokio::test]
    async fn test_a_job_that_cannot_record_its_run_does_not_run() {
        let repo = database().await;
        let start = Instant::now();
        let mut job = sync(&repo, at(start, Duration::ZERO)).await;

        drop_schedule(&repo).await;

        assert!(!job.claim(&repo, at(start, Duration::ZERO)).await);
        // Nor tries again every tick.
        assert!(!job.is_due(at(start, HOUR - SECOND)));
    }

    #[tokio::test]
    async fn test_a_schedule_that_cannot_be_read_waits_as_if_the_job_just_ran() {
        let repo = database().await;
        let start = Instant::now();

        drop_schedule(&repo).await;

        let job = sync(&repo, at(start, Duration::ZERO)).await;

        assert!(!job.is_due(at(start, Duration::ZERO)));
        assert!(!job.is_due(at(start, HOUR - SECOND)));
        assert!(job.is_due(at(start, HOUR)));
    }

    // A server that boots with its clock months ahead, then has it corrected.
    #[tokio::test]
    async fn test_a_clock_set_back_does_not_stall_a_running_job() {
        let repo = database().await;
        let start = Instant::now();
        let ahead = Now {
            wall: WALL + seconds(80 * DAY),
            instant: start,
        };
        let mut job = sync(&repo, ahead).await;

        assert!(job.claim(&repo, ahead).await);
        job.reschedule(&repo, ahead, DAY).await;

        // A day later by the monotonic clock, whatever the wall clock says.
        assert!(!job.is_due(at(start, DAY - SECOND)));
        assert!(job.is_due(at(start, DAY)));
        assert!(job.claim(&repo, at(start, DAY)).await);
    }

    #[tokio::test]
    async fn test_a_time_saved_while_the_clock_ran_ahead_waits_at_most_the_longest_wait() {
        let repo = database().await;
        let start = Instant::now();

        updater_schedule::set_next_at(&repo, "sync", WALL + seconds(80 * DAY))
            .await
            .unwrap();

        let mut job = sync(&repo, at(start, Duration::ZERO)).await;

        assert!(!job.is_due(at(start, DAY - SECOND)));
        assert!(job.is_due(at(start, DAY)));
        assert!(job.claim(&repo, at(start, DAY)).await);
    }

    #[tokio::test]
    async fn test_only_one_of_two_processes_sharing_the_database_runs_a_job() {
        let repo = database().await;
        let start = Instant::now();
        let mut first = sync(&repo, at(start, Duration::ZERO)).await;
        let mut second = sync(&repo, at(start, Duration::ZERO)).await;

        assert!(first.claim(&repo, at(start, Duration::ZERO)).await);
        assert!(!second.claim(&repo, at(start, Duration::ZERO)).await);

        // The second follows the first's schedule from then on...
        first.reschedule(&repo, at(start, MINUTE), DAY).await;
        assert!(second.is_due(at(start, HOUR)));
        assert!(!second.claim(&repo, at(start, HOUR)).await);
        assert!(!second.is_due(at(start, DAY)));

        // ...and once a run is due, whichever claims it first runs it.
        assert!(second.is_due(at(start, DAY + MINUTE)));
        assert!(second.claim(&repo, at(start, DAY + MINUTE)).await);
        assert!(!first.claim(&repo, at(start, DAY + MINUTE)).await);
    }
}

-- When each of the updater's jobs may next ask YouTube anything (Unix
-- seconds), so a restart waits out the same schedule instead of asking again
-- straight away.
CREATE TABLE updater_schedule
(
    job     TEXT   NOT NULL PRIMARY KEY,
    next_at BIGINT NOT NULL
);

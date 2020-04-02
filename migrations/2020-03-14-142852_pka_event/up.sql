CREATE TABLE pka_event
(
    event_id       TEXT   NOT NULL,
    episode_number REAL   NOT NULL,
    timestamp      BIGINT NOT NULL,
    description    TEXT   NOT NULL,
    PRIMARY KEY (event_id),
    FOREIGN KEY (episode_number) REFERENCES pka_episode (number) ON DELETE CASCADE
);
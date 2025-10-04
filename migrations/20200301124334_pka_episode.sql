CREATE TABLE IF NOT EXISTS pka_episode
(
    number       REAL   NOT NULL,
    name         TEXT   NOT NULL,
    youtube_link TEXT   NOT NULL,
    upload_date  BIGINT NOT NULL,
    PRIMARY KEY (number)
);

CREATE TABLE pka_youtube_details
(
    video_id       TEXT    NOT NULL,
    episode_number REAL    NOT NULL,
    title          TEXT    NOT NULL,
    length_seconds INTEGER NOT NULL,
    PRIMARY KEY (video_id),
    FOREIGN KEY (episode_number) REFERENCES pka_episode (number) ON DELETE CASCADE
);

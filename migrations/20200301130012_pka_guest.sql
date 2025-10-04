CREATE TABLE IF NOT EXISTS pka_guest
(
    name           TEXT NOT NULL,
    episode_number REAL NOT NULL,
    PRIMARY KEY (name),
    FOREIGN KEY (episode_number) REFERENCES pka_episode (number) ON DELETE CASCADE
);

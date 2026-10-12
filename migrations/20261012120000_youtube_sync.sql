-- Episodes whose moments are the site's own, never replaced from YouTube's
-- description: 134, whose description is only links, and any part added by
-- hand (a fractional number), whose video's description is for another part.
ALTER TABLE pka_episode ADD COLUMN moments_locked INTEGER NOT NULL DEFAULT 0;

-- When YouTube was first found without this episode's video. An episode is
-- hidden while its video is missing, and deleted once it has been for a week.
ALTER TABLE pka_episode ADD COLUMN missing_since BIGINT;

UPDATE pka_episode SET moments_locked = 1 WHERE number = 134 OR number <> CAST(number AS INTEGER);

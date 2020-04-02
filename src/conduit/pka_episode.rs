use diesel::prelude::*;
use diesel::result::Error;

use crate::models::pka_episode::PkaEpisode;
use crate::models::pka_episode_with_all::PkaEpisodeWithAll;
use crate::models::pka_event::PkaEvent;
use crate::models::pka_youtube_details::PkaYoutubeDetails;
use crate::schema::pka_episode::dsl::*;
use crate::schema::pka_event::columns::timestamp;
use crate::{schema, Repo};

pub async fn all(repo: &Repo) -> Result<Vec<PkaEpisode>, Error> {
    repo.run(move |conn| {
        pka_episode
            .order_by(number.desc())
            .load::<PkaEpisode>(&conn)
    })
    .await
}

pub async fn latest(repo: &Repo) -> Result<f32, Error> {
    repo.run(move |conn| {
        pka_episode
            .select(number)
            .order_by(number.desc())
            .first::<f32>(&conn)
    })
    .await
}

pub async fn find_with_all(repo: &Repo, id: f32) -> Result<PkaEpisodeWithAll, Error> {
    repo.run(move |conn| {
        let episode = pka_episode.find(id).first::<PkaEpisode>(&conn)?;

        let events = <PkaEvent as BelongingToDsl<&PkaEpisode>>::belonging_to(&episode)
            .order_by(timestamp.asc())
            .load(&conn)?;

        let youtube_details =
            <PkaYoutubeDetails as BelongingToDsl<&PkaEpisode>>::belonging_to(&episode)
                .first(&conn)?;

        Ok(PkaEpisodeWithAll::new(episode, youtube_details, events))
    })
    .await
}

pub async fn insert(repo: &Repo, episode: PkaEpisode) -> Result<(), Error> {
    repo.run(move |conn| {
        diesel::insert_into(schema::pka_episode::table)
            .values(episode)
            .execute(&conn)?;

        Ok(())
    })
    .await
}

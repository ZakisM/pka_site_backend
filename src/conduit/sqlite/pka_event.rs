use diesel::prelude::*;
use diesel::result::Error;
use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::models::pka_event::PkaEvent;
use crate::models::search::PkaEventSearchResult;
use crate::schema::pka_event::dsl::*;
use crate::{schema, Repo};

pub async fn all(repo: &Repo) -> Result<Vec<PkaEvent>, Error> {
    repo.run(move |conn| pka_event.load::<PkaEvent>(conn)).await
}

pub async fn insert(repo: &Repo, event: PkaEvent) -> Result<(), Error> {
    repo.run(move |conn| {
        diesel::insert_into(schema::pka_event::table)
            .values(event)
            .execute(conn)?;

        Ok(())
    })
    .await
}

pub async fn random_amount(repo: &Repo, amount: usize) -> Result<Vec<PkaEventSearchResult>, Error> {
    repo.run(move |conn| {
        let mut all_events = pka_event.load::<PkaEvent>(conn)?;

        all_events.retain(|e| {
            let des = e.description().to_lowercase();

            !des.contains("outro") && !des.contains("intro") && !des.contains("ad read")
        });

        let mut rng = thread_rng();

        let res = all_events
            .choose_multiple(&mut rng, amount)
            .map(PkaEventSearchResult::from)
            .collect::<Vec<_>>();

        Ok(res)
    })
    .await
}

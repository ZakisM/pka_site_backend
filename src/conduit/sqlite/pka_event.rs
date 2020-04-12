use diesel::prelude::*;
use diesel::result::Error;

use crate::models::pka_event::PkaEvent;
use crate::schema::pka_event::dsl::*;
use crate::{schema, Repo};

pub async fn all(repo: &Repo) -> Result<Vec<PkaEvent>, Error> {
    repo.run(move |conn| pka_event.load::<PkaEvent>(&conn))
        .await
}

pub async fn insert(repo: &Repo, event: PkaEvent) -> Result<(), Error> {
    repo.run(move |conn| {
        diesel::insert_into(schema::pka_event::table)
            .values(event)
            .execute(&conn)?;

        Ok(())
    })
    .await
}

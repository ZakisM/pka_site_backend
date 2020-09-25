use diesel::prelude::*;
use diesel::result::Error;

use crate::models::pka_youtube_details::PkaYoutubeDetails;
use crate::{schema, Repo};

pub async fn insert(repo: &Repo, details: PkaYoutubeDetails) -> Result<(), Error> {
    repo.run(move |conn| {
        diesel::insert_into(schema::pka_youtube_details::table)
            .values(details)
            .execute(&conn)?;

        Ok(())
    })
    .await
}

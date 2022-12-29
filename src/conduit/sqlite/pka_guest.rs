// use diesel::prelude::*;
// use diesel::result::Error;
//
// use crate::models::pka_guest::PkaGuest;
// use crate::schema::pka_guest::dsl::*;
// use crate::{schema, Repo};

// pub async fn all(repo: &Repo) -> Result<Vec<PkaGuest>, Error> {
//     repo.run(move |conn| pka_guest.load::<PkaGuest>(conn))
//         .await
// }
//
// pub async fn find(repo: &Repo, guest: String) -> Result<PkaGuest, Error> {
//     repo.run(move |conn| pka_guest.find(guest).first(conn))
//         .await
// }
//
// pub async fn insert(repo: &Repo, episode: PkaGuest) -> Result<(), Error> {
//     repo.run(move |conn| {
//         diesel::insert_into(schema::pka_guest::table)
//             .values(episode)
//             .execute(conn)?;
//
//         Ok(())
//     })
//     .await
// }

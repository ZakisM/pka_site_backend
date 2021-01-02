// use std::path::{Path, PathBuf};
//
// use diesel::prelude::*;
// use futures::stream::{self, StreamExt};
// use tokio::fs::{self};
//
// use crate::models::pka_episode::PkaEpisode;
// use crate::schema::pka_episode::dsl::*;
// use crate::updater::youtube_api::YoutubeAPI;
// use crate::{Repo, Result};

// Only used initially to store episodes in DB from external source.
//
// pub const PKA_DESCRIPTIONS_FOLDER: &str = "PKA-Descriptions";
//
// pub async fn add_all_event_durations(state: &Repo) -> Result<()> {
//     let all_episodes = pka_episode::all(state).await?;
//
//     for episode in all_episodes {
//         let episode_data = pka_episode::find_with_all(state, episode.number()).await?;
//
//         let events_without_duration = episode_data.events();
//
//         let events: Vec<PkaEvent> = events_without_duration
//             .iter()
//             .enumerate()
//             .map(|(index, event)| {
//                 //Calculate duration of each event
//                 let l = if let Some(next_event) = events_without_duration.get(index + 1) {
//                     next_event.timestamp() - event.timestamp()
//                 } else {
//                     episode_data.youtube_details.length_seconds() - event.timestamp()
//                 };
//
//                 if l < 0 {
//                     println!("{}", episode_data.youtube_details.length_seconds());
//                     panic!("length < 0 for {:?}", event);
//                 }
//
//                 PkaEvent {
//                     length_seconds: l,
//                     ..event.to_owned()
//                 }
//             })
//             .collect();
//
//         for e in events {
//             if let Err(e) = state
//                 .run(|conn| {
//                     diesel::update(pka_event.filter(event_id.eq(e.event_id())))
//                         .set(length_seconds.eq(e.length_seconds()))
//                         .execute(&conn)
//                 })
//                 .await
//             {
//                 println!("{}", e);
//             }
//         }
//     }
//
//     Ok(())
// }
//
// pub async fn add_all_pka_youtube_details(state: &Repo) -> Result<()> {
//     let yt_api = YoutubeAPI::new();
//
//     let all_episodes = pka_episode::all(state).await?;
//
//     let all_details = pka_youtube_details::all(state).await?;
//
//     let missing = all_episodes
//         .into_iter()
//         .filter(|e| {
//             e.youtube_endpoint() != "N/A"
//                 && all_details
//                     .iter()
//                     .find(|d| FloatOrd(d.episode_number()) == FloatOrd(e.number()))
//                     .is_none()
//         })
//         .collect::<Vec<PkaEpisode>>();
//
//     let bodies = stream::iter(missing)
//         .map(|ep| async {
//             let yt_api_data = yt_api.get_video_details(&ep.youtube_endpoint()).await;
//             (yt_api_data, ep)
//         })
//         .buffer_unordered(7);
//
//     let fut = bodies.for_each_concurrent(7, |(yt_api_data, ep)| async move {
//         match yt_api_data {
//             Ok(yt_api_data) => {
//                 let youtube_details = PkaYoutubeDetails::new(
//                     yt_api_data.id,
//                     ep.number(),
//                     yt_api_data.snippet.title,
//                     yt_api_data.content_details.duration,
//                 );
//
//                 if let Err(e) = pka_youtube_details::insert(state, youtube_details).await {
//                     error!("{}", e);
//                 };
//             }
//             Err(e) => error!(
//                 "Error downloading video details for: {} - {}",
//                 ep.number(),
//                 e
//             ),
//         }
//     });
//
//     fut.await;
//
//     Ok(())
// }
//
// pub async fn add_all_pka_episode_events(state: &Repo) -> Result<()> {
//     let dir_name = Path::new(PKA_DESCRIPTIONS_FOLDER);
//
//     let mut res = fs::read_dir(dir_name).await?;
//
//     while let Some(entry) = res.next().await {
//         if let Ok(entry) = entry {
//             let name = entry
//                 .file_name()
//                 .to_str()
//                 .expect("Failed to convert DirEntry to string")
//                 .to_owned();
//
//             if name.starts_with("PKA") {
//                 let data = fs::read_to_string(entry.path())
//                     .await
//                     .expect("Failed to read file contents.");
//
//                 let ep_number: f32 = name
//                     .replace("PKA ", "")
//                     .replace(".txt", "")
//                     .parse()
//                     .expect("Failed to convert ep_number to f32");
//
//                 match extract_pka_episode_events(ep_number, &data, None) {
//                     Ok(events) => {
//                         for event in events.into_iter() {
//                             if let Err(e) = pka_event::insert(state, event).await {
//                                 error!("{}", e);
//                             }
//                         }
//                     }
//                     Err(e) => error!("{}", e),
//                 }
//             }
//         }
//     }
//
//     Ok(())
// }
//
// pub async fn download_all_pka_episodes_timelines_pka_info(
//     state: &Repo,
//     client: &Client,
// ) -> Result<()> {
//     let all_episodes = pka_episode::all(state).await?;
//
//     let all_details = pka_event::all(state).await?;
//
//     let missing = all_episodes
//         .into_iter()
//         .filter(|e| {
//             e.youtube_endpoint() != "N/A"
//                 && all_details
//                     .iter()
//                     .find(|d| FloatOrd(d.episode_number()) == FloatOrd(e.number()))
//                     .is_none()
//         })
//         .map(|e| e.number())
//         .collect::<Vec<f32>>();
//
//     let dir_name = Path::new(PKA_DESCRIPTIONS_FOLDER);
//
//     if !dir_name.exists() {
//         fs::create_dir(dir_name)
//             .await
//             .expect("Failed to create dir");
//     }
//
//     let bodies = stream::iter(missing)
//         .map(move |ep| async move {
//             let res = client
//                 .get(
//                     format!(
//                         "https://www.painkilleralready.info/api/episode.php?episode={}",
//                         ep
//                     )
//                     .parse::<Url>()
//                     .expect("Failed to parse youtube_link into URL"),
//                 )
//                 .send()
//                 .await
//                 .expect("Failed to send painkilleralready.info request");
//             (res, ep)
//         })
//         .buffer_unordered(5);
//
//     let fut = bodies.for_each_concurrent(5, |(res, ep)| async move {
//         let api_response = serde_json::from_slice::<PkaInfoRoot>(
//             &res.bytes().await.expect("Failed to read response"),
//         );
//
//         if let Ok(api_response) = api_response {
//             if !api_response.timeline.timestamps.is_empty() {
//                 let mut file = File::create(format!(
//                     "{}/PKA {:<03}.txt",
//                     dir_name
//                         .to_str()
//                         .expect("Failed to convert dir_name to String"),
//                     ep
//                 ))
//                 .await
//                 .expect("Failed to create file");
//
//                 let formatted_timeline = api_response.timeline.timestamps.into_iter().fold(
//                     "".to_owned(),
//                     |mut next, curr| {
//                         next.push_str(format!("{} - {}\n", curr.hms, curr.value).as_str());
//                         next
//                     },
//                 );
//
//                 file.write_all(formatted_timeline.as_bytes())
//                     .await
//                     .expect("Failed to write description to file");
//
//                 println!("Downloaded: {}", ep);
//             } else {
//                 println!("Timelines were empty: {}", ep);
//             }
//         } else {
//             println!("Failed to download description for PKA: {}", ep)
//         }
//     });
//
//     fut.await;
//
//     Ok(())
// }
//
// pub async fn download_all_pka_episode_descriptions(state: &Repo) -> Result<()> {
//     let descriptions_path = Path::new(PKA_DESCRIPTIONS_FOLDER);
//
//     if !descriptions_path.exists() {
//         fs::create_dir(descriptions_path)
//             .await
//             .expect("Failed to create PKA_DESCRIPTIONS_FOLDER");
//     }
//
//     let yt_api = YoutubeAPI::new();
//
//     let all_episodes = state
//         .run(move |conn| pka_episode.load::<PkaEpisode>(&conn))
//         .await?
//         .into_iter()
//         .map(|ep| {
//             let path: PathBuf = descriptions_path.join(format!("PKA {:03}.txt", ep.number()));
//
//             (ep, path)
//         })
//         .filter(|(ep, path)| ep.youtube_endpoint() != "N/A" && !path.exists())
//         .collect::<Vec<_>>();
//
//     let mut job = stream::iter(all_episodes)
//         .map(|(ep, path)| async {
//             let video_data = yt_api.get_video_details(&ep.youtube_endpoint()).await;
//             (ep, video_data, path)
//         })
//         .buffer_unordered(5);
//
//     while let Some((ep, video_data, path)) = job.next().await {
//         match video_data {
//             Ok(video_data) => {
//                 if let Err(e) = fs::write(path, video_data.snippet.description).await {
//                     eprintln!("[{}] {}", ep.number(), e);
//                 } else {
//                     println!("Successfully downloaded: PKA {} description", ep.number());
//                 }
//             }
//             Err(e) => eprintln!("[{}] {}", ep.number(), e),
//         }
//     }
//
//     Ok(())
// }
//
// pub async fn add_pka_episodes_from_file(state: &Repo) {
//     let f = tokio::fs::read_to_string("./pka_episode_list.json")
//         .await
//         .expect("Failed to read 'pka_episode_list.json'.");
//
//     let data = serde_json::from_str::<EpisodesFileRoot>(&f)
//         .expect("Failed to convert episode list to struct.");
//
//     for ep in data.episodes.into_iter() {
//         let upload_date = chrono::naive::NaiveDateTime::parse_from_str(
//             &format!("{} 00:00:00", &ep.date_time),
//             "%Y-%m-%d %H:%M:%S",
//         )
//         .expect("Failed to convert upload_date to NaiveDateTime")
//         .timestamp();
//
//         let pka_ep = PkaEpisode::new(ep.number, ep.identifier, ep.you_tube, upload_date);
//
//         if let Err(e) = pka_episode::insert(state, pka_ep).await {
//             error!("Failed to add [{}] - {}", ep.number, e.to_string());
//         }
//     }
// }

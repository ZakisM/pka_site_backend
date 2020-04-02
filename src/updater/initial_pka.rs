use std::path::Path;

use float_ord::FloatOrd;
use futures::stream::{self, StreamExt};
use reqwest::{Client, Url};
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;

use crate::conduit::{pka_episode, pka_event, pka_youtube_details};
use crate::models::pka_episode::PkaEpisode;
use crate::models::pka_youtube_details::PkaYoutubeDetails;
use crate::models::updater::{EpisodesFileRoot, PkaInfoRoot};
use crate::updater::pka::{extract_pka_episode_events, get_video_details, PKA_DESCRIPTIONS_FOLDER};
use crate::{Repo, Result};

// Only used initially to store episodes in DB from external source.

#[allow(dead_code)]
pub async fn add_all_pka_youtube_details(state: &Repo, client: &Client) -> Result<()> {
    let all_episodes = pka_episode::all(state).await?;

    let all_details = pka_youtube_details::all(state).await?;

    let missing = all_episodes
        .into_iter()
        .filter(|e| {
            e.youtube_endpoint() != "N/A"
                && all_details
                    .iter()
                    .find(|d| FloatOrd(d.episode_number()) == FloatOrd(e.number()))
                    .is_none()
        })
        .collect::<Vec<PkaEpisode>>();

    let bodies = stream::iter(missing)
        .map(move |ep| async move {
            let res = client
                .get(&ep.youtube_link())
                .send()
                .await
                .expect("Failed to send youtube request");
            (res, ep)
        })
        .buffer_unordered(7);

    let fut = bodies.for_each_concurrent(7, |(res, ep)| async move {
        let data = String::from_utf8(res.bytes().await.expect("Failed to read response").to_vec())
            .expect("Failed to convert response to string");
        let details = get_video_details(&data);

        if let Ok(details) = details {
            let details = details.video_details;

            let db_details = PkaYoutubeDetails::new(
                details.video_id,
                ep.number(),
                details.title,
                details.length_seconds,
                details.average_rating,
            );

            if let Err(e) = pka_youtube_details::insert(state, db_details).await {
                error!("{}", e);
            };
        } else {
            println!("Failed to download details for PKA: {}", ep.number())
        }
    });

    fut.await;

    Ok(())
}

#[allow(dead_code)]
pub async fn add_all_pka_episode_events(state: &Repo) -> Result<()> {
    let dir_name = Path::new(PKA_DESCRIPTIONS_FOLDER);

    let mut res = fs::read_dir(dir_name).await?;

    while let Some(entry) = res.next().await {
        if let Ok(entry) = entry {
            let name = entry
                .file_name()
                .to_str()
                .expect("Failed to convert DirEntry to string")
                .to_owned();

            if name.starts_with("PKA") {
                let data = fs::read_to_string(entry.path())
                    .await
                    .expect("Failed to read file contents.");

                let ep_number: f32 = name
                    .replace("PKA ", "")
                    .replace(".txt", "")
                    .parse()
                    .expect("Failed to convert ep_number to f32");

                match extract_pka_episode_events(ep_number, &data) {
                    Ok(events) => {
                        for event in events.into_iter() {
                            if let Err(e) = pka_event::insert(state, event).await {
                                error!("{}", e);
                            }
                        }
                    }
                    Err(e) => error!("{}", e),
                }
            }
        }
    }

    Ok(())
}

#[allow(dead_code)]
pub async fn download_all_pka_episodes_timelines_pka_info(
    state: &Repo,
    client: &Client,
) -> Result<()> {
    let all_episodes = pka_episode::all(state).await?;

    let all_details = pka_event::all(state).await?;

    let missing = all_episodes
        .into_iter()
        .filter(|e| {
            e.youtube_endpoint() != "N/A"
                && all_details
                    .iter()
                    .find(|d| FloatOrd(d.episode_number()) == FloatOrd(e.number()))
                    .is_none()
        })
        .map(|e| e.number())
        .collect::<Vec<f32>>();

    let dir_name = Path::new(PKA_DESCRIPTIONS_FOLDER);

    if !dir_name.exists() {
        fs::create_dir(dir_name)
            .await
            .expect("Failed to create dir");
    }

    let bodies = stream::iter(missing)
        .map(move |ep| async move {
            let res = client
                .get(
                    format!(
                        "https://www.painkilleralready.info/api/episode.php?episode={}",
                        ep
                    )
                    .parse::<Url>()
                    .expect("Failed to parse youtube_link into URL"),
                )
                .send()
                .await
                .expect("Failed to send painkilleralready.info request");
            (res, ep)
        })
        .buffer_unordered(5);

    let fut = bodies.for_each_concurrent(5, |(res, ep)| async move {
        let api_response = serde_json::from_slice::<PkaInfoRoot>(
            &res.bytes().await.expect("Failed to read response"),
        );

        if let Ok(api_response) = api_response {
            if !api_response.timeline.timestamps.is_empty() {
                let mut file = File::create(format!(
                    "{}/PKA {:<03}.txt",
                    dir_name
                        .to_str()
                        .expect("Failed to convert dir_name to String"),
                    ep
                ))
                .await
                .expect("Failed to create file");

                let formatted_timeline = api_response.timeline.timestamps.into_iter().fold(
                    "".to_owned(),
                    |mut next, curr| {
                        next.push_str(format!("{} - {}\n", curr.hms, curr.value).as_str());
                        next
                    },
                );

                file.write_all(formatted_timeline.as_bytes())
                    .await
                    .expect("Failed to write description to file");

                println!("Downloaded: {}", ep);
            } else {
                println!("Timelines were empty: {}", ep);
            }
        } else {
            println!("Failed to download description for PKA: {}", ep)
        }
    });

    fut.await;

    Ok(())
}

#[allow(dead_code)]
pub async fn download_all_pka_episode_descriptions(state: &Repo, client: &Client) -> Result<()> {
    let mut all_episodes = pka_episode::all(state).await?;

    let dir_name = Path::new(PKA_DESCRIPTIONS_FOLDER);

    all_episodes.retain(|ep| {
        ep.youtube_endpoint() != "N/A"
            && !Path::new(&format!(
                "{}/{}.txt",
                dir_name
                    .to_str()
                    .expect("Failed to convert DirEntry to string"),
                ep.name()
            ))
            .exists()
    });

    if !dir_name.exists() {
        fs::create_dir(dir_name)
            .await
            .expect("Failed to create dir");
    }

    let bodies = stream::iter(all_episodes)
        .map(move |ep| async move {
            let res = client
                .get(&ep.youtube_link())
                .send()
                .await
                .expect("Failed to send youtube request");
            (res, ep)
        })
        .buffer_unordered(7);

    let fut = bodies.for_each_concurrent(7, |(res, ep)| async move {
        let data = String::from_utf8(res.bytes().await.expect("Failed to read response").to_vec())
            .expect("Failed to convert response to string");
        let details = get_video_details(&data);

        if let Ok(details) = details {
            let mut file = File::create(format!(
                "{}/{}.txt",
                dir_name
                    .to_str()
                    .expect("Failed to convert dir_name to String"),
                ep.name()
            ))
            .await
            .expect("Failed to create file");

            file.write_all(details.video_details.short_description.as_bytes())
                .await
                .expect("Failed to write description to file");

            println!("Downloaded: {}", ep.name());
        } else {
            println!("Failed to download description for PKA: {}", ep.number())
        }
    });

    fut.await;

    Ok(())
}

#[allow(dead_code)]
pub async fn add_pka_episodes_from_file(state: &Repo) {
    let f = tokio::fs::read_to_string("./pka_episode_list.json")
        .await
        .expect("Failed to read 'pka_episode_list.json'.");

    let data = serde_json::from_str::<EpisodesFileRoot>(&f)
        .expect("Failed to convert episode list to struct.");

    for ep in data.episodes.into_iter() {
        let upload_date = chrono::naive::NaiveDateTime::parse_from_str(
            &format!("{} 00:00:00", &ep.date_time),
            "%Y-%m-%d %H:%M:%S",
        )
        .expect("Failed to convert upload_date to NaiveDateTime")
        .timestamp();

        let pka_ep = PkaEpisode::new(ep.number, ep.identifier, ep.you_tube, upload_date);

        if let Err(e) = pka_episode::insert(state, pka_ep).await {
            error!("Failed to add [{}] - {}", ep.number, e.to_string());
        }
    }
}

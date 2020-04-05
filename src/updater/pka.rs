use std::sync::Arc;

use chrono::{NaiveTime, Timelike};
use regex::Regex;
use reqwest::{Client, StatusCode};
use tokio::time;
use tokio::time::Duration;

use crate::conduit::{pka_episode, pka_event, pka_youtube_details};
use crate::models::errors::ApiError;
use crate::models::pka_episode::PkaEpisode;
use crate::models::pka_event::PkaEvent;
use crate::models::pka_youtube_details::PkaYoutubeDetails;
use crate::updater::youtube_api::YoutubeAPI;
use crate::YT_API_KEY;
use crate::{Repo, Result};

pub const PKA_DESCRIPTIONS_FOLDER: &str = "PKA-Descriptions";

const WOODY_YOUTUBE_RSA_FEED: &str =
    "https://www.youtube.com/feeds/videos.xml?channel_id=UCIPVJoHb_A5S3kcv3TJlyEg";

pub async fn spawn_get_latest_worker(state: Arc<Repo>) {
    loop {
        info!("Checking for latest episode...");

        if let Err(e) = get_latest_pka_episode_data(&state).await {
            error!("get_latest_worker error: {}", e);
        } else {
            info!("Successfully added latest episode");
        }

        //Check once every 24 hours.
        time::delay_for(Duration::from_secs(86400)).await;
    }
}

pub async fn get_latest_pka_episode_data(state: &Repo) -> Result<()> {
    let latest_episode_number = pka_episode::latest(state).await? + 1.0;
    let latest_episode = format!("PKA {}", latest_episode_number);

    info!("Looking for {}.", latest_episode);

    let client = Client::new();

    // Check RSA feed for latest youtube link
    let res = client.get(WOODY_YOUTUBE_RSA_FEED).send().await?;
    let data = String::from_utf8(res.bytes().await?.to_vec())?;

    let c = feed_rs::parser::parse(data.as_bytes())?;

    let (youtube_link, uploaded) = c
        .entries
        .into_iter()
        .filter(|e| e.title.is_some() && e.published.is_some())
        .find(|e| e.title.as_ref().unwrap().content.contains(&latest_episode))
        .map(|e| {
            (
                e.id.replace("yt:video:", ""),
                e.published.unwrap().timestamp(),
            )
        })
        .ok_or_else(|| ApiError::new("Couldn't find episode", StatusCode::NOT_FOUND))?;

    // Extract data from youtube_link
    let yt_api = YoutubeAPI::new(&YT_API_KEY);

    let details = yt_api.get_video_details(&youtube_link).await?;

    let pka_ep = PkaEpisode::new(
        latest_episode_number,
        latest_episode,
        youtube_link,
        uploaded,
    );

    let events = extract_pka_episode_events(latest_episode_number, &details.snippet.description)?;

    let youtube_details = PkaYoutubeDetails::new(
        details.id,
        latest_episode_number,
        details.snippet.title,
        details.content_details.duration,
    );

    info!("Extracted video details.");

    if let Err(e) = pka_episode::insert(state, pka_ep).await {
        error!("Error adding latest pka_ep: {}", e);
    }

    for evt in events.into_iter() {
        if let Err(e) = pka_event::insert(state, evt).await {
            error!("Error adding latest events: {}", e);
        }
    }

    if let Err(e) = pka_youtube_details::insert(state, youtube_details).await {
        error!("Error adding latest youtube_details: {}", e);
    }

    Ok(())
}

pub fn extract_pka_episode_events(ep_number: f32, data: &str) -> Result<Vec<PkaEvent>> {
    lazy_static! {
        static ref TIMELINE_REGEX: Regex = Regex::new(r"(\d{1,2}:\d{2}:?\d*)(?:\s*-\s*)*\s*(.+)")
            .expect("Failed to create TIMELINE_REGEX.");
        static ref UNPADDED_MINUTE_REGEX: Regex =
            Regex::new(r#"^(\d)(?::)"#).expect("Failed to create UNPADDED_MINUTE_REGEX");
    }

    let mut events = Vec::new();

    for result in TIMELINE_REGEX.captures_iter(data) {
        let mut time_date = result
            .get(1)
            .ok_or_else(|| ApiError::new_internal_error("Failed to get time_date from regex"))?
            .as_str()
            .trim_end_matches(':')
            .to_owned();

        //must ensure format is 00:00:00 ie H:M:S
        match time_date.matches(':').count() {
            1 => {
                // if minute doesn't have a padding 0 then must add it.
                if UNPADDED_MINUTE_REGEX.is_match(&time_date) {
                    time_date = format!("0{}", time_date);
                }
                //then add leading 00:
                time_date = format!("00:{}", time_date);
            }
            2 => {
                if UNPADDED_MINUTE_REGEX.is_match(&time_date) {
                    time_date = format!("0{}", time_date);
                }
            }
            _ => return Err(ApiError::new_internal_error("Unknown timestamp found")),
        };

        let timestamp = NaiveTime::parse_from_str(&time_date, "%H:%M:%S")
            .or_else(|_| {
                Err(ApiError::new_internal_error(
                    "Failed to convert time_date to timestamp",
                ))
            })?
            .num_seconds_from_midnight() as i64;

        let description = result
            .get(2)
            .ok_or_else(|| ApiError::new_internal_error("Failed to get description from regex"))?
            .as_str()
            .trim()
            .replace("’", "'")
            .replace("â€™", "'")
            .replace("â€¦", "…")
            .replace("â€“", "–")
            .replace("â€œ", "“")
            .replace("â€", "”")
            .to_owned();

        let event_id = format!("{:<03}-{}", ep_number, timestamp);

        let event = PkaEvent::new(event_id, ep_number, timestamp, description);
        events.push(event);
    }

    Ok(events)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_pka_description() {
        let data = r#"docid=5BwqArPs5tU\\u0026ei=LwptXoe5LJWnxwLG-YK4Bg\\u0026len=15168\\u0026ns=yt\\u0026plid=AAWg01CvYYmOcuUv\\u0026ver=2\",\"elapsedMediaTimeSeconds\":5},\"youtubeRemarketingUrl\":{\"baseUrl\":\"https:\/\/www.youtube.com\/pagead\/viewthroughconversion\/962985656\/?backend=innertube\\u0026cname=1\\u0026cver=2_20200311\\u0026foc_id=IPVJoHb_A5S3kcv3TJlyEg\\u0026label=followon_view\\u0026ptype=no_rmkt\\u0026random=553324739\",\"elapsedMediaTimeSeconds\":0}},\"videoDetails\":{\"videoId\":\"5BwqArPs5tU\",\"title\":\"PKA 282 w Cinema Sins   Success, Prison Surival, and Manhood\",\"lengthSeconds\":\"15168\",\"keywords\":[\"commentary\",\"woodysgamertag\",\"mail monday\",\"minecraft\",\"pka\",\"painkiller\",\"already\",\"podcast\",\"diy\",\"vlog\",\"blog\",\"cinemasins\",\"fpsrussia\",\"fps russia\",\"cinema sins\",\"captain america\",\"success\",\"finance\",\"everything wrong with\",\"pixar\"],\"channelId\":\"UCIPVJoHb_A5S3kcv3TJlyEg\",\"isOwnerViewing\":false,\"shortDescription\":\"http:\/\/Scorebig.com - Promo code “PKA”\\nhttp:\/\/HelixSleep.com - Promo code “PKA”\\nhttp:\/\/Headspace.com\/PKA\\nhttp:\/\/squarespace.com\/PKA\\nhttp:\/\/DollarShaveClub.com\/PKA\\nGuest: https:\/\/www.youtube.com\/cinemasins\\nMerch: http:\/\/painkilleralready.net\\nhttp:\/\/patreon.com\/pka\\nPKA on iTunes: http:\/\/bit.ly\/PKAOniTunes\\n0:05 Ad Read: (Scorebig, DSC, Helix Sleep, HeadSpace, Squarespace)0:55 Welcoming the guest Jeremy from CinemaSins\\n1:41 How Kyle’s doing out in LA\\n5:45 Hockey Tak: Jeremy is a fan, Nashville Predators and the Blues\\n12:30 YouTube and Reddit comments, Family watching your content\\n17:26 Back to Hockey\\n18:12 The Trayvon Martin murder weapon is up for sale\\n29:55 First experience with guns\\n31:53 What do you do in the Zombie apocalypse\\n32:48 Crossbows and longbows\\n43:22 Periscope suicide\\n46:49 Amanda Todd\\n48:09 Internet reputation, Annotations\\n54:00 PKA Knives are sold out\\n55:08 Hockey Update\\n1:00:29 Politics Talk: Trump and Immigration\\n1:23:26 Ad Read: (DollarShaveClub.com\/PKA) \\u0026 (HelixSleep.com\/PKA)\\n1:27:27 60 Days In\\n1:37:05 Slingblade, Forrest Gump, The Godfather\\n1:43:18 AMA Question: Favorite youtuber and genre\\n1:49:44 Company of Heroes\\n1:52:00 Civilization 6 Trailer\\n1:56:42 Call of Duty\\n2:02:52 Deadpool review video with Ryan Reynolds\\n2:06:30 Woody has been watching vlog channels\\n2:13:00 (Cinemasins)Jeremy leaves\\n2:16:10 Junkyard Wars and Channel video ideas\\n2:22:56 Ad Read: (Scorebig.com\/PKA)\\n2:25:56 What Movie has had the biggest effect on your life\\n2:34:37 How Woody would change UFC for the better\\n2:37:58 Connor McGregor vs Floyd Mayweather\\n2:40:00 Floyd Mayweather talk\\n2:48:40 AMA question- Is it normal to hate work?\\n2:50:25 The best job you can get - Kyle\\n2:51:50 Woody’s passion changes\\n2:57:05 Fatty Bootcamp Program idea\\n2:58:30 Kyle’s coach flight and the fatty on board\\n3:02:57 Have you ever realized someone is extremely intelligent?\\n3:11:00 Gasoline Pumps are dangerous\\n3:14:20 Musical instrument talk, woody sucks at guitar\\n3:18:40 Languages talk\\n3:20:49 Kyle’s Lithuanian buddies aruging\\n3:21:34 Ad read (Squarespace)(Total War: Warhammer)\\n3:30:25 Woody’s greatest rap of all time (pka 178)\\n3:33:38 M. Night Shamalayn’s new movie topic\\n3:38:00 More Movie talk\\n3:41:10 Kyle wants to be an Astronaut\\n3:42:15 Gravitational Boogers talk\\n3:46:10 What does it mean to be a Man? (ama question)\\n3:55:00 Infrastructure is key\\n3:56:30 Money talk\\n4:01:00 Obama's confusing speech and his constant use of liberal buzzwords\\n4:03:00 Woody4president\\n4:10:00 Trump investment talk. just do it.\",\"isCrawlable\":true,\"thumbnail\":{\"thumbnails\":[{\"url\":\"https:\/\/i.ytimg.com\/vi\/5BwqArPs5tU\/hqdefault.jpg?sqp=-oaymwEYCKgBEF5IVfKriqkDCwgBFQAAiEIYAXAB\\u0026rs=AOn4CLD_0Y5a-UlX964InTnOORa4N2n27Q\",\"width\":168,\"height\":94},{\"url\":\"https:\/\/i.ytimg.com\/vi\/5BwqArPs5tU\/hqdefault.jpg?sqp=-oaymwEYCMQBEG5IVfKriqkDCwgBFQAAiEIYAXAB\\u0026rs=AOn4CLARRVQZy649zjr1v5FSH69Yej_0zg\",\"width\":196,\"height\":110},{\"url\":\"https:\/\/i.ytimg.com\/vi\/5BwqArPs5tU\/hqdefault.jpg?sqp=-oaymwEZCPYBEIoBSFXyq4qpAwsIARUAAIhCGAFwAQ==\\u0026rs=AOn4CLA5GfaVun3Db3bb66_lRJstQ2lWKw\",\"width\":246,\"height\":138},{\"url\":\"https:\/\/i.ytimg.com\/vi\/5BwqArPs5tU\/hqdefault.jpg?sqp=-oaymwEZCNACELwBSFXyq4qpAwsIARUAAIhCGAFwAQ==\\u0026rs=AOn4CLBxBgeG0fn9x00qRdJGXK-VgF8JiA\",\"width\":336,\"height\":188},{\"url\":\"https:\/\/i.ytimg.com\/vi_webp\/5BwqArPs5tU\/maxresdefault.webp\",\"width\":1920,\"height\":1080}]},\"averageRating\":4.7478862,\"allowRatings\":true,\"viewCount\":\"160076\",\"author\":\"WoodysGamertag\",\"isPrivate\":false,\"isUnpluggedCorpus\":false,\"isLiveContent\":false},\"annotations\":[{\"playerAnnotationsExpandedRenderer\":{\"featuredChannel\":{\"startTimeMs\":\"0\",\"endTimeMs\":\"15168000\",\"watermark\":{\"thumbnails\":[{\"url\":\"https:\/\/i.ytimg.com\/an\/IPVJoHb_A5S3kcv3TJlyEg\/featured_channel.jpg?v=525fe85e\",\"width\":40,\"height\":40}]},\"trackingParams\":\"CAEQ8zciEwjHjLuFtZroAhWV01EKHca8AGc=\",\"navigationEndpoint\":{\"clickTrackingParams\":\"CAEQ8zciEwjHjLuFtZroAhWV01EKHca8AGcyAml2\",\"commandMetadata\":{\"webCommandMetadata\":{\"url\":\"\/channel\/UCIPVJoHb_A5S3kcv3TJlyEg\",\"webPageType\":\"WEB_PAGE_TYPE_BROWSE\",\"rootVe\":3611}},\"browseEndpoint\":{\"browseId\":\"UCIPVJoHb_A5S3kcv3TJlyEg\"}},\"channelName\":\"WoodysGamertag\",\"subscribeButton\":{\"subscribeButtonRenderer\":{\"buttonText\":{\"runs\":[{\"text\":\"SUBSCRIBED\"}]},\"subscribed\":true,\"enabled\":true,\"type\":\"FREE\",\"channelId\":\"UCIPVJoHb_A5S3kcv3TJlyEg\",\"showPreferences\":false,\"subscribedButtonText\":{\"runs\":[{\"text\":\"SUBSCRIBED\"}]},\"#;
        let description_str = get_video_details(data).expect("Failed to get description");

        dbg!(description_str);
    }
}

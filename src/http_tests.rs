//! The HTTP application end to end, from request to JSON, against an
//! in-memory database built from the migrations and filled through the same
//! save the updater uses.

use std::str::FromStr;

use axum::Router;
use axum::body::{Body, Bytes, to_bytes};
use axum::http::{HeaderMap, Request, StatusCode, header};
use serde_json::{Value, json};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use tower::ServiceExt;

use crate::Repo;
use crate::app_state::AppState;
use crate::catalog::search::GROUPS_PER_PAGE;
use crate::catalog::{self, SharedCatalog};
use crate::models::pka_episode::PkaEpisode;
use crate::models::pka_event::PkaEvent;
use crate::models::pka_youtube_details::PkaYoutubeDetails;
use crate::routes;
use crate::updater::pka::save_episode;

const SCHEMA: [&str; 4] = [
    include_str!("../migrations/20200301124334_pka_episode.sql"),
    include_str!("../migrations/20200301130012_pka_guest.sql"),
    include_str!("../migrations/20200314142852_pka_event.sql"),
    include_str!("../migrations/20200329181719_pka_youtube_details.sql"),
];

// 2026-10-01T00:00:00Z
const UPLOADED: i64 = 1_790_812_800;

async fn add_episode(repo: &Repo, number: f32, title: &str, moments: &[(i32, &str)]) {
    let video_id = format!("video-{number}");
    let events = moments
        .iter()
        .map(|&(timestamp, description)| {
            PkaEvent::new(
                format!("{number}-{timestamp}").into(),
                number,
                timestamp,
                description.into(),
                60,
                UPLOADED,
            )
        })
        .collect();

    save_episode(
        repo,
        PkaEpisode::new(
            number,
            format!("PKA {number}").into(),
            video_id.clone().into(),
            UPLOADED + (number as i64) * 86_400,
        ),
        PkaYoutubeDetails::new(video_id.into(), number, title.into(), 3 * 3600),
        events,
    )
    .await
    .expect("Failed to save a test episode");
}

async fn database() -> Repo {
    // One connection that never closes: each connection to `:memory:` is a
    // Database of its own.
    let repo = SqlitePoolOptions::new()
        .max_connections(1)
        .idle_timeout(None)
        .max_lifetime(None)
        .connect_with(SqliteConnectOptions::from_str("sqlite::memory:").unwrap())
        .await
        .unwrap();

    for sql in SCHEMA {
        sqlx::raw_sql(sql).execute(&repo).await.unwrap();
    }

    add_episode(
        &repo,
        825.0,
        "PKA 825 W/ Harley",
        &[
            (30, "Intro"),
            (600, "Why Kyle loves Hellraiser movies"),
            (1200, "Taylor's bible story"),
        ],
    )
    .await;
    add_episode(
        &repo,
        524.1,
        "Part 1 Confederate Talk",
        &[(10, "Taylor watches Hellraiser for the first time")],
    )
    .await;
    add_episode(
        &repo,
        400.0,
        "PKA 400 w/ Tucker",
        &[(90, "Kyle and Taylor argue")],
    )
    .await;

    repo
}

async fn app(repo: &Repo) -> (Router, SharedCatalog) {
    let catalog = SharedCatalog::new(catalog::load(repo).await.unwrap());

    (routes::app(AppState::new(catalog.clone()), false), catalog)
}

async fn get(app: &Router, uri: &str) -> (StatusCode, HeaderMap, Bytes) {
    let response = app
        .clone()
        .oneshot(Request::get(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();

    (status, headers, body)
}

async fn get_json(app: &Router, uri: &str) -> (StatusCode, HeaderMap, Value) {
    let (status, headers, body) = get(app, uri).await;

    (status, headers, serde_json::from_slice(&body).unwrap())
}

fn cache_control(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::CACHE_CONTROL)
        .map(|value| value.to_str().unwrap())
}

#[tokio::test]
async fn test_episode_list_is_newest_first_and_cacheable() {
    let (app, _) = app(&database().await).await;
    let (status, headers, body) = get_json(&app, "/api/v1/episodes").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers[header::CONTENT_TYPE], "application/json");
    assert_eq!(cache_control(&headers), Some("public, max-age=300"));
    assert_eq!(
        body["data"][0],
        json!({
            "episodeNumber": 825.0,
            "uploadDate": UPLOADED + 825 * 86_400,
            "title": "PKA 825 W/ Harley",
            "lengthSeconds": 10_800,
            "videoId": "video-825",
            "eventCount": 3,
        })
    );

    let numbers = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|episode| episode["episodeNumber"].as_f64().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(numbers, vec![825.0, 524.1, 400.0]);
}

#[tokio::test]
async fn test_episode_details_include_moments_in_order() {
    let (app, _) = app(&database().await).await;
    let (status, _, body) = get_json(&app, "/api/v1/episodes/825").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["episode"]["number"], 825.0);
    assert_eq!(body["data"]["youtubeDetails"]["videoId"], "video-825");
    // Internal columns stay internal.
    assert!(body["data"]["episode"].get("youtubeLink").is_none());

    let timestamps = body["data"]["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|event| event["timestamp"].as_i64().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(timestamps, vec![30, 600, 1200]);

    let (status, _, body) = get_json(&app, "/api/v1/episodes/524.1").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["events"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_unknown_and_malformed_episodes_are_errors_that_are_not_cached() {
    let (app, _) = app(&database().await).await;
    let (status, headers, body) = get_json(&app, "/api/v1/episodes/9999").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], 404);
    assert_eq!(cache_control(&headers), None);

    let (status, _, _) = get_json(&app, "/api/v1/episodes/nine").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, _, body) = get_json(&app, "/api/v1/nothing-here").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["message"], "Page not found.");
}

#[tokio::test]
async fn test_latest_and_youtube_link() {
    let (app, _) = app(&database().await).await;
    let (_, _, latest) = get_json(&app, "/api/v1/episodes/latest").await;

    assert_eq!(latest["data"]["episode"]["number"], 825.0);

    let (status, _, link) = get_json(&app, "/api/v1/episodes/400/youtube-link").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(link["data"], "video-400");
}

#[tokio::test]
async fn test_random_picks_are_never_cached() {
    let (app, _) = app(&database().await).await;

    for _ in 0..10 {
        let (status, headers, event) = get_json(&app, "/api/v1/events/random").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(cache_control(&headers), Some("no-store"));
        // Never the intro.
        assert_ne!(event["data"]["description"], "Intro");
    }

    let (status, headers, episode) = get_json(&app, "/api/v1/episodes/random").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(cache_control(&headers), Some("no-store"));
    assert!(episode["data"]["episode"]["number"].is_number());
}

#[tokio::test]
async fn test_search_groups_moments_by_episode() {
    let (app, _) = app(&database().await).await;
    let (status, headers, body) = get_json(&app, "/api/v1/search?q=taylor").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(cache_control(&headers), Some("public, max-age=300"));
    assert_eq!(body["data"]["totalMoments"], 3);
    assert_eq!(body["data"]["totalEpisodes"], 3);
    assert_eq!(body["data"]["groups"][0]["episode"]["episodeNumber"], 825.0);
    assert_eq!(
        body["data"]["groups"][0]["hits"][0]["description"],
        "Taylor's bible story"
    );
    assert_eq!(body["data"]["nextPage"], Value::Null);

    // Curly apostrophes, as phones type them, find straight ones.
    let (_, _, body) = get_json(&app, "/api/v1/search?q=taylor%E2%80%99s").await;

    assert_eq!(body["data"]["totalMoments"], 1);

    let (_, _, body) = get_json(&app, "/api/v1/search?q=kyle&sort=matches").await;

    assert_eq!(body["data"]["totalEpisodes"], 2);
}

#[tokio::test]
async fn test_search_pages_and_rejects_bad_requests() {
    let repo = database().await;

    for number in 1..=(GROUPS_PER_PAGE as i32 + 2) {
        add_episode(&repo, number as f32, "Filler", &[(5, "Woody's garage")]).await;
    }

    let (app, _) = app(&repo).await;
    let (_, _, first) = get_json(&app, "/api/v1/search?q=garage").await;
    let (_, _, second) = get_json(&app, "/api/v1/search?q=garage&page=1").await;

    assert_eq!(
        first["data"]["groups"].as_array().unwrap().len(),
        GROUPS_PER_PAGE
    );
    assert_eq!(first["data"]["nextPage"], 1);
    assert_eq!(second["data"]["groups"].as_array().unwrap().len(), 2);
    assert_eq!(second["data"]["nextPage"], Value::Null);

    let (status, headers, _) = get_json(&app, "/api/v1/search?q=garage&sort=oldest").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(cache_control(&headers), None);

    let long = "a".repeat(201);
    let (status, _, _) = get_json(&app, &format!("/api/v1/search?q={long}")).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_related_moments_come_from_other_episodes() {
    let (app, _) = app(&database().await).await;
    let (status, _, body) =
        get_json(&app, "/api/v1/events/related?episode=825&timestamp=600").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["data"],
        json!([{
            "episode": {
                "episodeNumber": 524.1,
                "uploadDate": UPLOADED + 524 * 86_400,
                "title": "Part 1 Confederate Talk",
                "lengthSeconds": 10_800,
                "videoId": "video-524.1",
                "eventCount": 1,
            },
            "timestamp": 10,
            "description": "Taylor watches Hellraiser for the first time",
            "lengthSeconds": 60,
            "sharedTerms": ["hellraiser"],
        }])
    );

    let (status, _, body) =
        get_json(&app, "/api/v1/events/related?episode=825&timestamp=601").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"], json!([]));
}

#[tokio::test]
async fn test_sitemap_and_robots() {
    let (app, _) = app(&database().await).await;
    let (status, headers, body) = get(&app, "/sitemap.xml").await;
    let xml = String::from_utf8(body.to_vec()).unwrap();

    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers[header::CONTENT_TYPE], "application/xml");
    assert_eq!(cache_control(&headers), Some("public, max-age=3600"));
    assert!(xml.contains("<loc>https://www.pkaindex.com/watch/524.1</loc>"));
    assert_eq!(xml.matches("<url>").count(), 4);

    let (status, _, body) = get(&app, "/robots.txt").await;

    assert_eq!(status, StatusCode::OK);
    assert!(
        String::from_utf8(body.to_vec())
            .unwrap()
            .contains("Sitemap:")
    );
}

#[tokio::test]
async fn test_openapi_only_when_exposed() {
    let repo = database().await;
    let catalog = SharedCatalog::new(catalog::load(&repo).await.unwrap());
    let hidden = routes::app(AppState::new(catalog.clone()), false);
    let exposed = routes::app(AppState::new(catalog), true);

    assert_eq!(get(&hidden, "/openapi.json").await.0, StatusCode::NOT_FOUND);

    let (status, _, doc) = get_json(&exposed, "/openapi.json").await;

    assert_eq!(status, StatusCode::OK);
    assert!(doc["paths"]["/api/v1/search"].is_object());
}

#[tokio::test]
async fn test_a_saved_episode_appears_after_a_refresh() {
    let repo = database().await;
    let (app, catalog) = app(&repo).await;

    add_episode(&repo, 826.0, "PKA 826", &[(15, "Kyle's new bike")]).await;

    // Served from the snapshot until it's rebuilt.
    let (_, _, before) = get_json(&app, "/api/v1/episodes/latest").await;

    assert_eq!(before["data"]["episode"]["number"], 825.0);

    catalog::refresh(&repo, &catalog).await.unwrap();

    let (_, _, after) = get_json(&app, "/api/v1/episodes/latest").await;
    let (_, _, search) = get_json(&app, "/api/v1/search?q=bike").await;

    assert_eq!(after["data"]["episode"]["number"], 826.0);
    assert_eq!(search["data"]["totalMoments"], 1);
}

#[tokio::test]
async fn test_a_failed_save_leaves_nothing_behind() {
    let repo = database().await;

    // The video is already stored under episode 400, so the last insert fails.
    let result = save_episode(
        &repo,
        PkaEpisode::new(827.0, "PKA 827".into(), "video-400".into(), UPLOADED),
        PkaYoutubeDetails::new("video-400".into(), 827.0, "PKA 827".into(), 3600),
        vec![PkaEvent::new(
            "827-5".into(),
            827.0,
            5,
            "Kyle".into(),
            60,
            UPLOADED,
        )],
    )
    .await;

    assert!(result.is_err());

    let episodes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM pka_episode WHERE number = 827")
        .fetch_one(&repo)
        .await
        .unwrap();
    let events: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM pka_event WHERE episode_number = 827")
            .fetch_one(&repo)
            .await
            .unwrap();

    assert_eq!((episodes, events), (0, 0));
}

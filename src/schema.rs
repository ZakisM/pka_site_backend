table! {
    pka_episode (number) {
        number -> Float,
        name -> Text,
        youtube_link -> Text,
        upload_date -> BigInt,
    }
}

table! {
    pka_event (event_id) {
        event_id -> Text,
        episode_number -> Float,
        timestamp -> Integer,
        description -> Text,
        length_seconds -> Integer,
        upload_date -> BigInt,
    }
}

table! {
    pka_guest (name) {
        name -> Text,
        episode_number -> Float,
    }
}

table! {
    pka_youtube_details (video_id) {
        video_id -> Text,
        episode_number -> Float,
        title -> Text,
        length_seconds -> Integer,
    }
}

joinable!(pka_event -> pka_episode (episode_number));
joinable!(pka_guest -> pka_episode (episode_number));
joinable!(pka_youtube_details -> pka_episode (episode_number));

allow_tables_to_appear_in_same_query!(pka_episode, pka_event, pka_guest, pka_youtube_details,);

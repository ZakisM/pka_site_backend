use flatbuffers::{FlatBufferBuilder, WIPOffset};

use crate::flatbuffers::pka_event::pka_event_search_results_generated::{
    finish_all_pka_event_search_results_fb_buffer, root_as_all_pka_event_search_results_fb,
    AllPkaEventSearchResultsFb, AllPkaEventSearchResultsFbArgs, PkaEventSearchResultFb,
    PkaEventSearchResultFbArgs,
};
use crate::models::pka_event::PkaEvent;

pub mod pka_event_search_results_generated;

pub fn flatbuff_from_pka_events(events: Vec<&PkaEvent>) -> Vec<u8> {
    let mut bldr = FlatBufferBuilder::with_capacity(events.capacity());

    let events_vec: Vec<WIPOffset<PkaEventSearchResultFb>> = events
        .iter()
        .map(|e| {
            let e_arg = PkaEventSearchResultFbArgs {
                episode_number: e.episode_number(),
                timestamp: e.timestamp(),
                description: Some(bldr.create_string(e.description())),
                length_seconds: e.length_seconds(),
                upload_date: e.upload_date(),
            };

            PkaEventSearchResultFb::create(&mut bldr, &e_arg)
        })
        .collect();

    let all_events_arg = AllPkaEventSearchResultsFbArgs {
        results: Some(bldr.create_vector(&events_vec)),
    };

    let all_events = AllPkaEventSearchResultsFb::create(&mut bldr, &all_events_arg);

    finish_all_pka_event_search_results_fb_buffer(&mut bldr, all_events);

    let finished_data = bldr.finished_data();

    finished_data.to_vec()
}

#[allow(dead_code)]
fn read_event(buf: &[u8], index: usize) -> (f32, i32, &str, i32, i64) {
    let e = root_as_all_pka_event_search_results_fb(buf);
    let results = e.unwrap();
    let first_event = results.results().unwrap().get(index);

    (
        first_event.episode_number(),
        first_event.timestamp(),
        first_event.description().unwrap(),
        first_event.length_seconds(),
        first_event.upload_date(),
    )
}

#[cfg(test)]
mod tests {
    use compact_str::ToCompactString;

    use super::*;

    #[test]
    fn read_events() {
        let first_event = PkaEvent::new(
            "488-1234".to_compact_string(),
            488.0,
            1234,
            "Zak joins the first show.".to_compact_string(),
            10,
            1372377600,
        );

        let second_event = PkaEvent::new(
            "489-5678".to_compact_string(),
            489.0,
            5678,
            "Zak joins the second show.".to_compact_string(),
            300,
            1572377600,
        );

        let all_events = vec![&first_event, &second_event];

        let fb = flatbuff_from_pka_events(all_events);

        assert_eq!(
            read_event(&fb, 0),
            (488.0, 1234, "Zak joins the first show.", 10, 1372377600)
        );
        assert_eq!(
            read_event(&fb, 1),
            (489.0, 5678, "Zak joins the second show.", 300, 1572377600)
        );
    }
}

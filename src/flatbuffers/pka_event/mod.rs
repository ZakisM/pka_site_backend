use flatbuffers::{FlatBufferBuilder, WIPOffset};

use crate::flatbuffers::pka_event::pka_event_search_results_generated::{
    finish_all_pka_event_search_results_fb_buffer, get_root_as_all_pka_event_search_results_fb,
    AllPkaEventSearchResultsFb, AllPkaEventSearchResultsFbArgs, PkaEventSearchResultFb,
    PkaEventSearchResultFbArgs,
};
use crate::models::search::PkaEventSearchResult;

pub mod pka_event_search_results_generated;

pub fn flatbuff_from_pka_events(events: Vec<PkaEventSearchResult>) -> Vec<u8> {
    let mut bldr = FlatBufferBuilder::new();
    let mut bytes: Vec<u8> = Vec::new();

    let events_vec: Vec<WIPOffset<PkaEventSearchResultFb>> = events
        .into_iter()
        .map(|e| {
            let e_arg = PkaEventSearchResultFbArgs {
                episode_number: e.episode_number(),
                timestamp: e.timestamp(),
                description: Some(bldr.create_string(e.description())),
                length_seconds: e.length_seconds(),
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
    bytes.extend_from_slice(finished_data);

    bytes
}

#[allow(dead_code)]
fn read_event(buf: &[u8], index: usize) -> (f32, i32, &str, i32) {
    let e = get_root_as_all_pka_event_search_results_fb(buf);
    let results = e.results().unwrap();
    let first_event = results.get(index);

    (
        first_event.episode_number(),
        first_event.timestamp(),
        first_event.description().unwrap(),
        first_event.length_seconds(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_events() {
        let all_events = vec![
            PkaEventSearchResult::new(488.0, 1234, "Zak joins the first show.", 10),
            PkaEventSearchResult::new(489.0, 5678, "Zak joins the second show.", 300),
        ];

        let fb = flatbuff_from_pka_events(all_events);

        assert_eq!(
            read_event(&fb, 0),
            (488.0, 1234, "Zak joins the first show.", 10)
        );
        assert_eq!(
            read_event(&fb, 1),
            (489.0, 5678, "Zak joins the second show.", 300)
        );
    }
}

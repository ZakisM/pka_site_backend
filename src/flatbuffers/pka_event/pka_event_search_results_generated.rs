// automatically generated by the FlatBuffers compiler, do not modify
extern crate flatbuffers;

use self::flatbuffers::InvalidFlatbuffer;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PkaEventSearchResultFb<'a> {
    pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for PkaEventSearchResultFb<'a> {
    type Inner = PkaEventSearchResultFb<'a>;
    #[inline]
    fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self {
            _tab: flatbuffers::Table { buf, loc },
        }
    }
}

impl<'a> PkaEventSearchResultFb<'a> {
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
        args: &'args PkaEventSearchResultFbArgs<'args>,
    ) -> flatbuffers::WIPOffset<PkaEventSearchResultFb<'bldr>> {
        let mut builder = PkaEventSearchResultFbBuilder::new(_fbb);
        builder.add_upload_date(args.upload_date);
        builder.add_length_seconds(args.length_seconds);
        if let Some(x) = args.description {
            builder.add_description(x);
        }
        builder.add_timestamp(args.timestamp);
        builder.add_episode_number(args.episode_number);
        builder.finish()
    }

    pub const VT_EPISODE_NUMBER: flatbuffers::VOffsetT = 4;
    pub const VT_TIMESTAMP: flatbuffers::VOffsetT = 6;
    pub const VT_DESCRIPTION: flatbuffers::VOffsetT = 8;
    pub const VT_LENGTH_SECONDS: flatbuffers::VOffsetT = 10;
    pub const VT_UPLOAD_DATE: flatbuffers::VOffsetT = 12;

    #[inline]
    pub fn episode_number(&self) -> f32 {
        self._tab
            .get::<f32>(PkaEventSearchResultFb::VT_EPISODE_NUMBER, Some(0.0))
            .unwrap()
    }
    #[inline]
    pub fn timestamp(&self) -> i32 {
        self._tab
            .get::<i32>(PkaEventSearchResultFb::VT_TIMESTAMP, Some(0))
            .unwrap()
    }
    #[inline]
    pub fn description(&self) -> Option<&'a str> {
        self._tab
            .get::<flatbuffers::ForwardsUOffset<&str>>(PkaEventSearchResultFb::VT_DESCRIPTION, None)
    }
    #[inline]
    pub fn length_seconds(&self) -> i32 {
        self._tab
            .get::<i32>(PkaEventSearchResultFb::VT_LENGTH_SECONDS, Some(0))
            .unwrap()
    }
    #[inline]
    pub fn upload_date(&self) -> i64 {
        self._tab
            .get::<i64>(PkaEventSearchResultFb::VT_UPLOAD_DATE, Some(0))
            .unwrap()
    }
}

pub struct PkaEventSearchResultFbArgs<'a> {
    pub episode_number: f32,
    pub timestamp: i32,
    pub description: Option<flatbuffers::WIPOffset<&'a str>>,
    pub length_seconds: i32,
    pub upload_date: i64,
}

impl<'a> Default for PkaEventSearchResultFbArgs<'a> {
    #[inline]
    fn default() -> Self {
        PkaEventSearchResultFbArgs {
            episode_number: 0.0,
            timestamp: 0,
            description: None,
            length_seconds: 0,
            upload_date: 0,
        }
    }
}

pub struct PkaEventSearchResultFbBuilder<'a: 'b, 'b> {
    fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
    start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}

impl<'a: 'b, 'b> PkaEventSearchResultFbBuilder<'a, 'b> {
    #[inline]
    pub fn add_episode_number(&mut self, episode_number: f32) {
        self.fbb_.push_slot::<f32>(
            PkaEventSearchResultFb::VT_EPISODE_NUMBER,
            episode_number,
            0.0,
        );
    }
    #[inline]
    pub fn add_timestamp(&mut self, timestamp: i32) {
        self.fbb_
            .push_slot::<i32>(PkaEventSearchResultFb::VT_TIMESTAMP, timestamp, 0);
    }
    #[inline]
    pub fn add_description(&mut self, description: flatbuffers::WIPOffset<&'b str>) {
        self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(
            PkaEventSearchResultFb::VT_DESCRIPTION,
            description,
        );
    }
    #[inline]
    pub fn add_length_seconds(&mut self, length_seconds: i32) {
        self.fbb_
            .push_slot::<i32>(PkaEventSearchResultFb::VT_LENGTH_SECONDS, length_seconds, 0);
    }
    #[inline]
    pub fn add_upload_date(&mut self, upload_date: i64) {
        self.fbb_
            .push_slot::<i64>(PkaEventSearchResultFb::VT_UPLOAD_DATE, upload_date, 0);
    }
    #[inline]
    pub fn new(
        _fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>,
    ) -> PkaEventSearchResultFbBuilder<'a, 'b> {
        let start = _fbb.start_table();
        PkaEventSearchResultFbBuilder {
            fbb_: _fbb,
            start_: start,
        }
    }
    #[inline]
    pub fn finish(self) -> flatbuffers::WIPOffset<PkaEventSearchResultFb<'a>> {
        let o = self.fbb_.end_table(self.start_);
        flatbuffers::WIPOffset::new(o.value())
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AllPkaEventSearchResultsFb<'a> {
    pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for AllPkaEventSearchResultsFb<'a> {
    type Inner = AllPkaEventSearchResultsFb<'a>;
    #[inline]
    fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self {
            _tab: flatbuffers::Table { buf, loc },
        }
    }
}

impl<'a> flatbuffers::Verifiable for AllPkaEventSearchResultsFb<'a> {
    #[inline]
    fn run_verifier(
        v: &mut flatbuffers::Verifier,
        pos: usize,
    ) -> Result<(), flatbuffers::InvalidFlatbuffer> {
        i8::run_verifier(v, pos)
    }
}

impl<'a> AllPkaEventSearchResultsFb<'a> {
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
        args: &'args AllPkaEventSearchResultsFbArgs<'args>,
    ) -> flatbuffers::WIPOffset<AllPkaEventSearchResultsFb<'bldr>> {
        let mut builder = AllPkaEventSearchResultsFbBuilder::new(_fbb);
        if let Some(x) = args.results {
            builder.add_results(x);
        }
        builder.finish()
    }

    pub const VT_RESULTS: flatbuffers::VOffsetT = 4;

    #[inline]
    pub fn results(
        &self,
    ) -> Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<PkaEventSearchResultFb<'a>>>>
    {
        self._tab.get::<flatbuffers::ForwardsUOffset<
            flatbuffers::Vector<flatbuffers::ForwardsUOffset<PkaEventSearchResultFb<'a>>>,
        >>(AllPkaEventSearchResultsFb::VT_RESULTS, None)
    }
}

pub struct AllPkaEventSearchResultsFbArgs<'a> {
    pub results: Option<
        flatbuffers::WIPOffset<
            flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<PkaEventSearchResultFb<'a>>>,
        >,
    >,
}

impl<'a> Default for AllPkaEventSearchResultsFbArgs<'a> {
    #[inline]
    fn default() -> Self {
        AllPkaEventSearchResultsFbArgs { results: None }
    }
}

pub struct AllPkaEventSearchResultsFbBuilder<'a: 'b, 'b> {
    fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
    start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}

impl<'a: 'b, 'b> AllPkaEventSearchResultsFbBuilder<'a, 'b> {
    #[inline]
    pub fn add_results(
        &mut self,
        results: flatbuffers::WIPOffset<
            flatbuffers::Vector<'b, flatbuffers::ForwardsUOffset<PkaEventSearchResultFb<'b>>>,
        >,
    ) {
        self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(
            AllPkaEventSearchResultsFb::VT_RESULTS,
            results,
        );
    }
    #[inline]
    pub fn new(
        _fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>,
    ) -> AllPkaEventSearchResultsFbBuilder<'a, 'b> {
        let start = _fbb.start_table();
        AllPkaEventSearchResultsFbBuilder {
            fbb_: _fbb,
            start_: start,
        }
    }
    #[inline]
    pub fn finish(self) -> flatbuffers::WIPOffset<AllPkaEventSearchResultsFb<'a>> {
        let o = self.fbb_.end_table(self.start_);
        flatbuffers::WIPOffset::new(o.value())
    }
}

#[inline]
pub fn get_root_as_all_pka_event_search_results_fb<'a>(
    buf: &'a [u8],
) -> Result<AllPkaEventSearchResultsFb<'a>, InvalidFlatbuffer> {
    flatbuffers::root::<AllPkaEventSearchResultsFb<'a>>(buf)
}

#[inline]
pub fn finish_all_pka_event_search_results_fb_buffer<'a, 'b>(
    fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>,
    root: flatbuffers::WIPOffset<AllPkaEventSearchResultsFb<'a>>,
) {
    fbb.finish(root, None);
}

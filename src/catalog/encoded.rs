//! A body that is the same for every request, rendered once per snapshot and
//! kept in each encoding a browser may ask for. Compressing it once allows
//! Brotli's best setting, which per-request compression can't afford: the
//! episode list comes out about a fifth smaller than at the default.

use std::io::Write;

use axum::body::Bytes;
use axum::http::{HeaderMap, HeaderValue, header};
use flate2::Compression;
use flate2::write::GzEncoder;

const BROTLI_QUALITY: u32 = 11;
// Brotli's largest standard window, so repeats across the whole list count.
const BROTLI_WINDOW: u32 = 22;
const BROTLI_BUFFER: usize = 4096;

#[derive(Clone, Default)]
pub struct Encoded {
    identity: Bytes,
    br: Bytes,
    gzip: Bytes,
}

impl Encoded {
    pub fn new(identity: Vec<u8>) -> anyhow::Result<Self> {
        let mut br =
            brotli::CompressorWriter::new(Vec::new(), BROTLI_BUFFER, BROTLI_QUALITY, BROTLI_WINDOW);

        br.write_all(&identity)?;

        let mut gzip = GzEncoder::new(Vec::new(), Compression::best());

        gzip.write_all(&identity)?;

        Ok(Self {
            br: br.into_inner().into(),
            gzip: gzip.finish()?.into(),
            identity: identity.into(),
        })
    }

    /// The best encoding the request accepts, with its `Content-Encoding`.
    pub fn negotiate(&self, request: &HeaderMap) -> (Option<HeaderValue>, Bytes) {
        if accepts(request, "br") {
            (Some(HeaderValue::from_static("br")), self.br.clone())
        } else if accepts(request, "gzip") {
            (Some(HeaderValue::from_static("gzip")), self.gzip.clone())
        } else {
            (None, self.identity.clone())
        }
    }
}

/// Whether `Accept-Encoding` lists `coding`, and not as `q=0` (refused).
fn accepts(request: &HeaderMap, coding: &str) -> bool {
    request
        .get_all(header::ACCEPT_ENCODING)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .any(|item| {
            let mut parts = item.split(';');
            let named = parts
                .next()
                .is_some_and(|name| name.trim().eq_ignore_ascii_case(coding));
            let refused = parts.any(|parameter| {
                parameter
                    .trim()
                    .strip_prefix("q=")
                    .and_then(|q| q.trim().parse::<f32>().ok())
                    .is_some_and(|q| q == 0.0)
            });

            named && !refused
        })
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;

    fn request(accept_encoding: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();

        headers.insert(
            header::ACCEPT_ENCODING,
            HeaderValue::from_str(accept_encoding).unwrap(),
        );
        headers
    }

    #[test]
    fn test_prefers_brotli_then_gzip_and_respects_refusals() {
        let body = Encoded::new(br#"{"data":[1,2,3]}"#.repeat(50)).unwrap();
        let encoding = |accept| body.negotiate(&request(accept)).0;

        assert_eq!(
            encoding("gzip, deflate, br"),
            Some(HeaderValue::from_static("br"))
        );
        assert_eq!(
            encoding("gzip;q=1.0, br;q=0"),
            Some(HeaderValue::from_static("gzip"))
        );
        assert_eq!(encoding("identity"), None);
        assert_eq!(body.negotiate(&HeaderMap::new()).0, None);
    }

    #[test]
    fn test_every_encoding_holds_the_same_body() {
        let original = br#"{"data":["Kyle","Taylor","Woody"]}"#.repeat(100);
        let body = Encoded::new(original.clone()).unwrap();
        let mut from_br = Vec::new();
        let mut from_gzip = Vec::new();

        brotli::Decompressor::new(&body.br[..], BROTLI_BUFFER)
            .read_to_end(&mut from_br)
            .unwrap();
        flate2::read::GzDecoder::new(&body.gzip[..])
            .read_to_end(&mut from_gzip)
            .unwrap();

        assert_eq!(from_br, original);
        assert_eq!(from_gzip, original);
        assert!(body.br.len() < body.gzip.len());
    }
}

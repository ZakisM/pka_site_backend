use async_compression::tokio::write::ZstdEncoder;
use tokio::io::AsyncWriteExt;

pub mod pka_search;

pub trait Searchable {
    fn field_to_match(&self) -> &str;
}

pub trait Encodeable
where
    Self: bitcode::Encode,
{
    async fn as_bitcode_compressed(&self) -> anyhow::Result<Vec<u8>> {
        let bytes = bitcode::encode(self);

        let mut encoder = ZstdEncoder::with_quality(Vec::new(), async_compression::Level::Default);
        encoder.write_all(&bytes).await?;
        encoder.shutdown().await?;

        Ok(encoder.into_inner())
    }
}

impl<T> Encodeable for Vec<T> where T: bitcode::Encode {}

use std::io;

use anyhow::Result;
use bytes::Bytes;
use futures::stream::TryStreamExt;
use futures::Stream;
use tokio::{fs as tokio_fs, io::BufWriter};
use tokio_util::io::StreamReader;

/// Alias for a type-erased error type.
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

// Save a `Stream` to a file
pub async fn stream_to_file<S, E>(file: tokio_fs::File, stream: S) -> Result<()>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    // Convert the stream into an `AsyncRead`.
    let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
    let body_reader = StreamReader::new(body_with_io_error);
    futures::pin_mut!(body_reader);

    // Create the file. `File` implements `AsyncWrite`.
    let mut file = BufWriter::new(file);

    // Copy the body into the file.
    tokio::io::copy(&mut body_reader, &mut file).await?;

    Ok(())
}

use std::io;
use std::io::Read;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use async_stream::try_stream;
use bytes::Bytes;
use futures::Stream;
use futures::stream::TryStreamExt;
use tokio::task;
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
    let body_with_io_error = stream.map_err(|err| io::Error::other(err));
    let body_reader = StreamReader::new(body_with_io_error);
    futures::pin_mut!(body_reader);

    // Create the file. `File` implements `AsyncWrite`.
    let mut file = BufWriter::new(file);

    // Copy the body into the file.
    tokio::io::copy(&mut body_reader, &mut file).await?;

    Ok(())
}

type ReaderStream = Pin<Box<dyn Stream<Item = Result<Bytes, std::io::Error>> + Send>>;

pub fn reader_to_stream<R: Read + Send + 'static>(reader: R, chunk_size: usize) -> ReaderStream {
    let reader = Arc::new(Mutex::new(reader));
    Box::pin(try_stream! {
        loop {
            let reader_clone = reader.clone();
            // Allocate a new buffer for each iteration.
            let (n, buf) = task::spawn_blocking(move || {
                let mut buf = vec![0; chunk_size];
                let mut reader = reader_clone.lock().unwrap();
                let n = reader.read(&mut buf)?;

                Ok::<(usize, Vec<u8>), io::Error>((n, buf))
            }).await??;

            if n == 0 {
                break;
            }

            // If the buffer is fully used, we can avoid copying.
            if n == buf.len() {
                yield Bytes::from(buf);
            } else {
                yield Bytes::copy_from_slice(&buf[..n]);
            }
        }
    })
}

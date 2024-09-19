use std::io::{BufRead, Cursor, Write};

use anyhow::Result;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

use rs_utils::{
    confidential1::{create_confidential1_reader, create_confidential1_writer, Confidential1Key},
    create_file_reader, create_file_writer, create_gz_reader, create_gz_writer,
    crypto_key::CryptoKey,
    format_bytes, generate_alpanumeric_string, new_random_crypto_byte_array, read_container_lines,
    write_container_lines, TempFile,
};

fn container_write(mut writer: &mut impl Write, data: &[String]) {
    let index = (0..data.len())
        .map(|value| value.to_string())
        .collect::<Vec<_>>();

    write_container_lines(&mut writer, &index, data.iter().map(|value| value.as_str()))
        .expect("must write");
}

fn gz_container_write(writer: &mut impl Write, data: &[String]) {
    let mut gz_writer = create_gz_writer(writer);

    container_write(&mut gz_writer, data);

    gz_writer.finish().expect("must finish gz writer");
}

fn confidential1_container_write(writer: &mut impl Write, data: &[String], key: &Confidential1Key) {
    let mut confidential1_writer =
        create_confidential1_writer(writer, key).expect("must create confidential1 writer");
    let mut gz_writer = create_gz_writer(&mut confidential1_writer);

    container_write(&mut gz_writer, data);

    gz_writer.finish().expect("must finish gz writer");

    confidential1_writer
        .finish()
        .expect("must finish confidential1 writer");
}

fn container_read(reader: impl BufRead) -> Vec<String> {
    let (_index, iter) = read_container_lines(reader).expect("must read");

    iter.collect::<Result<Vec<_>>>()
        .expect("must read all lines")
}

fn gz_container_read(reader: impl BufRead) -> Vec<String> {
    let gz_reader = create_gz_reader(reader);

    container_read(gz_reader)
}

fn confidential1_container_read(reader: impl BufRead, key: &Confidential1Key) -> Vec<String> {
    let confidential1_reader =
        create_confidential1_reader(reader, key).expect("must create confidential1 reader");

    let gz_reader = create_gz_reader(confidential1_reader);

    container_read(gz_reader)
}

const BLOB_SIZE: usize = 2 * 1024;
const TOTAL_BLOBS_COUNT: usize = 10_000;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("container");
    group.sample_size(10);

    let data = (0..TOTAL_BLOBS_COUNT)
        .map(|_| generate_alpanumeric_string(BLOB_SIZE))
        .collect::<Vec<_>>();

    let new_cursor = || {
        Cursor::new(Vec::<u8>::with_capacity(
            BLOB_SIZE * TOTAL_BLOBS_COUNT +
            // index overhead
            10 * TOTAL_BLOBS_COUNT,
        ))
    };

    group.bench_function("text_container_write", |b| {
        b.iter(|| {
            let mut cursor = new_cursor();
            container_write(&mut cursor, &data);
        })
    });

    {
        let mut cursor = new_cursor();
        container_write(&mut cursor, &data);

        group.bench_function("text_container_read", |b| {
            b.iter(|| {
                cursor.set_position(0);
                black_box(container_read(&mut cursor));
            })
        });
    }

    group.bench_function("gz_container_write", |b| {
        b.iter(|| {
            let mut cursor = new_cursor();
            gz_container_write(&mut cursor, &data)
        })
    });

    {
        let mut cursor = new_cursor();
        gz_container_write(&mut cursor, &data);
        group.bench_function("gz_container_read", |b| {
            b.iter(|| {
                cursor.set_position(0);
                black_box(gz_container_read(&mut cursor));
            })
        });
    }

    let key = Confidential1Key::Key(CryptoKey::new(
        new_random_crypto_byte_array(),
        CryptoKey::random_salt(),
    ));
    group.bench_function("confidential1_container_write", |b| {
        b.iter(|| {
            let mut cursor = new_cursor();
            confidential1_container_write(&mut cursor, &data, &key)
        })
    });

    {
        let mut cursor = new_cursor();
        confidential1_container_write(&mut cursor, &data, &key);
        group.bench_function("confidential1_container_read", |b| {
            b.iter(|| {
                cursor.set_position(0);
                black_box(confidential1_container_read(&mut cursor, &key));
            })
        });
    }

    {
        let temp1 = TempFile::new();
        group.bench_function("confidential1_container_write to file", |b| {
            b.iter(|| {
                let mut writer = create_file_writer(&temp1.path).expect("must create file writer");
                confidential1_container_write(&mut writer, &data, &key);
            })
        });

        println!(
            "Created confidential1 file size: {}",
            format_bytes(temp1.size().unwrap()),
        );

        group.bench_function("confidential1_container_read from file", |b| {
            b.iter(|| {
                let reader = create_file_reader(&temp1.path).expect("must create file reader");
                black_box(confidential1_container_read(reader, &key))
            })
        });
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

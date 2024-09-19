use std::io::{BufRead, Write};

use anyhow::Result;
use criterion::{criterion_group, criterion_main, Criterion};

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

fn text_container_write(file_path: &str, data: &[String]) {
    let mut writer = create_file_writer(file_path).expect("must create text writer");

    container_write(&mut writer, data);
}

fn gz_container_write(file_path: &str, data: &[String]) {
    let writer = create_file_writer(file_path).expect("must create text writer");
    let mut gz_writer = create_gz_writer(writer);

    container_write(&mut gz_writer, data);

    gz_writer.finish().expect("must finish gz writer");
}

fn confidential1_container_write(file_path: &str, data: &[String], key: &Confidential1Key) {
    let writer = create_file_writer(file_path).expect("must create text writer");
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

fn text_container_read(file_path: &str) -> Vec<String> {
    let reader = create_file_reader(file_path).expect("must create text writer");

    container_read(reader)
}

fn gz_container_read(file_path: &str) -> Vec<String> {
    let reader = create_file_reader(file_path).expect("must create text writer");
    let gz_reader = create_gz_reader(reader);

    container_read(gz_reader)
}

fn confidential1_container_read(file_path: &str, key: &Confidential1Key) -> Vec<String> {
    let reader = create_file_reader(file_path).expect("must create text writer");

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

    let temp1 = TempFile::new();

    group.bench_function("text_container_write", |b| {
        b.iter(|| text_container_write(&temp1.path, &data))
    });

    {
        text_container_write(&temp1.path, &data);
        println!(
            "Created text file size: {}",
            format_bytes(temp1.size().unwrap())
        );

        group.bench_function("text_container_read", |b| {
            b.iter(|| text_container_read(&temp1.path))
        });
    }

    group.bench_function("gz_container_write", |b| {
        b.iter(|| gz_container_write(&temp1.path, &data))
    });

    {
        gz_container_write(&temp1.path, &data);
        println!(
            "Created gz file size: {}",
            format_bytes(temp1.size().unwrap())
        );
        group.bench_function("gz_container_read", |b| {
            b.iter(|| gz_container_read(&temp1.path))
        });
    }

    let key = Confidential1Key::Key(CryptoKey::new(
        new_random_crypto_byte_array(),
        CryptoKey::random_salt(),
    ));
    group.bench_function("confidential1_container_write", |b| {
        b.iter(|| confidential1_container_write(&temp1.path, &data, &key))
    });

    {
        confidential1_container_write(&temp1.path, &data, &key);
        println!(
            "Created confidential1 file size: {}",
            format_bytes(temp1.size().unwrap()),
        );
        group.bench_function("confidential1_container_read", |b| {
            b.iter(|| confidential1_container_read(&temp1.path, &key))
        });
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

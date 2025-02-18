use std::io::{BufRead, BufReader, Cursor, Write};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use rs_utils::{
    age::{AgeKey, AgeReader, AgeWriter},
    create_gz_reader, create_gz_writer, generate_alpanumeric_string, generate_bytes,
    get_file_hash_sha256, ContainerReader, ContainerWriter,
};

fn container_write(mut writer: &mut impl Write, data: &[String]) {
    let index = (0..data.len())
        .map(|value| value.to_string())
        .collect::<Vec<_>>();

    let mut writer = ContainerWriter::new(&mut writer);

    writer
        .write_index(&(&index).into())
        .expect("must create container writer");

    writer
        .write_lines(data.iter().map(|value| value.as_str()))
        .expect("must write");
}

fn gz_container_write(writer: &mut impl Write, data: &[String]) {
    let mut gz_writer = create_gz_writer(writer);

    container_write(&mut gz_writer, data);

    gz_writer.finish().expect("must finish gz writer");
}

fn age_gz_container_write(writer: &mut impl Write, data: &[String], key: AgeKey) {
    let mut age_writer = AgeWriter::new(writer, key).expect("must create age writer");
    let mut gz_writer = create_gz_writer(&mut age_writer);

    container_write(&mut gz_writer, data);

    gz_writer.finish().expect("must finish gz writer");

    age_writer.finish().expect("must finish age writer");
}

fn container_read(reader: impl BufRead) -> Vec<String> {
    ContainerReader::init_buffered(reader)
        .expect("must create container reader")
        .read_all()
        .expect("must read all lines")
}

fn gz_container_read(reader: impl BufRead) -> Vec<String> {
    let gz_reader = create_gz_reader(reader);
    let gz_buf_reader = BufReader::new(gz_reader);

    container_read(gz_buf_reader)
}

fn age_gz_container_read(reader: impl BufRead, key: AgeKey) -> Vec<String> {
    let age_reader = AgeReader::new(reader, key).expect("must create age reader");
    let age_buf_reader = BufReader::new(age_reader);

    let gz_reader = create_gz_reader(age_buf_reader);
    let gz_buf_reader = BufReader::new(gz_reader);

    container_read(gz_buf_reader)
}

const BLOB_SIZE: usize = 2 * 1024;
const TOTAL_BLOBS_COUNT: usize = 10_000;

fn new_cursor() -> Cursor<Vec<u8>> {
    Cursor::new(Vec::<u8>::with_capacity(
        BLOB_SIZE * TOTAL_BLOBS_COUNT +
            // index overhead
            10 * TOTAL_BLOBS_COUNT,
    ))
}

fn gen_data() -> Vec<String> {
    (0..TOTAL_BLOBS_COUNT)
        .map(|_| generate_alpanumeric_string(BLOB_SIZE))
        .collect::<Vec<_>>()
}

fn bench_text_container(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_container");
    group.sample_size(10);
    group.sampling_mode(criterion::SamplingMode::Flat);

    let data = gen_data();

    group.bench_function("text_container_write", |b| {
        b.iter(|| {
            let mut cursor = new_cursor();
            container_write(&mut cursor, &data);
        })
    });

    let mut cursor = new_cursor();
    container_write(&mut cursor, &data);

    group.bench_function("text_container_read", |b| {
        b.iter(|| {
            cursor.set_position(0);
            black_box(container_read(&mut cursor));
        })
    });

    group.finish();
}

fn bench_gz_container(c: &mut Criterion) {
    let mut group = c.benchmark_group("gz_container");
    group.sample_size(10);
    group.sampling_mode(criterion::SamplingMode::Flat);

    let data = gen_data();

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

    group.finish();
}

fn bench_age_container(c: &mut Criterion) {
    let mut group = c.benchmark_group("age_container");
    group.sample_size(10);
    group.sampling_mode(criterion::SamplingMode::Flat);

    let data = gen_data();

    let key = AgeKey::generate_age_x25519_key();
    group.bench_function("age_gz_container_write", |b| {
        b.iter(|| {
            let mut cursor = new_cursor();
            age_gz_container_write(&mut cursor, &data, key.clone())
        })
    });

    let mut cursor = new_cursor();
    age_gz_container_write(&mut cursor, &data, key.clone());
    group.bench_function("age_gz_container_read", |b| {
        b.iter(|| {
            cursor.set_position(0);
            black_box(age_gz_container_read(&mut cursor, key.clone()));
        })
    });

    group.finish();
}

fn bench_hashes(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashes");
    group.sample_size(10);
    group.sampling_mode(criterion::SamplingMode::Flat);

    let mut data = Cursor::new(generate_bytes(20 * 1024 * 1024));

    group.bench_function("sha256", |b| {
        b.iter(|| {
            data.set_position(0);
            black_box(get_file_hash_sha256(&mut data).expect("must hash"));
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_text_container,
    bench_gz_container,
    bench_age_container,
    bench_hashes
);
criterion_main!(benches);

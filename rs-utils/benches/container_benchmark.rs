use std::io::{BufRead, BufReader, Cursor, Write};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use rs_utils::{
    confidential1::{Confidential1Key, Confidential1Reader, Confidential1Writer},
    create_file_reader, create_file_writer, create_gz_reader, create_gz_writer,
    crypto_key::CryptoKey,
    format_bytes, generate_alpanumeric_string, generate_bytes, get_file_hash_blake3,
    get_file_hash_sha256, new_random_crypto_byte_array, ContainerReader, ContainerWriter, TempFile,
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

fn confidential1_gz_container_write(
    writer: &mut impl Write,
    data: &[String],
    key: &Confidential1Key,
) {
    let mut c1_writer =
        Confidential1Writer::new(writer, key).expect("must create confidential1 writer");
    let mut gz_writer = create_gz_writer(&mut c1_writer);

    container_write(&mut gz_writer, data);

    gz_writer.finish().expect("must finish gz writer");

    c1_writer
        .finish()
        .expect("must finish confidential1 writer");
}

fn container_read(reader: impl BufRead) -> Vec<String> {
    ContainerReader::init_buffered(reader)
        .expect("must create container reader")
        .read_all()
        .expect("must read all lines")
}

fn gz_container_read(reader: impl BufRead) -> Vec<String> {
    let gz_reader = create_gz_reader(reader);
    let gz_reader = BufReader::new(gz_reader);

    container_read(gz_reader)
}

fn confidential1_gz_container_read(reader: impl BufRead, key: &Confidential1Key) -> Vec<String> {
    let c1_reader =
        Confidential1Reader::new(reader, key).expect("must create confidential1 reader");
    let c1_reader = BufReader::new(c1_reader);

    let gz_reader = create_gz_reader(c1_reader);
    let gz_reader = BufReader::new(gz_reader);

    container_read(gz_reader)
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

fn bench_confidential1_container(c: &mut Criterion) {
    let mut group = c.benchmark_group("confidential1_container");
    group.sample_size(10);
    group.sampling_mode(criterion::SamplingMode::Flat);

    let data = gen_data();

    let key = Confidential1Key::Key(CryptoKey::new(
        new_random_crypto_byte_array(),
        CryptoKey::random_salt(),
    ));
    group.bench_function("confidential1_gz_container_write", |b| {
        b.iter(|| {
            let mut cursor = new_cursor();
            confidential1_gz_container_write(&mut cursor, &data, &key)
        })
    });

    let mut cursor = new_cursor();
    confidential1_gz_container_write(&mut cursor, &data, &key);
    group.bench_function("confidential1_gz_container_read", |b| {
        b.iter(|| {
            cursor.set_position(0);
            black_box(confidential1_gz_container_read(&mut cursor, &key));
        })
    });

    group.finish();
}

fn bench_confidential1_container_file(c: &mut Criterion) {
    let mut group = c.benchmark_group("confidential1_gz_container_file");
    group.sample_size(10);
    group.sampling_mode(criterion::SamplingMode::Flat);
    group.noise_threshold(6.5);

    let data = gen_data();

    let key = Confidential1Key::Key(CryptoKey::new(
        new_random_crypto_byte_array(),
        CryptoKey::random_salt(),
    ));

    let temp1 = TempFile::new();
    temp1.create_file().unwrap();

    group.bench_function("confidential1_gz_container_write", |b| {
        b.iter(|| {
            let mut writer = create_file_writer(&temp1.path).expect("must create file writer");
            confidential1_gz_container_write(&mut writer, &data, &key);
        })
    });

    {
        let mut writer = create_file_writer(&temp1.path).expect("must create file writer");
        confidential1_gz_container_write(&mut writer, &data, &key);

        println!(
            "Created confidential1 file size: {}",
            format_bytes(temp1.size().unwrap()),
        );
    }

    group.bench_function("confidential1_gz_container_read", |b| {
        b.iter(|| {
            let reader = create_file_reader(&temp1.path).expect("must create file reader");
            black_box(confidential1_gz_container_read(reader, &key))
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

    group.bench_function("blake3", |b| {
        b.iter(|| {
            data.set_position(0);
            black_box(get_file_hash_blake3(&mut data).expect("must hash"));
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_text_container,
    bench_gz_container,
    bench_confidential1_container,
    bench_confidential1_container_file,
    bench_hashes
);
criterion_main!(benches);

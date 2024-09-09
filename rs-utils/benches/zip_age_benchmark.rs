use std::collections::HashMap;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rs_utils::{format_bytes, generate_alpanumeric_string, TempFile, ZipAge};

fn create_agezip(data: &HashMap<&str, &[u8]>) {
    let temp1 = TempFile::new();

    ZipAge::create(&temp1.path, &data).unwrap();
}

fn read_agezip(path: &str, data: &HashMap<&str, &[u8]>) {
    let mut zip = ZipAge::open(&path).unwrap();
    let all_files = zip
        .list_files()
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    for file in all_files {
        let orig_file_data = data.get(file.as_str()).unwrap();
        let file_data = zip.get_file_bytes(&file).unwrap();

        assert_eq!(file_data.as_slice(), *orig_file_data);
    }
}

fn update_agezip(path: &str, updates: &HashMap<&str, Option<&[u8]>>) {
    let temp1 = TempFile::new();
    let mut zip = ZipAge::open(&path).unwrap();

    zip.update_and_save(&temp1.path, updates).unwrap();
}

fn generate_blobs(count: usize, size: usize) -> Vec<Vec<u8>> {
    (0..count)
        .map(|_| generate_alpanumeric_string(size).into())
        .collect()
}

fn create_data<'b>(blobs: &'b Vec<Vec<u8>>) -> HashMap<String, &'b [u8]> {
    let mut map = HashMap::new();

    for (i, blob) in blobs.into_iter().enumerate() {
        map.insert(format!("/blob-{i}"), blob.as_slice());
    }

    map
}

fn create_updates<'b>(
    new_blobs: &'b Vec<Vec<u8>>,
    delete_count: usize,
) -> HashMap<String, Option<&'b [u8]>> {
    let mut map = HashMap::new();

    for (i, blob) in new_blobs.into_iter().enumerate() {
        map.insert(format!("/blob-{i}"), Some(blob.as_slice()));
    }

    for i in 0..delete_count {
        map.insert(format!("/blob-{}", i + new_blobs.len()), None);
    }

    map
}

const BLOB_SIZE: usize = 2 * 1024;
const TOTAL_BLOBS_COUNT: usize = 10_000;
const UPDATE_BLOBS_COUNT: usize = TOTAL_BLOBS_COUNT / 3;
const DELETE_BLOBS_COUNT: usize = UPDATE_BLOBS_COUNT / 2;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("age-zip");
    group.sample_size(10);

    let data_blobs = generate_blobs(TOTAL_BLOBS_COUNT, BLOB_SIZE);
    let data = create_data(&data_blobs);
    let data = data
        .iter()
        .map(|(key, value)| (key.as_str(), *value))
        .collect();

    let updates_blobs = generate_blobs(UPDATE_BLOBS_COUNT, BLOB_SIZE);
    let updates = create_updates(&updates_blobs, DELETE_BLOBS_COUNT);
    let updates = updates
        .iter()
        .map(|(key, value)| (key.as_str(), *value))
        .collect();

    group.bench_function("create_agezip", |b| {
        b.iter(|| black_box(create_agezip(black_box(&data))))
    });

    let temp1 = TempFile::new();
    ZipAge::create(&temp1.path, &data).unwrap();
    println!("Created file size: {}", format_bytes(temp1.size().unwrap()));

    group.bench_function("read_agezip", |b| {
        b.iter(|| black_box(read_agezip(black_box(&temp1.path), black_box(&data))))
    });

    group.bench_function("update_agezip", |b| {
        b.iter(|| black_box(update_agezip(black_box(&temp1.path), black_box(&updates))))
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

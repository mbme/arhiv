use std::io::{BufReader, Write};

use anyhow::Result;

use rs_utils::{
    confidential1::{Confidential1Key, Confidential1Reader, Confidential1Writer},
    create_gz_reader, create_gz_writer,
    crypto_key::CryptoKey,
    crypto_stream::{CryptoStreamReader, CryptoStreamWriter},
    generate_alpanumeric_string, generate_alphanumeric_lines, new_random_crypto_byte_array,
    read_all_as_string, ContainerReader, ContainerWriter,
};

#[test]
fn test_c1_gz_read_write() -> Result<()> {
    let data = "ab";

    let key = Confidential1Key::new_random_key();

    let encrypted = {
        let mut c1_writer = Confidential1Writer::new(Vec::new(), &key)?;
        let mut gz_writer = create_gz_writer(&mut c1_writer);

        gz_writer.write_all(data.as_bytes())?;

        gz_writer.finish()?;
        c1_writer.finish()?
    };

    {
        let c1_reader = Confidential1Reader::new(encrypted.as_slice(), &key)?;
        let c1_reader = BufReader::new(c1_reader);
        let gz_reader = create_gz_reader(c1_reader);

        let result = read_all_as_string(gz_reader)?;

        assert_eq!(result, data);
    }

    Ok(())
}

#[test]
fn test_c1_gz_container_read_write() -> Result<()> {
    let lines = generate_alphanumeric_lines(2, 1);

    let key = Confidential1Key::new_random_key();

    let encrypted = {
        let mut data = Vec::with_capacity(lines.iter().map(|line| line.len()).sum());
        let mut c1_writer = Confidential1Writer::new(&mut data, &key)?;
        let mut gz_writer = create_gz_writer(&mut c1_writer);
        let mut writer = ContainerWriter::new(&mut gz_writer);

        let index = (0..lines.len()).map(|i| i.to_string()).collect::<Vec<_>>();
        writer.write_index(&index.as_slice().into())?;
        writer.write_lines(lines.iter().map(|line| line.as_str()))?;

        gz_writer.finish()?;
        c1_writer.finish()?;

        data
    };

    {
        let c1_reader = Confidential1Reader::new(encrypted.as_slice(), &key)?;
        let c1_reader = BufReader::new(c1_reader);
        let gz_reader = create_gz_reader(c1_reader);
        let reader = ContainerReader::init(gz_reader)?;

        let mut new_lines = vec![];
        for line in reader.into_lines_iter() {
            let (_key, value) = line?;
            new_lines.push(value);
        }
        assert_eq!(new_lines, lines);
    }

    Ok(())
}

#[test]
fn test_crypto_gz_container_read_write() -> Result<()> {
    let lines = generate_alphanumeric_lines(2, 1);

    let key = CryptoKey::new_random_key();
    let nonce = new_random_crypto_byte_array();

    let encrypted = {
        let mut data = Vec::with_capacity(lines.iter().map(|line| line.len()).sum());
        let mut crypto_writer = CryptoStreamWriter::new_chacha20(&mut data, key.get(), &nonce);
        let mut gz_writer = create_gz_writer(&mut crypto_writer);
        let mut writer = ContainerWriter::new(&mut gz_writer);

        let index = (0..lines.len()).map(|i| i.to_string()).collect::<Vec<_>>();
        writer.write_index(&index.as_slice().into())?;
        writer.write_lines(lines.iter().map(|line| line.as_str()))?;

        gz_writer.finish()?;

        data
    };

    {
        let crypto_reader =
            CryptoStreamReader::new_chacha20(encrypted.as_slice(), key.get(), &nonce);
        let crypto_reader = BufReader::new(crypto_reader);
        let gz_reader = create_gz_reader(crypto_reader);
        let reader = ContainerReader::init(gz_reader)?;

        let mut new_lines = vec![];
        for line in reader.into_lines_iter() {
            let (_key, value) = line?;
            new_lines.push(value);
        }
        assert_eq!(new_lines, lines);
    }

    Ok(())
}

#[test]
fn test_crypto_gz_read_write() -> Result<()> {
    let line = generate_alpanumeric_string(100);

    let key = CryptoKey::new_random_key();
    let nonce = new_random_crypto_byte_array();

    let encrypted = {
        let mut data = Vec::with_capacity(line.bytes().len());
        let mut crypto_writer = CryptoStreamWriter::new_chacha20(&mut data, key.get(), &nonce);
        let mut gz_writer = create_gz_writer(&mut crypto_writer);

        gz_writer.write_all(line.as_bytes())?;

        gz_writer.finish()?;

        data
    };

    {
        let crypto_reader =
            CryptoStreamReader::new_chacha20(encrypted.as_slice(), key.get(), &nonce);

        let crypto_reader = BufReader::new(crypto_reader);
        let gz_reader = create_gz_reader(crypto_reader);

        let result = read_all_as_string(gz_reader).unwrap();

        assert_eq!(result, line);
    }

    Ok(())
}

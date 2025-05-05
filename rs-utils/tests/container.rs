use std::io::{BufReader, Write};

use anyhow::Result;

use rs_utils::{
    ContainerReader, ContainerWriter,
    age::{AgeKey, AgeReader, AgeWriter},
    create_gz_reader, create_gz_writer, generate_alphanumeric_lines, read_all_as_string,
};

#[test]
fn test_age_gz_read_write() -> Result<()> {
    let data = "ab";

    let key = AgeKey::generate_age_x25519_key();

    let encrypted = {
        let mut age_writer = AgeWriter::new(Vec::new(), key.clone())?;
        let mut gz_writer = create_gz_writer(&mut age_writer);

        gz_writer.write_all(data.as_bytes())?;

        gz_writer.finish()?;
        age_writer.finish()?
    };

    {
        let age_reader = AgeReader::new(encrypted.as_slice(), key)?;
        let buf_reader = BufReader::new(age_reader);
        let gz_reader = create_gz_reader(buf_reader);

        let result = read_all_as_string(gz_reader)?;

        assert_eq!(result, data);
    }

    Ok(())
}

#[test]
fn test_c1_gz_container_read_write() -> Result<()> {
    let lines = generate_alphanumeric_lines(2, 1);

    let key = AgeKey::generate_age_x25519_key();

    let encrypted = {
        let mut data = Vec::with_capacity(lines.iter().map(|line| line.len()).sum());
        let mut age_writer = AgeWriter::new(&mut data, key.clone())?;
        let mut gz_writer = create_gz_writer(&mut age_writer);
        let mut writer = ContainerWriter::new(&mut gz_writer);

        let index = (0..lines.len()).map(|i| i.to_string()).collect::<Vec<_>>();
        writer.write_index(&index.as_slice().into())?;
        writer.write_lines(lines.iter().map(|line| line.as_str()))?;

        gz_writer.finish()?;
        age_writer.finish()?;

        data
    };

    {
        let age_reader = AgeReader::new(encrypted.as_slice(), key)?;
        let buf_reader = BufReader::new(age_reader);
        let gz_reader = create_gz_reader(buf_reader);
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

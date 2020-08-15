use anyhow::*;
use std::fs;

pub fn file_exists(path: &str) -> Result<bool> {
    match fs::metadata(path) {
        Ok(metadata) if !metadata.is_file() => Err(anyhow!("path isn't a file: {}", path)),

        Ok(_) => Ok(true),

        Err(_) => Ok(false),
    }
}

pub fn dir_exists(path: &str) -> Result<bool> {
    match fs::metadata(path) {
        Ok(metadata) if !metadata.is_dir() => Err(anyhow!("path isn't a directory: {}", path)),

        Ok(_) => Ok(true),

        Err(_) => Ok(false),
    }
}

pub fn ensure_dir_exists(path: &str) -> Result<()> {
    if dir_exists(path)? {
        Ok(())
    } else {
        Err(anyhow!("dir doesn't exist {}", path))
    }
}

pub fn ensure_file_exists(path: &str) -> Result<()> {
    if file_exists(path)? {
        Ok(())
    } else {
        Err(anyhow!("file doesn't exist {}", path))
    }
}

pub fn fuzzy_match(needle: &str, haystack: &str) -> bool {
    // if needle is empty then it matches everything
    if needle.is_empty() {
        return true;
    }

    if needle.len() > haystack.len() {
        return false;
    }

    let needle = needle.to_lowercase();
    let haystack = haystack.to_lowercase();

    if needle.len() == haystack.len() {
        return needle == haystack;
    }

    let mut haystack_chars = haystack.chars();

    'outer: for needle_char in needle.chars() {
        loop {
            if let Some(haystack_char) = haystack_chars.next() {
                if haystack_char == needle_char {
                    continue 'outer;
                }
            } else {
                return false;
            }
        }
    }

    return true;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_match() {
        assert_eq!(fuzzy_match("", ""), true);
        assert_eq!(fuzzy_match("", "test"), true);
        assert_eq!(fuzzy_match("test", "test"), true);
        assert_eq!(fuzzy_match("test", "te"), false);
        assert_eq!(fuzzy_match("TEST", "teSt"), true);
        assert_eq!(fuzzy_match("123", "1test2test3"), true);
        assert_eq!(fuzzy_match("123", "123test2test3"), true);
        assert_eq!(fuzzy_match("123", "12test2test"), false);
    }
}

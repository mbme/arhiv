use crate::entities::Id;
use rand::seq::SliceRandom;

const ID_ALPHABET: &str = "0123456789abcdefghijklmnopqrstuvwxyz";
const ID_LENGTH: usize = 15;

pub fn gen_random_id() -> Id {
    ID_ALPHABET
        .chars()
        .collect::<Vec<char>>()
        .choose_multiple(&mut rand::thread_rng(), ID_LENGTH)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gen_random_id() {
        let id = gen_random_id();

        assert_eq!(id.len(), ID_LENGTH);

        let only_known_chars = id
            .chars()
            .map(|ch| ID_ALPHABET.contains(ch))
            .fold(true, |acc, res| res || acc);

        assert_eq!(only_known_chars, true);
    }
}

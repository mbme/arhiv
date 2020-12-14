use arhiv::entities::*;
use rand::prelude::*;
use rand::thread_rng;
use rs_utils::{project_relpath, Markov};
use std::fs;

use crate::markup::create_ref;
use crate::markup::MarkupString;

pub struct TextGenerator {
    markov: Markov,
    attachment_ids: Vec<Id>,
}

impl TextGenerator {
    pub fn new(attachments: &Vec<AttachmentSource>) -> Self {
        let text = fs::read_to_string(project_relpath("../resources/text.txt")).unwrap();
        let markov = Markov::new(&text);

        TextGenerator {
            markov,
            attachment_ids: attachments.iter().map(|item| item.id.clone()).collect(),
        }
    }

    pub fn gen_string(&self, min_words: u32, max_words: u32) -> String {
        self.markov
            .generate_sentence_constrained(min_words, max_words, false)
    }

    pub fn gen_markup_string(&self, min_paragraphs: u32, max_paragraphs: u32) -> MarkupString {
        let mut data: Vec<String> = vec![];
        let mut rng = thread_rng();
        for _ in 0..rng.gen_range(min_paragraphs, max_paragraphs + 1) {
            let mut sentences = vec![];

            for _ in 0..rng.gen_range(1, 8) {
                sentences.push(self.markov.generate_sentence_constrained(2, 20, true));
            }

            if rng.gen_bool(0.33) {
                let id = self
                    .attachment_ids
                    .choose(&mut rng)
                    .expect("attachment ids must be provided");

                sentences.push(create_ref(id, ""));
            }

            sentences.shuffle(&mut rng);

            data.push(sentences.join(" "));
        }

        data.join("\n\n").into()
    }
}

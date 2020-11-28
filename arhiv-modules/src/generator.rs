use arhiv::entities::*;
use rand::prelude::*;
use rand::thread_rng;
use rs_utils::{project_relpath, Markov};
use std::fs;

use crate::markup::create_ref;
use crate::markup::MarkupString;

pub fn create_attachments() -> Vec<AttachmentSource> {
    let mut attachments: Vec<AttachmentSource> = vec![];

    let dir = project_relpath("../resources");
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let path = path.to_str().unwrap();

        if path.ends_with(".jpg") || path.ends_with(".jpeg") {
            let attachment = AttachmentSource::new(path);
            attachments.push(attachment);
        }
    }

    attachments
}

pub struct Generator {
    markov: Markov,
    attachment_ids: Vec<Id>,
}

impl Generator {
    pub fn new(attachments: &Vec<AttachmentSource>) -> Self {
        let text = fs::read_to_string(project_relpath("../resources/text.txt")).unwrap();
        let markov = Markov::new(&text);

        Generator {
            markov,
            attachment_ids: attachments.iter().map(|item| item.id.clone()).collect(),
        }
    }

    pub fn gen_string(&self) -> String {
        self.markov.generate_sentence_constrained(8, false)
    }

    pub fn gen_markup_string(&self, min_paragraphs: u8, max_paragraphs: u8) -> MarkupString {
        let mut data: Vec<String> = vec![];
        let mut rng = thread_rng();
        for _ in 0..rng.gen_range(min_paragraphs, max_paragraphs) {
            let mut sentences = vec![];

            for _ in 0..rng.gen_range(1, 8) {
                sentences.push(self.markov.generate_sentence_constrained(20, true));
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

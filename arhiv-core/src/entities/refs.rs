use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::Id;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Refs {
    pub documents: HashSet<Id>,
    pub collections: HashSet<Id>,
}

impl Refs {
    #[must_use]
    pub fn new() -> Self {
        Refs {
            documents: HashSet::new(),
            collections: HashSet::new(),
        }
    }

    pub fn clear(&mut self) {
        self.documents.clear();
        self.collections.clear();
    }

    #[must_use]
    pub fn all(&self) -> HashSet<&Id> {
        self.documents.union(&self.collections).collect()
    }
}

impl Default for Refs {
    fn default() -> Self {
        Refs::new()
    }
}

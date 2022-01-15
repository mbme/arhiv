use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::{BLOBId, Id};

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Refs {
    pub documents: HashSet<Id>,
    pub collections: HashSet<Id>,
    pub blobs: HashSet<BLOBId>,
}

impl Refs {
    #[must_use]
    pub fn all(&self) -> HashSet<&Id> {
        self.documents.union(&self.collections).collect()
    }
}

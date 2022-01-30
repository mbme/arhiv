use std::{collections::HashSet, fmt};

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

impl fmt::Display for Refs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(&self).expect("failed to serialize Refs")
        )
    }
}

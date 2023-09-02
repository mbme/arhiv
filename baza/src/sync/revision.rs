use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashSet},
};

use anyhow::{Context, Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::instance_id::InstanceId;

#[derive(Serialize, Deserialize, Clone, Debug, Eq)]
pub struct Revision(BTreeMap<InstanceId, u32>);

#[derive(Debug, PartialEq)]
pub enum VectorClockOrder {
    Before,
    After,
    Equal,
    Concurrent,
}

impl Revision {
    pub fn from_value(value: Value) -> Result<Self> {
        value.try_into()
    }

    pub fn staging() -> Self {
        Revision(BTreeMap::new())
    }

    #[must_use]
    pub fn is_staged(&self) -> bool {
        self.0.values().sum::<u32>() == 0
    }

    #[must_use]
    pub fn is_committed(&self) -> bool {
        !self.is_staged()
    }

    pub fn get_version(&self, id: &InstanceId) -> u32 {
        self.0.get(id).copied().unwrap_or_default()
    }

    pub fn set_version(&mut self, id: &InstanceId, version: u32) {
        if version == 0 {
            self.0.remove(id);
        } else {
            self.0.insert(id.clone(), version);
        }
    }

    pub fn inc(&mut self, id: &InstanceId) {
        let next_version = self.get_version(id) + 1;

        self.set_version(id, next_version);
    }

    #[must_use]
    pub fn compare_vector_clocks(&self, other: &Self) -> VectorClockOrder {
        let all_keys: HashSet<&InstanceId> = self.0.keys().chain(other.0.keys()).collect();

        let mut is_self_empty = true;
        let mut is_other_empty = true;

        let mut has_before = false;
        let mut has_after = false;

        for key in all_keys {
            let value = self.0.get(key).unwrap_or(&0);
            let other_value = other.0.get(key).unwrap_or(&0);

            if *value != 0 {
                is_self_empty = false;
            }

            if *other_value != 0 {
                is_other_empty = false;
            }

            match value.cmp(other_value) {
                Ordering::Less => {
                    has_before = true;
                }
                Ordering::Greater => {
                    has_after = true;
                }
                Ordering::Equal => {}
            };

            if has_before && has_after {
                return VectorClockOrder::Concurrent;
            }
        }

        match (is_self_empty, is_other_empty) {
            (true, true) => return VectorClockOrder::Equal,
            (true, false) => return VectorClockOrder::After,
            (false, true) => return VectorClockOrder::Before,
            (false, false) => {}
        }

        match (has_before, has_after) {
            (true, false) => VectorClockOrder::Before,
            (false, false) => VectorClockOrder::Equal,
            (false, true) => VectorClockOrder::After,
            (true, true) => VectorClockOrder::Concurrent,
        }
    }

    #[must_use]
    pub fn is_concurrent_or_newer_than(&self, other: &Self) -> bool {
        matches!(
            self.compare_vector_clocks(other),
            VectorClockOrder::After | VectorClockOrder::Concurrent
        )
    }

    #[must_use]
    pub fn is_concurrent_or_older_than(&self, other: &Self) -> bool {
        matches!(
            self.compare_vector_clocks(other),
            VectorClockOrder::Before | VectorClockOrder::Concurrent
        )
    }

    pub fn serialize(&self) -> String {
        let mut keys: Vec<_> = self.0.keys().collect();

        keys.sort();

        let mut result = String::new();

        result.push('{');

        let mut is_first = true;
        for key in keys {
            let value = *self
                .0
                .get(key)
                .expect("revision must contain a value for a key");

            if !is_first {
                result.push(',');
            }
            is_first = false;

            result.push('\"');
            result.push_str(key.as_ref());
            result.push_str("\":");
            result.push_str(&value.to_string());
        }

        result.push('}');

        result
    }
}

impl TryInto<Revision> for Value {
    type Error = Error;

    fn try_into(self) -> Result<Revision, Self::Error> {
        let mut result: Revision =
            serde_json::from_value(self).context("failed to convert into Revision")?;

        result.0.retain(|_, value| *value > 0);

        Ok(result)
    }
}

impl PartialEq for Revision {
    fn eq(&self, other: &Self) -> bool {
        self.compare_vector_clocks(other) == VectorClockOrder::Equal
    }
}

impl PartialOrd for Revision {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.compare_vector_clocks(other) {
            VectorClockOrder::Before => Some(Ordering::Less),
            VectorClockOrder::After => Some(Ordering::Greater),
            VectorClockOrder::Equal => Some(Ordering::Equal),
            VectorClockOrder::Concurrent => None,
        }
    }
}

impl Ord for Revision {
    fn cmp(&self, other: &Self) -> Ordering {
        if let Some(ordering) = self.partial_cmp(other) {
            ordering
        } else {
            self.serialize().cmp(&other.serialize())
        }
    }
}

impl Default for Revision {
    fn default() -> Self {
        Revision::staging()
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use serde_json::json;

    use crate::sync::{instance_id::InstanceId, revision::VectorClockOrder};

    use super::Revision;

    #[test]
    fn test_revision_inc() -> Result<()> {
        {
            let mut rev = Revision::from_value(json!({}))?;
            let instance_id = InstanceId::from_string("1");

            rev.inc(&instance_id);

            assert_eq!(rev, Revision::from_value(json!({ "1": 1 }))?);
        }

        {
            let mut rev = Revision::from_value(json!({ "1": 1, "2": 2 }))?;
            let instance_id = InstanceId::from_string("1");

            rev.inc(&instance_id);

            assert_eq!(rev, Revision::from_value(json!({ "1": 2, "2": 2 }))?);
        }
        Ok(())
    }

    #[test]
    fn test_revision_compare_vector_clocks() -> Result<()> {
        {
            let rev1 = Revision::from_value(json!({ "1": 1, "2": 2 }))?;
            let rev2 = Revision::from_value(json!({ "1": 2, "2": 1 }))?;

            assert_eq!(
                rev1.compare_vector_clocks(&rev2),
                VectorClockOrder::Concurrent
            );
            assert_eq!(
                rev2.compare_vector_clocks(&rev1),
                VectorClockOrder::Concurrent
            );
        }

        {
            let rev1 = Revision::from_value(json!({ "1": 1, "2": 2 }))?;
            let rev2 = Revision::from_value(json!({ "1": 1, "2": 2 }))?;

            assert_eq!(rev1.compare_vector_clocks(&rev2), VectorClockOrder::Equal);
            assert_eq!(rev2.compare_vector_clocks(&rev1), VectorClockOrder::Equal);
        }

        {
            let rev1 = Revision::from_value(json!({ "1": 1, "2": 1 }))?;
            let rev2 = Revision::from_value(json!({ "1": 1, "2": 2 }))?;

            assert_eq!(rev1.compare_vector_clocks(&rev2), VectorClockOrder::Before);
            assert_eq!(rev2.compare_vector_clocks(&rev1), VectorClockOrder::After);
        }

        {
            let rev1 = Revision::from_value(json!({ "1": 1, }))?;
            let rev2 = Revision::from_value(json!({ "1": 1, "2": 2}))?;

            assert_eq!(rev1.compare_vector_clocks(&rev2), VectorClockOrder::Before);
            assert_eq!(rev2.compare_vector_clocks(&rev1), VectorClockOrder::After);
        }

        {
            let rev1 = Revision::from_value(json!({ "1": 1, "2": 1 }))?;
            let rev2 = Revision::from_value(json!({ "1": 0 }))?;

            assert_eq!(rev1.compare_vector_clocks(&rev2), VectorClockOrder::Before);
            assert_eq!(rev2.compare_vector_clocks(&rev1), VectorClockOrder::After);
        }

        Ok(())
    }

    #[test]
    fn test_revision_cmp() -> Result<()> {
        {
            let rev0 = Revision::default();
            let rev1 = Revision::from_value(json!({ "1": 1, "2": 1 }))?;
            let rev2 = Revision::from_value(json!({ "1": 1, "2": 2 }))?;
            let rev3 = Revision::from_value(json!({ "1": 1, "2": 1 }))?;
            let rev4 = Revision::from_value(json!({ "1": 2, "2": 1 }))?;

            assert!(rev0 > rev1);
            assert!(rev0 > rev2);
            assert!(rev0 > rev3);
            assert!(rev0 > rev4);

            assert!(rev1 < rev2);
            assert!(rev1 <= rev2);

            assert!(rev2 > rev1);
            assert!(rev2 >= rev1);

            assert!(rev3 == rev1);
            assert!(rev3 <= rev1);
            assert!(rev3 >= rev1);

            assert!(rev4 != rev2);
        }

        Ok(())
    }

    #[test]
    fn test_revision_serialize() -> Result<()> {
        {
            let rev = Revision::from_value(json!({ "1": 1, "2": 1 }))?;
            assert_eq!(rev.serialize(), r#"{"1":1,"2":1}"#);
        }

        {
            let rev = Revision::from_value(json!({ "2": 1, "1": 1 }))?;
            assert_eq!(rev.serialize(), r#"{"1":1,"2":1}"#);
        }

        {
            let rev = Revision::from_value(json!({ "1": 0, "2": 1 }))?;
            assert_eq!(rev.serialize(), r#"{"2":1}"#);
        }

        Ok(())
    }
}

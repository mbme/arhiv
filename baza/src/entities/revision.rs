use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashSet},
};

use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::instance_id::InstanceId;

#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Serialize, Deserialize, Hash, Clone, Debug, Eq)]
pub struct Revision(BTreeMap<InstanceId, u32>);

#[derive(Debug, PartialEq)]
pub enum VectorClockOrder {
    Before,
    After,
    Equal,
    Concurrent,
}

impl Revision {
    pub const STAGED_STRING: &'static str = "null";

    pub const INITIAL: &'static Self = &Self::initial();

    pub const fn initial() -> Self {
        Revision(BTreeMap::new())
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

        let mut has_before = false;
        let mut has_after = false;

        for key in all_keys {
            let value = self.0.get(key).unwrap_or(&0);
            let other_value = other.0.get(key).unwrap_or(&0);

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

    #[must_use]
    pub fn is_concurrent_or_equal(&self, other: &Self) -> bool {
        matches!(
            self.compare_vector_clocks(other),
            VectorClockOrder::Equal | VectorClockOrder::Concurrent
        )
    }

    #[must_use]
    pub fn is_concurrent(&self, other: &Self) -> bool {
        matches!(
            self.compare_vector_clocks(other),
            VectorClockOrder::Concurrent
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

    pub fn to_string(rev: &Option<Revision>) -> String {
        rev.as_ref()
            .map(|rev| rev.serialize())
            .unwrap_or(Revision::STAGED_STRING.to_string())
    }

    pub fn to_file_name(&self) -> String {
        let mut items: Vec<_> = self
            .0
            .iter()
            .map(|(id, version)| format!("{id}:{version}"))
            .collect();

        items.sort();

        items.join("-")
    }

    pub fn from_file_name(value: &str) -> Result<Self> {
        let map = value
            .split("-")
            .map(|segment| {
                let mut parts = segment.split(":");

                let id = parts
                    .next()
                    .context("Failed to extract id from the segment")?;
                let version = parts
                    .next()
                    .context("Failed to extract version from the segment")?;
                ensure!(parts.next().is_none(), "Got invalid segment {segment}");

                let id: InstanceId = id.try_into().context("Failed to parse instance id")?;
                let version: u32 = version.parse().context("Failed to parse version")?;

                Ok((id, version))
            })
            .collect::<Result<_>>()?;

        Ok(Revision(map))
    }

    pub fn from_value(value: Value) -> Result<Revision> {
        Revision::try_from_value(value)?.context("expected a valid revision map")
    }

    pub fn try_from_value(value: Value) -> Result<Option<Revision>> {
        let mut result: Option<Revision> =
            serde_json::from_value(value).context("failed to convert into Revision")?;

        if let Some(ref mut rev) = result {
            rev.0.retain(|_, value| *value > 0);
        }

        Ok(result)
    }

    pub fn merge(&mut self, other: &Self) {
        for (key, value) in &other.0 {
            if let Some(local_value) = self.0.get_mut(key) {
                *local_value = (*local_value).max(*value);
            } else {
                self.0.insert(key.clone(), *value);
            }
        }
    }

    #[must_use]
    pub fn merge_all<'r>(revs: impl Iterator<Item = &'r Revision>) -> Revision {
        revs.fold(Revision::initial(), |mut acc, rev| {
            acc.merge(rev);

            acc
        })
    }

    #[must_use]
    pub fn get_latest_rev<'r>(revs: &[&'r Revision]) -> HashSet<&'r Revision> {
        revs.iter().fold(HashSet::new(), |mut acc, rev| {
            if acc.is_empty() {
                acc.insert(rev);

                return acc;
            }

            let max_rev = acc.iter().next().expect("acc isn't empty");

            if rev > max_rev {
                acc.clear();
                acc.insert(rev);

                return acc;
            }

            if rev.is_concurrent_or_equal(max_rev) {
                acc.insert(rev);
            }

            acc
        })
    }

    #[must_use]
    pub fn compute_next_rev<'r>(
        revs: impl Iterator<Item = &'r Revision>,
        for_instance: &InstanceId,
    ) -> Revision {
        let mut max_rev = Self::merge_all(revs);

        max_rev.inc(for_instance);

        max_rev
    }
}

impl PartialEq for Revision {
    fn eq(&self, other: &Self) -> bool {
        self.compare_vector_clocks(other) == VectorClockOrder::Equal
    }
}

#[allow(clippy::non_canonical_partial_ord_impl)]
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

impl Default for &Revision {
    fn default() -> Self {
        Revision::INITIAL
    }
}

pub struct LatestRevComputer<'r>(HashSet<&'r Revision>);

impl<'r> LatestRevComputer<'r> {
    pub fn new() -> Self {
        let mut revs = HashSet::new();
        revs.insert(Revision::INITIAL);

        Self(revs)
    }
}

impl<'r> Default for LatestRevComputer<'r> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'r> LatestRevComputer<'r> {
    pub fn update(&mut self, new_revs: impl IntoIterator<Item = &'r Revision>) {
        for new_rev in new_revs.into_iter() {
            let latest_rev = self.0.iter().next().expect("latest revs must not be empty");

            if latest_rev > &new_rev {
                continue;
            }

            if latest_rev < &new_rev {
                self.0.clear();
            }

            self.0.insert(new_rev);
        }
    }

    #[must_use]
    pub fn get(self) -> HashSet<&'r Revision> {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use anyhow::Result;
    use serde_json::json;

    use crate::entities::{revision::VectorClockOrder, InstanceId};

    use super::{LatestRevComputer, Revision};

    #[test]
    fn test_revision_inc() -> Result<()> {
        {
            let mut rev = Revision::from_value(json!({}))?;
            let instance_id = InstanceId::from_string("a");

            rev.inc(&instance_id);

            assert_eq!(rev, Revision::from_value(json!({ "a": 1 }))?);
        }

        {
            let mut rev = Revision::from_value(json!({ "a": 1, "b": 2 }))?;
            let instance_id = InstanceId::from_string("a");

            rev.inc(&instance_id);

            assert_eq!(rev, Revision::from_value(json!({ "a": 2, "b": 2 }))?);
        }
        Ok(())
    }

    #[test]
    fn test_revision_compare_vector_clocks() -> Result<()> {
        {
            let rev1 = Revision::from_value(json!({ "a": 1, "b": 2 }))?;
            let rev2 = Revision::from_value(json!({ "a": 2, "b": 1 }))?;

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
            let rev1 = Revision::from_value(json!({ "a": 1, "b": 2 }))?;
            let rev2 = Revision::from_value(json!({ "a": 1, "b": 2 }))?;

            assert_eq!(rev1.compare_vector_clocks(&rev2), VectorClockOrder::Equal);
            assert_eq!(rev2.compare_vector_clocks(&rev1), VectorClockOrder::Equal);
        }

        {
            let rev1 = Revision::from_value(json!({ "a": 1, "b": 1 }))?;
            let rev2 = Revision::from_value(json!({ "a": 1, "b": 2 }))?;

            assert_eq!(rev1.compare_vector_clocks(&rev2), VectorClockOrder::Before);
            assert_eq!(rev2.compare_vector_clocks(&rev1), VectorClockOrder::After);
        }

        {
            let rev1 = Revision::from_value(json!({ "a": 1, }))?;
            let rev2 = Revision::from_value(json!({ "a": 1, "b": 2}))?;

            assert_eq!(rev1.compare_vector_clocks(&rev2), VectorClockOrder::Before);
            assert_eq!(rev2.compare_vector_clocks(&rev1), VectorClockOrder::After);
        }

        Ok(())
    }

    #[test]
    fn test_revision_cmp() -> Result<()> {
        {
            let rev0 = Revision::initial();
            let rev1 = Revision::from_value(json!({ "a": 1, "b": 1 }))?;
            let rev2 = Revision::from_value(json!({ "a": 1, "b": 2 }))?;
            let rev3 = Revision::from_value(json!({ "a": 1, "b": 1 }))?;
            let rev4 = Revision::from_value(json!({ "a": 2, "b": 1 }))?;

            assert!(rev0 < rev1);
            assert!(rev0 < rev2);
            assert!(rev0 < rev3);
            assert!(rev0 < rev4);

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
    fn test_revision_is_concurrent_or_newer_than() -> Result<()> {
        {
            let rev0 = Revision::initial();
            let rev1 = Revision::from_value(json!({ "a": 1, "b": 1 }))?;
            let rev2 = Revision::from_value(json!({ "a": 1, "b": 2 }))?;
            let rev3 = Revision::from_value(json!({ "a": 2, "b": 1 }))?;

            assert!(!rev0.is_concurrent_or_newer_than(&rev1));
            assert!(rev1.is_concurrent_or_newer_than(&rev0));

            assert!(rev3.is_concurrent_or_newer_than(&rev1));

            assert!(rev2.is_concurrent_or_newer_than(&rev3));
            assert!(rev3.is_concurrent_or_newer_than(&rev2));
        }

        Ok(())
    }

    #[test]
    fn test_revision_is_concurrent_or_older_than() -> Result<()> {
        {
            let rev0 = Revision::initial();
            let rev1 = Revision::from_value(json!({ "a": 1, "b": 1 }))?;
            let rev2 = Revision::from_value(json!({ "a": 1, "b": 2 }))?;
            let rev3 = Revision::from_value(json!({ "a": 2, "b": 1 }))?;

            assert!(rev0.is_concurrent_or_older_than(&rev1));
            assert!(!rev1.is_concurrent_or_older_than(&rev0));

            assert!(rev1.is_concurrent_or_older_than(&rev3));

            assert!(rev2.is_concurrent_or_older_than(&rev3));
            assert!(rev3.is_concurrent_or_older_than(&rev2));
        }

        Ok(())
    }

    #[test]
    fn test_revision_serialize() -> Result<()> {
        {
            let rev = Revision::from_value(json!({ "a": 1, "b": 1 }))?;
            assert_eq!(rev.serialize(), r#"{"a":1,"b":1}"#);
        }

        {
            let rev = Revision::from_value(json!({ "b": 1, "a": 1 }))?;
            assert_eq!(rev.serialize(), r#"{"a":1,"b":1}"#);
        }

        {
            let rev = Revision::from_value(json!({ "a": 0, "b": 1 }))?;
            assert_eq!(rev.serialize(), r#"{"b":1}"#);
        }

        Ok(())
    }

    #[test]
    fn test_revision_merge() -> Result<()> {
        {
            let mut rev1 = Revision::from_value(json!({ "a": 1, "b": 2 }))?;
            let rev2 = Revision::from_value(json!({ "a": 2, "b": 1 }))?;

            rev1.merge(&rev2);

            assert_eq!(rev1, Revision::from_value(json!({ "a": 2, "b": 2 }))?);
        }

        Ok(())
    }

    #[test]
    fn test_revision_get_latest_rev() -> Result<()> {
        let rev1 = Revision::from_value(json!({ "a": 1, "b": 1 }))?;
        let rev2 = Revision::from_value(json!({ "a": 1, "b": 2 }))?;
        let rev3 = Revision::from_value(json!({ "a": 2, "b": 1 }))?;

        {
            let refs = vec![&rev1, &rev2, &rev3];

            assert_eq!(
                Revision::get_latest_rev(refs.as_slice()),
                vec![&rev2, &rev3].into_iter().collect(),
            );
        }

        {
            let refs = vec![&rev1, &rev3];

            assert_eq!(
                Revision::get_latest_rev(refs.as_slice()),
                vec![&rev3].into_iter().collect()
            );
        }

        Ok(())
    }

    #[test]
    fn test_revision_compute_next_rev() -> Result<()> {
        let rev1 = Revision::from_value(json!({ "a": 1, "b": 1 }))?;
        let rev2 = Revision::from_value(json!({ "a": 1, "b": 2 }))?;
        let rev3 = Revision::from_value(json!({ "a": 2, "b": 1 }))?;

        {
            let refs = [rev1.clone(), rev2.clone(), rev3.clone()];

            assert_eq!(
                Revision::compute_next_rev(refs.iter(), &InstanceId::from_string("a")),
                Revision::from_value(json!({ "a": 3, "b": 2 }))?
            );
        }

        {
            let refs = [rev1.clone(), rev2.clone(), rev3.clone()];

            assert_eq!(
                Revision::compute_next_rev(refs.iter(), &InstanceId::from_string("c")),
                Revision::from_value(json!({ "a": 2, "b": 2, "c": 1 }))?
            );
        }

        {
            let rev4 = Revision::from_value(json!({ "a": 1, "b": 1, "c": 2 }))?;

            let refs = [rev1.clone(), rev2.clone(), rev3.clone(), rev4.clone()];

            assert_eq!(
                Revision::compute_next_rev(refs.iter(), &InstanceId::from_string("c")),
                Revision::from_value(json!({ "a": 2, "b": 2, "c": 3 }))?
            );
        }

        Ok(())
    }

    #[test]
    fn test_revision_to_file_name() -> Result<()> {
        {
            let rev1 = Revision::from_value(json!({ "a": 1, "b": 2 }))?;
            let rev2 = Revision::from_value(json!({ "b": 2, "a": 1 }))?;

            assert_eq!(Revision::from_file_name(&rev1.to_file_name())?, rev1);
            assert_eq!(Revision::from_file_name(&rev2.to_file_name())?, rev2);
        }

        Ok(())
    }

    #[test]
    fn test_revision_from_file_name() -> Result<()> {
        {
            let rev1 = Revision::from_value(json!({ "a": 1, "b": 2 }))?;
            let rev2 = Revision::from_value(json!({ "b": 2, "a": 1 }))?;

            assert_eq!(Revision::from_file_name(&rev1.to_file_name())?, rev1);
            assert_eq!(Revision::from_file_name(&rev2.to_file_name())?, rev2);
        }

        Ok(())
    }

    #[test]
    fn test_latest_rev_computer() {
        {
            let rev1 = Revision::from_value(json!({ "a": 1 })).unwrap();
            let rev2 = Revision::from_value(json!({ "a": 2 })).unwrap();

            let mut latest_rev_computer = LatestRevComputer::new();
            latest_rev_computer.update([&rev1, &rev2]);

            assert_eq!(latest_rev_computer.get(), HashSet::from_iter([&rev2]));
        }

        {
            let rev1 = Revision::from_value(json!({ "a": 1 })).unwrap();
            let rev2 = Revision::from_value(json!({ "b": 1 })).unwrap();

            let mut latest_rev_computer = LatestRevComputer::new();
            latest_rev_computer.update([&rev1, &rev2]);

            assert_eq!(
                latest_rev_computer.get(),
                HashSet::from_iter([&rev1, &rev2])
            );
        }

        {
            let rev1 = Revision::from_value(json!({ "a": 1 })).unwrap();
            let rev2 = Revision::from_value(json!({ "b": 1 })).unwrap();
            let rev3 = Revision::from_value(json!({ "a": 2, "b": 1 })).unwrap();

            let mut latest_rev_computer = LatestRevComputer::new();
            latest_rev_computer.update([&rev1, &rev2, &rev3]);

            assert_eq!(latest_rev_computer.get(), HashSet::from_iter([&rev3]));
        }
    }
}

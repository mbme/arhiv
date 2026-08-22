use core::fmt;
use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashSet},
};

use anyhow::{Context, Result, anyhow, ensure};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use super::instance_id::InstanceId;

/// A vector clock stored in canonical form.
///
/// Zero counters are omitted so equality, hashing, ordering, and serialization
/// all operate on the same representation.
#[derive(Serialize, Hash, Clone, Eq, PartialEq)]
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

    fn from_versions(mut versions: BTreeMap<InstanceId, u32>) -> Self {
        versions.retain(|_, version| *version > 0);
        Self(versions)
    }

    pub fn is_initial(&self) -> bool {
        self == Revision::INITIAL
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

    /// Compares revisions by vector-clock causality.
    #[must_use]
    pub fn causal_cmp(&self, other: &Self) -> VectorClockOrder {
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

    /// Compares the canonical revision representation independently of causal order.
    #[must_use]
    pub fn canonical_cmp(&self, other: &Self) -> Ordering {
        self.0.iter().cmp(other.0.iter())
    }

    /// Compares revisions for deterministic history display while preserving causality.
    #[must_use]
    pub fn history_cmp(&self, other: &Self) -> Ordering {
        let generation = |revision: &Self| {
            revision
                .0
                .values()
                .map(|&version| u64::from(version))
                .sum::<u64>()
        };

        generation(self)
            .cmp(&generation(other))
            .then_with(|| self.canonical_cmp(other))
    }

    #[must_use]
    pub fn is_concurrent_or_newer_than(&self, other: &Self) -> bool {
        matches!(
            self.causal_cmp(other),
            VectorClockOrder::After | VectorClockOrder::Concurrent
        )
    }

    #[must_use]
    pub fn is_concurrent_or_older_than(&self, other: &Self) -> bool {
        matches!(
            self.causal_cmp(other),
            VectorClockOrder::Before | VectorClockOrder::Concurrent
        )
    }

    #[must_use]
    pub fn is_concurrent_or_equal(&self, other: &Self) -> bool {
        matches!(
            self.causal_cmp(other),
            VectorClockOrder::Equal | VectorClockOrder::Concurrent
        )
    }

    #[must_use]
    pub fn is_concurrent(&self, other: &Self) -> bool {
        matches!(self.causal_cmp(other), VectorClockOrder::Concurrent)
    }

    #[must_use]
    pub fn is_older_than(&self, other: &Self) -> bool {
        matches!(self.causal_cmp(other), VectorClockOrder::Before)
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

    pub fn to_string(rev: &Revision) -> String {
        if rev.is_initial() {
            Revision::STAGED_STRING.to_string()
        } else {
            rev.serialize()
        }
    }

    pub fn to_safe_string(&self) -> String {
        let mut items: Vec<_> = self
            .0
            .iter()
            .map(|(id, version)| format!("{id}:{version}"))
            .collect();

        items.sort();

        items.join("-")
    }

    pub fn from_safe_string(value: &str) -> Result<Self> {
        if value.trim().is_empty() {
            return Ok(Revision::initial());
        }

        let map = value
            .split("-")
            .map(|segment| {
                let mut parts = segment.split(":");

                let id = parts.next().context(anyhow!(
                    "Failed to extract instance id from revision segment {segment}"
                ))?;
                let version = parts.next().context(anyhow!(
                    "Failed to extract version from revision segment {segment}"
                ))?;
                ensure!(
                    parts.next().is_none(),
                    "Got invalid revision segment {segment}"
                );

                let id: InstanceId = id.try_into().context("Failed to parse instance id")?;
                let version: u32 = version.parse().context("Failed to parse version")?;

                Ok((id, version))
            })
            .collect::<Result<_>>()
            .context(anyhow!("Failed to parse revision from safe string {value}"))?;

        Ok(Revision::from_versions(map))
    }

    pub fn from_value(value: Value) -> Result<Revision> {
        let result = serde_json::from_value::<Option<Revision>>(value)
            .context("failed to convert into Revision")?
            .unwrap_or_else(Revision::initial);

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
    pub fn compute_next_rev<'r>(
        revs: impl Iterator<Item = &'r Revision>,
        for_instance: &InstanceId,
    ) -> Revision {
        let mut max_rev = Self::merge_all(revs);

        max_rev.inc(for_instance);

        max_rev
    }

    /// Finds the unique latest revision older than every supplied revision.
    ///
    /// Multiple concurrent common ancestors have no single causal maximum, so
    /// this returns `None` rather than selecting one by an unrelated ordering.
    pub fn find_base_rev<'r>(
        revs: &HashSet<&'r Revision>,
        all_revs: impl Iterator<Item = &'r Revision>,
    ) -> Option<&'r Revision> {
        let common_ancestors = all_revs
            .filter(|rev| revs.iter().all(|item| rev.is_older_than(item)))
            .collect::<Vec<_>>();
        if common_ancestors.is_empty() {
            return None;
        }

        let mut latest = LatestRevComputer::new();
        latest.update(common_ancestors);
        let latest = latest.get();

        (latest.len() == 1).then(|| {
            latest
                .into_iter()
                .next()
                .expect("one revision is available")
        })
    }
}

impl<'de> Deserialize<'de> for Revision {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let versions = BTreeMap::deserialize(deserializer)?;
        Ok(Revision::from_versions(versions))
    }
}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for Revision {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.causal_cmp(other) {
            VectorClockOrder::Before => Some(Ordering::Less),
            VectorClockOrder::After => Some(Ordering::Greater),
            VectorClockOrder::Equal => Some(Ordering::Equal),
            VectorClockOrder::Concurrent => None,
        }
    }
}

impl Default for &Revision {
    fn default() -> Self {
        Revision::INITIAL
    }
}

impl fmt::Debug for Revision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<rev: {}>", self.to_safe_string())
    }
}

pub struct LatestRevComputer<'r>(HashSet<&'r Revision>);

impl LatestRevComputer<'_> {
    pub fn new() -> Self {
        let mut revs = HashSet::new();
        revs.insert(Revision::INITIAL);

        Self(revs)
    }
}

impl Default for LatestRevComputer<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'r> LatestRevComputer<'r> {
    pub fn update(&mut self, new_revs: impl IntoIterator<Item = &'r Revision>) {
        for new_rev in new_revs.into_iter() {
            // skip new_rev if any existing rev is newer
            if self
                .0
                .iter()
                .any(|&rev| rev.causal_cmp(new_rev) == VectorClockOrder::After)
            {
                continue;
            }

            // remove all existing revs older than new_rev
            self.0
                .retain(|&rev| rev.causal_cmp(new_rev) != VectorClockOrder::Before);

            // insert new_rev if no equal rev exists
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
    use std::collections::{HashMap, HashSet};

    use anyhow::Result;
    use serde_json::json;

    use crate::entities::{InstanceId, revision::VectorClockOrder};

    use super::{LatestRevComputer, Revision};

    #[test]
    fn test_revision_inc() -> Result<()> {
        {
            let mut rev = Revision::from_value(json!({}))?;
            let instance_id = InstanceId::from_string("a").unwrap();

            rev.inc(&instance_id);

            assert_eq!(rev, Revision::from_value(json!({ "a": 1 }))?);
        }

        {
            let mut rev = Revision::from_value(json!({ "a": 1, "b": 2 }))?;
            let instance_id = InstanceId::from_string("a").unwrap();

            rev.inc(&instance_id);

            assert_eq!(rev, Revision::from_value(json!({ "a": 2, "b": 2 }))?);
        }
        Ok(())
    }

    #[test]
    fn test_revision_causal_cmp() -> Result<()> {
        {
            let rev1 = Revision::from_value(json!({ "a": 1, "b": 2 }))?;
            let rev2 = Revision::from_value(json!({ "a": 2, "b": 1 }))?;

            assert_eq!(rev1.causal_cmp(&rev2), VectorClockOrder::Concurrent);
            assert_eq!(rev2.causal_cmp(&rev1), VectorClockOrder::Concurrent);
        }

        {
            let rev1 = Revision::from_value(json!({ "a": 1, "b": 2 }))?;
            let rev2 = Revision::from_value(json!({ "a": 1, "b": 2 }))?;

            assert_eq!(rev1.causal_cmp(&rev2), VectorClockOrder::Equal);
            assert_eq!(rev2.causal_cmp(&rev1), VectorClockOrder::Equal);
        }

        {
            let rev1 = Revision::from_value(json!({ "a": 1, "b": 1 }))?;
            let rev2 = Revision::from_value(json!({ "a": 1, "b": 2 }))?;

            assert_eq!(rev1.causal_cmp(&rev2), VectorClockOrder::Before);
            assert_eq!(rev2.causal_cmp(&rev1), VectorClockOrder::After);
        }

        {
            let rev1 = Revision::from_value(json!({ "a": 1, }))?;
            let rev2 = Revision::from_value(json!({ "a": 1, "b": 2}))?;

            assert_eq!(rev1.causal_cmp(&rev2), VectorClockOrder::Before);
            assert_eq!(rev2.causal_cmp(&rev1), VectorClockOrder::After);
        }

        Ok(())
    }

    #[test]
    fn test_revision_partial_cmp() -> Result<()> {
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
            assert_eq!(rev4.partial_cmp(&rev2), None);
        }

        Ok(())
    }

    #[test]
    fn canonical_comparison_is_transitive_for_concurrent_and_causal_revisions() -> Result<()> {
        let rev_b = Revision::from_value(json!({ "b": 1 }))?;
        let rev_c = Revision::from_value(json!({ "c": 1 }))?;
        let rev_ac = Revision::from_value(json!({ "a": 1, "c": 2 }))?;

        assert!(rev_ac.canonical_cmp(&rev_b).is_lt());
        assert!(rev_b.canonical_cmp(&rev_c).is_lt());
        assert!(rev_ac.canonical_cmp(&rev_c).is_lt());

        Ok(())
    }

    #[test]
    fn history_comparison_preserves_causal_order() -> Result<()> {
        let ancestor = Revision::from_value(json!({ "b": 1 }))?;
        let descendant = Revision::from_value(json!({ "a": 1, "b": 1 }))?;

        assert!(descendant.canonical_cmp(&ancestor).is_lt());
        assert!(ancestor.history_cmp(&descendant).is_lt());
        assert!(descendant.history_cmp(&ancestor).is_gt());

        Ok(())
    }

    #[test]
    fn history_comparison_orders_concurrent_revisions_deterministically() -> Result<()> {
        let rev_a = Revision::from_value(json!({ "a": 1 }))?;
        let rev_b = Revision::from_value(json!({ "b": 1 }))?;

        assert_eq!(rev_a.history_cmp(&rev_b), rev_a.canonical_cmp(&rev_b));
        assert_eq!(rev_b.history_cmp(&rev_a), rev_b.canonical_cmp(&rev_a));

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
    fn test_revision_compute_next_rev() -> Result<()> {
        let rev1 = Revision::from_value(json!({ "a": 1, "b": 1 }))?;
        let rev2 = Revision::from_value(json!({ "a": 1, "b": 2 }))?;
        let rev3 = Revision::from_value(json!({ "a": 2, "b": 1 }))?;

        {
            let refs = [rev1.clone(), rev2.clone(), rev3.clone()];

            assert_eq!(
                Revision::compute_next_rev(refs.iter(), &InstanceId::from_string("a").unwrap()),
                Revision::from_value(json!({ "a": 3, "b": 2 }))?
            );
        }

        {
            let refs = [rev1.clone(), rev2.clone(), rev3.clone()];

            assert_eq!(
                Revision::compute_next_rev(refs.iter(), &InstanceId::from_string("c").unwrap()),
                Revision::from_value(json!({ "a": 2, "b": 2, "c": 1 }))?
            );
        }

        {
            let rev4 = Revision::from_value(json!({ "a": 1, "b": 1, "c": 2 }))?;

            let refs = [rev1.clone(), rev2.clone(), rev3.clone(), rev4.clone()];

            assert_eq!(
                Revision::compute_next_rev(refs.iter(), &InstanceId::from_string("c").unwrap()),
                Revision::from_value(json!({ "a": 2, "b": 2, "c": 3 }))?
            );
        }

        Ok(())
    }

    #[test]
    fn test_revision_to_safe_string() -> Result<()> {
        {
            let rev1 = Revision::from_value(json!({ "a": 1, "b": 2 }))?;
            let rev2 = Revision::from_value(json!({ "b": 2, "a": 1 }))?;

            assert_eq!(Revision::from_safe_string(&rev1.to_safe_string())?, rev1);
            assert_eq!(Revision::from_safe_string(&rev2.to_safe_string())?, rev2);
        }

        Ok(())
    }

    #[test]
    fn test_revision_from_safe_string() -> Result<()> {
        {
            let rev0 = Revision::initial();
            let rev1 = Revision::from_value(json!({ "a": 1, "b": 2 }))?;
            let rev2 = Revision::from_value(json!({ "b": 2, "a": 1 }))?;

            assert_eq!(Revision::from_safe_string(&rev0.to_safe_string())?, rev0);
            assert_eq!(Revision::from_safe_string(&rev1.to_safe_string())?, rev1);
            assert_eq!(Revision::from_safe_string(&rev2.to_safe_string())?, rev2);
        }

        Ok(())
    }

    #[test]
    fn parsing_normalizes_zero_versions() -> Result<()> {
        let canonical = Revision::from_value(json!({ "a": 1 }))?;
        let from_json: Revision = serde_json::from_value(json!({ "a": 1, "b": 0 }))?;
        let from_safe_string = Revision::from_safe_string("a:1-b:0")?;

        assert_eq!(from_json, canonical);
        assert_eq!(from_safe_string, canonical);
        assert_eq!(from_json.to_safe_string(), "a:1");
        assert_eq!(from_safe_string.to_safe_string(), "a:1");

        Ok(())
    }

    #[test]
    fn equal_revisions_are_interchangeable_hash_map_keys() -> Result<()> {
        let canonical = Revision::from_value(json!({ "a": 1 }))?;
        let with_zero = Revision::from_safe_string("a:1-b:0")?;
        let revisions = HashMap::from([(canonical, "snapshot")]);

        assert_eq!(revisions.get(&with_zero), Some(&"snapshot"));

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

        // keep only latest revision of each conflicting branch
        {
            let rev1 = Revision::from_value(json!({ "a": 1 })).unwrap();
            let rev2 = Revision::from_value(json!({ "b": 1 })).unwrap();
            let rev3 = Revision::from_value(json!({ "b": 2 })).unwrap();

            {
                let mut latest_rev_computer = LatestRevComputer::new();
                latest_rev_computer.update([&rev1, &rev2, &rev3]);

                assert_eq!(
                    latest_rev_computer.get(),
                    HashSet::from_iter([&rev1, &rev3])
                );
            }

            // different order
            {
                let mut latest_rev_computer = LatestRevComputer::new();
                latest_rev_computer.update([&rev3, &rev1, &rev2]);

                assert_eq!(
                    latest_rev_computer.get(),
                    HashSet::from_iter([&rev1, &rev3])
                );
            }
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

    #[test]
    fn test_find_base_rev() {
        let rev1 = Revision::from_value(json!({ "a": 1 })).unwrap();
        let rev2 = Revision::from_value(json!({ "a": 1, "b": 1 })).unwrap();
        let rev3 = Revision::from_value(json!({ "a": 2, "b": 1 })).unwrap();
        let rev4 = Revision::from_value(json!({ "a": 1, "b": 2 })).unwrap();
        let rev5 = Revision::from_value(json!({ "a": 1, "b": 3 })).unwrap();

        let all_revs: HashSet<&Revision> = HashSet::from_iter([&rev1, &rev2, &rev3, &rev4, &rev5]);

        let mut latest_rev_computer = LatestRevComputer::new();
        latest_rev_computer.update(all_revs.iter().copied());

        let latest_revs = latest_rev_computer.get();

        let base_rev = Revision::find_base_rev(&latest_revs, all_revs.iter().copied());

        assert_eq!(base_rev, Some(&rev2));
    }

    #[test]
    fn find_base_rev_rejects_multiple_concurrent_common_ancestors() {
        let base_a = Revision::from_value(json!({ "a": 1 })).unwrap();
        let base_b = Revision::from_value(json!({ "b": 1 })).unwrap();
        let head_a = Revision::from_value(json!({ "a": 2, "b": 1 })).unwrap();
        let head_b = Revision::from_value(json!({ "a": 1, "b": 2 })).unwrap();
        let latest_revs = HashSet::from([&head_a, &head_b]);

        assert_eq!(
            Revision::find_base_rev(&latest_revs, [&base_a, &base_b].into_iter()),
            None
        );
    }
}

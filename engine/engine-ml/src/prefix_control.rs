use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrefixCacheEntry {
    pub active_hash: String,
    pub hardware_channel: String,
    pub token_count: u32,
    pub last_used_epoch: u64,
}

#[derive(Debug, Default)]
pub struct PrefixCacheController {
    active_hashes: DashMap<String, PrefixCacheEntry>,
    epoch: AtomicU64,
}

impl PrefixCacheController {
    pub fn new() -> Self {
        Self {
            active_hashes: DashMap::new(),
            epoch: AtomicU64::new(1),
        }
    }

    pub fn register_prefix(
        &self,
        active_hash: String,
        hardware_channel: String,
        token_count: u32,
    ) -> PrefixCacheEntry {
        let entry = PrefixCacheEntry {
            active_hash: active_hash.clone(),
            hardware_channel,
            token_count,
            last_used_epoch: self.next_epoch(),
        };
        self.active_hashes.insert(active_hash, entry.clone());
        entry
    }

    pub fn touch_prefix(&self, active_hash: &str) -> Option<PrefixCacheEntry> {
        let epoch = self.next_epoch();
        self.active_hashes.get_mut(active_hash).map(|mut entry| {
            entry.last_used_epoch = epoch;
            entry.clone()
        })
    }

    pub fn remove_prefix(&self, active_hash: &str) -> Option<PrefixCacheEntry> {
        self.active_hashes
            .remove(active_hash)
            .map(|(_, entry)| entry)
    }

    pub fn get_prefix(&self, active_hash: &str) -> Option<PrefixCacheEntry> {
        self.active_hashes
            .get(active_hash)
            .map(|entry| entry.clone())
    }

    pub fn active_hashes_for_channel(&self, hardware_channel: &str) -> Vec<String> {
        self.active_hashes
            .iter()
            .filter(|entry| entry.hardware_channel == hardware_channel)
            .map(|entry| entry.active_hash.clone())
            .collect()
    }

    pub fn len(&self) -> usize {
        self.active_hashes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.active_hashes.is_empty()
    }

    fn next_epoch(&self) -> u64 {
        self.epoch.fetch_add(1, Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::PrefixCacheController;

    #[test]
    fn tracks_owned_hashes_across_hardware_channels() {
        let controller = PrefixCacheController::new();
        let hash = String::from("prefix-hash-a");
        let channel = String::from("npu-0");

        let entry = controller.register_prefix(hash.clone(), channel.clone(), 4096);

        assert_eq!(entry.active_hash, hash);
        assert_eq!(entry.hardware_channel, channel);
        assert_eq!(
            controller.active_hashes_for_channel("npu-0"),
            vec![String::from("prefix-hash-a")]
        );
    }

    #[test]
    fn touch_updates_epoch_without_reallocating_key_contract() {
        let controller = PrefixCacheController::new();
        controller.register_prefix("hash-b".to_string(), "gpu-1".to_string(), 128);
        let before = controller.get_prefix("hash-b").expect("entry exists");

        let after = controller.touch_prefix("hash-b").expect("touch succeeds");

        assert_eq!(after.active_hash, "hash-b");
        assert!(after.last_used_epoch > before.last_used_epoch);
    }

    #[test]
    fn removes_prefix_by_borrowed_hash() {
        let controller = PrefixCacheController::new();
        controller.register_prefix("hash-c".to_string(), "npu-1".to_string(), 512);

        let removed = controller.remove_prefix("hash-c").expect("remove succeeds");

        assert_eq!(removed.active_hash, "hash-c");
        assert!(controller.is_empty());
    }
}

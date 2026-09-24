use std::collections::VecDeque;
use std::sync::Mutex;

/// How many rendered images are kept. Re-showing a chart is common, but the
/// images are large enough that keeping many is not worth the memory.
const CAPACITY: usize = 8;

/// Recently rendered images, keyed by chart identity and requested size.
#[derive(Default)]
pub struct RasterCache {
    entries: Mutex<VecDeque<(String, Vec<u8>)>>,
}

impl RasterCache {
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let entries = self
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        entries
            .iter()
            .find(|(entry_key, _)| entry_key == key)
            .map(|(_, png)| png.clone())
    }

    pub fn put(&self, key: String, png: Vec<u8>) {
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if entries.iter().any(|(entry_key, _)| *entry_key == key) {
            return;
        }

        entries.push_back((key, png));
        while entries.len() > CAPACITY {
            entries.pop_front();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{RasterCache, CAPACITY};

    #[test]
    fn returns_what_was_stored() {
        let cache = RasterCache::default();
        cache.put("a".to_string(), vec![1, 2, 3]);

        assert_eq!(cache.get("a"), Some(vec![1, 2, 3]));
        assert_eq!(cache.get("b"), None);
    }

    #[test]
    fn drops_the_oldest_entry_past_capacity() {
        let cache = RasterCache::default();
        for index in 0..CAPACITY + 1 {
            cache.put(index.to_string(), vec![index as u8]);
        }

        assert_eq!(cache.get("0"), None);
        assert_eq!(cache.get(&CAPACITY.to_string()), Some(vec![CAPACITY as u8]));
    }
}

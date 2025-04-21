extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use core::clone::Clone;
use core::fmt::Display;
use core::hash::{Hash, Hasher};

struct SimpleHasher(u64);

impl Hasher for SimpleHasher {
    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.0 = self.0.wrapping_mul(31).wrapping_add(byte as u64);
        }
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

pub struct HashMap <K, V> {
    buckets: Vec<Vec<(K, V)>>,
    capacity: usize,
}

impl<'a, K, V> HashMap<K, V> 
    where K: Hash + Eq + Clone + Display,
          V: Clone + Display
{
    pub fn new() -> Self {
        let capacity = 32;
        HashMap::<K, V> {
            buckets: vec![Vec::new(); capacity],
            capacity
        }
    }

    fn hash_key(&self, key: &K) -> u64 {
        let mut hasher = SimpleHasher(0);
        key.hash(&mut hasher);
        hasher.finish()
    }

    pub fn insert(&mut self, key: K, value: V) {
        let index = (self.hash_key(&key) % self.capacity as u64) as usize;
        let bucket = &mut self.buckets[index];
        // println!("Inserting key: {} into bucket index: {}", key, index);
        for (k, v) in bucket.iter_mut() {
            if *k == key {
                *v = value;
                return;
            }
        }
        bucket.push((key, value));
    }
    
    pub fn get(&self, key: &K) -> Option<&V> {
        let index = (self.hash_key(key) % self.capacity as u64) as usize;
        self.buckets[index]
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
            .or_else(|| {
                None
            })
    }

    pub fn iter(&self) -> HashMapIterator<'_, K, V> {
        HashMapIterator::new(self)
    }
}

pub struct HashMapIterator<'a, K, V> {
    hashmap: &'a HashMap<K, V>,
    bucket_index: usize,
    item_index: usize,
}

impl<'a, K, V> HashMapIterator<'a, K, V> {
    pub fn new(hashmap: &'a HashMap<K, V>) -> Self {
        HashMapIterator {
            hashmap,
            bucket_index: 0,
            item_index: 0,
        }
    }
}

impl<'a, K, V> Iterator for HashMapIterator<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        while self.bucket_index < self.hashmap.buckets.len() {
            let bucket = &self.hashmap.buckets[self.bucket_index];
            if self.item_index < bucket.len() {
                let item = &bucket[self.item_index];
                self.item_index += 1;
                return Some((&item.0, &item.1));
            } else {
                self.bucket_index += 1;
                self.item_index = 0;
            }
        }
        None
    }
}

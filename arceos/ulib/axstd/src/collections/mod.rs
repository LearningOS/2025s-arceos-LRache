mod hashmap;

#[cfg(feature = "alloc")]
pub use hashmap::HashMap;
#[cfg(feature = "alloc")]
pub use hashmap::HashMapIterator;

#![allow(unaligned_references)]

use dict::TestMap;
use phf::PhfHash;
use phf_shared::PhfBorrow;

mod dict;

impl<K, V> TestMap<K, V> {
    fn get_entry<T: ?Sized>(&self, key: &T) -> Option<(&K, &V)>
    where
        T: Eq + PhfHash,
        K: PhfBorrow<T>,
    {
        if self.disps.is_empty() {
            return None;
        } //Prevent panic on empty map
        let hashes = phf_shared::hash(key, &self.key);
        let index = phf_shared::get_index(&hashes, self.disps, self.entries.len());
        let entry = &self.entries[index as usize];
        let b: &T = entry.0.borrow();
        if b == key {
            Some((&entry.0, &entry.1))
        } else {
            None
        }
    }
}

pub fn lookup_kanji(kanji: &str) -> Option<&&[(&str, &str)]> {
    // dict::KANJI_DICT.get(kanji)
    dict::KANJI_DICT.get_entry(kanji).map(|entry| entry.1)
}

pub fn lookup_syn(c: &char) -> Option<&char> {
    dict::SYN_DICT.get(c)
}

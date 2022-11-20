use byteorder::{ByteOrder, LittleEndian};
use phf::PhfHash;

const LEN_ENTRY: usize = 5;
const START_DISPS: usize = 16;

pub struct PhfMap {
    data: &'static [u8],
    key: u64,
    n_disps: u32,
    n_entries: u32,
    start_entries: usize,
    start_entry_data: usize,
}

pub trait Decodable {
    fn decode(data: &'static [u8]) -> Self;
}

impl PhfMap {
    pub fn new(data: &'static [u8]) -> Self {
        let n_disps = LittleEndian::read_u32(&data[8..12]);
        let n_entries = LittleEndian::read_u32(&data[12..16]);
        let start_entries = START_DISPS + n_disps as usize * 8;
        let start_entry_data = start_entries + n_entries as usize * LEN_ENTRY;

        Self {
            data,
            key: LittleEndian::read_u64(&data[0..8]),
            n_disps,
            n_entries,
            start_entries,
            start_entry_data,
        }
    }

    fn displacement(&self, i: usize) -> (u32, u32) {
        let start = START_DISPS + i * 8;
        (
            LittleEndian::read_u32(&self.data[start..start + 4]),
            LittleEndian::read_u32(&self.data[start + 4..start + 8]),
        )
    }

    fn entry(&self, i: usize) -> (u32, u8, u8) {
        let start = self.start_entries + i * LEN_ENTRY;
        (
            LittleEndian::read_u24(&self.data[start..start + 3]),
            self.data[start + 3],
            self.data[start + 4],
        )
    }

    fn entry_data<K: Decodable, V: Decodable>(&self, i: usize) -> (K, V) {
        let (i, k_len, v_len) = self.entry(i);

        let k_start = self.start_entry_data + i as usize;
        let v_start = k_start + k_len as usize;
        let k_bytes = &self.data[k_start..v_start];
        let v_bytes = &self.data[v_start..v_start + v_len as usize];

        (K::decode(k_bytes), V::decode(v_bytes))
    }

    pub fn get<K: PhfHash + Eq + Decodable, V: Decodable>(&self, key: K) -> Option<V> {
        let hashes = phf_shared::hash(&key, &self.key);
        let (d1, d2) = self.displacement((hashes.g % self.n_disps) as usize);
        let i = phf_shared::displace(hashes.f1, hashes.f2, d1, d2) % self.n_entries;
        let (k, v) = self.entry_data::<K, V>(i as usize);
        if k == key {
            Some(v)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{KanjiString, Readings};

    use super::*;

    #[test]
    fn lookup() {
        let phfmap = PhfMap::new(crate::KANJI_DICT);

        dbg!(phfmap.get::<KanjiString, Readings>(KanjiString::new("描")));
    }

    #[test]
    fn utf16() {
        let t = "Abc 描 😅";

        let bytes = t
            .encode_utf16()
            .flat_map(|c16| [((c16 & 0xff00) >> 8) as u8, (c16 & 0xff) as u8])
            .collect::<Vec<_>>();

        let decoded = String::from_utf16(
            &bytes
                .chunks_exact(2)
                .map(|c| ((c[0] as u16) << 8) | c[1] as u16)
                .collect::<Vec<_>>(),
        )
        .unwrap();

        assert_eq!(decoded, t);
    }
}

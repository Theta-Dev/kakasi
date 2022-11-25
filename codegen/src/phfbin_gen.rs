use phf_shared::PhfHash;

/// A builder for the `phf::Map` type.
#[derive(Default)]
pub struct Map<K, V> {
    keys: Vec<K>,
    values: Vec<V>,
}

pub trait Encodable {
    fn encode(&self) -> Vec<u8>;
}

impl Encodable for String {
    fn encode(&self) -> Vec<u8> {
        self.as_bytes().into()
    }
}

impl<K: Encodable + PhfHash, V: Encodable> Map<K, V> {
    /// Adds an entry to the builder.
    ///
    /// `value` will be written exactly as provided in the constructed source.
    pub fn entry(&mut self, key: K, value: V) -> &mut Map<K, V> {
        self.keys.push(key);
        self.values.push(value);
        self
    }

    pub fn build(&self) -> Vec<u8> {
        let hashstate = phf_generator::generate_hash(&self.keys);
        let n_disps: u32 = hashstate.disps.len().try_into().unwrap();
        let n_entries: u32 = self.values.len().try_into().unwrap();

        // Header
        let mut header = Vec::new();
        header.extend(hashstate.key.to_le_bytes()); // key
        header.extend(n_disps.to_le_bytes()); // n_disps
        header.extend(n_entries.to_le_bytes()); // n_entries

        // Displacements
        let mut displacements = Vec::new();
        hashstate.disps.iter().for_each(|(a, b)| {
            displacements.extend(a.to_le_bytes());
            displacements.extend(b.to_le_bytes());
        });

        // Entries
        let mut entries = Vec::new();
        let mut edata: Vec<u8> = Vec::new();

        for mapped_key in hashstate.map {
            let k = self.keys[mapped_key].encode();
            let v = self.values[mapped_key].encode();

            let i_bts = edata.len().to_le_bytes();
            assert_eq!(i_bts[3], 0, "index longer than 24bit");
            entries.extend(&i_bts[0..3]);

            let k_len: u8 = k.len().try_into().unwrap();
            entries.push(k_len);
            let v_len: u8 = v.len().try_into().unwrap();
            entries.push(v_len);

            edata.extend(k);
            edata.extend(v);
        }

        // Combine file
        header.append(&mut displacements);
        header.append(&mut entries);
        header.append(&mut edata);
        header
    }
}

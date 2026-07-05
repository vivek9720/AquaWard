use crate::cursor::ByteCursor;
use crate::error::Result;
use crate::model::{Dictionary, DictionaryEntry};

pub fn parse_dictionary(data: &[u8]) -> Result<Dictionary> {
    let mut cursor = ByteCursor::new(data);
    let declared = cursor.read_u16().unwrap_or(0) as usize;
    let mut dictionary = Dictionary::default();
    let limit = declared.min(2048);
    for _ in 0..limit {
        if cursor.remaining() < 4 {
            break;
        }
        let kind = cursor.read_u8()?;
        let key_len = cursor.read_u8()? as usize;
        let value_len = cursor.read_u16()? as usize;
        if cursor.remaining() < key_len.saturating_add(value_len) {
            break;
        }
        let key = normalize_key(&String::from_utf8_lossy(cursor.read_exact(key_len)?));
        let value = cursor.read_exact(value_len)?.to_vec();
        dictionary.insert(DictionaryEntry { kind, key, value });
    }
    Ok(dictionary)
}

pub fn normalize_key(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut dash = false;
    for ch in input.chars() {
        let next = if ch.is_ascii_alphanumeric() {
            dash = false;
            ch.to_ascii_lowercase()
        } else if !dash {
            dash = true;
            '-'
        } else {
            continue;
        };
        out.push(next);
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

impl Dictionary {
    pub fn insert(&mut self, entry: DictionaryEntry) {
        let index = self.entries.len();
        self.index.insert(entry.key.clone(), index);
        self.entries.push(entry);
    }

    pub fn get(&self, key: &str) -> Option<&DictionaryEntry> {
        let key = normalize_key(key);
        self.index.get(&key).and_then(|&idx| self.entries.get(idx))
    }

    pub fn value_string(&self, key: &str) -> Option<String> {
        self.get(key)
            .map(|entry| String::from_utf8_lossy(&entry.value).into_owned())
    }

    pub fn value_by_index(&self, index: usize) -> Option<String> {
        self.entries
            .get(index)
            .map(|entry| String::from_utf8_lossy(&entry.value).into_owned())
    }

    pub fn compacted(&self, kind_mask: u8) -> Dictionary {
        let mut out = Dictionary::default();
        for entry in self
            .entries
            .iter()
            .filter(|entry| entry.kind & kind_mask != 0)
        {
            out.insert(entry.clone());
        }
        out
    }
}

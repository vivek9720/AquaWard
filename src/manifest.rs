use crate::error::Result;
use crate::model::Manifest;

pub fn parse_manifest(data: &[u8]) -> Result<Manifest> {
    let text = String::from_utf8_lossy(data);
    let mut manifest = Manifest::default();
    for raw_line in text.lines().take(2048) {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_ascii_lowercase();
            let value = value.trim().to_string();
            if key == "crew" {
                manifest.crews.push(value.clone());
            }
            if key == "incident" {
                if let Ok(code) = value.parse::<u16>() {
                    manifest.incident_codes.push(code);
                }
            }
            if key.starts_with("flag.") && boolish(&value) {
                manifest.flags = manifest.flags.wrapping_add(hash_flag(&key));
            }
            manifest.fields.insert(key, value);
        } else {
            let key = line.to_ascii_lowercase();
            manifest.fields.insert(key, String::new());
        }
    }
    Ok(manifest)
}

pub fn boolish(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on" | "enabled"
    )
}

fn hash_flag(key: &str) -> u32 {
    let mut acc = 0x811c_9dc5u32;
    for byte in key.bytes() {
        acc ^= byte as u32;
        acc = acc.wrapping_mul(0x0100_0193);
    }
    acc
}

impl Manifest {
    pub fn bool_field(&self, key: &str) -> bool {
        self.fields
            .get(&key.to_ascii_lowercase())
            .map(|v| boolish(v))
            .unwrap_or(false)
    }

    pub fn int_field(&self, key: &str) -> Option<u32> {
        self.fields
            .get(&key.to_ascii_lowercase())
            .and_then(|v| v.parse().ok())
    }
}

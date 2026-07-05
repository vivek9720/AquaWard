use crate::cursor::ByteCursor;
use crate::error::Result;
use crate::model::{Dictionary, JournalEntry};

pub fn parse_journal(data: &[u8], dictionary: &Dictionary) -> Result<Vec<JournalEntry>> {
    if data.starts_with(b"JTXT") {
        return parse_text_journal(&data[4..], dictionary);
    }
    parse_binary_journal(data, dictionary)
}

fn parse_binary_journal(data: &[u8], dictionary: &Dictionary) -> Result<Vec<JournalEntry>> {
    let mut cursor = ByteCursor::new(data);
    let declared = cursor.read_u16().unwrap_or(0) as usize;
    let mut entries = Vec::with_capacity(declared.min(256));
    for _ in 0..declared.min(4096) {
        if cursor.remaining() < 10 {
            break;
        }
        let ts = cursor.read_u32()?;
        let station = cursor.read_u16()?;
        let severity = cursor.read_u8()?;
        let dict_index = cursor.read_u16()? as usize;
        let text_len = cursor.read_u8()? as usize;
        if cursor.remaining() < text_len {
            break;
        }
        let suffix = String::from_utf8_lossy(cursor.read_exact(text_len)?).into_owned();
        let prefix = dictionary
            .value_by_index(dict_index)
            .unwrap_or_else(|| "field note".to_string());
        entries.push(JournalEntry {
            ts,
            station,
            severity,
            text: format!("{prefix}: {suffix}"),
        });
    }
    Ok(entries)
}

fn parse_text_journal(data: &[u8], dictionary: &Dictionary) -> Result<Vec<JournalEntry>> {
    let text = String::from_utf8_lossy(data);
    let mut entries = Vec::new();
    for (idx, line) in text.lines().enumerate().take(1024) {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 4 {
            let ts = parts[0].parse().unwrap_or(idx as u32);
            let station = parts[1].parse().unwrap_or(0);
            let severity = parts[2].parse().unwrap_or(1);
            let text = dictionary
                .value_string(parts[3])
                .unwrap_or_else(|| parts[3].to_string());
            entries.push(JournalEntry {
                ts,
                station,
                severity,
                text,
            });
        }
    }
    Ok(entries)
}

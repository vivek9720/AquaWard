use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
pub struct Bundle {
    pub version: u16,
    pub flags: u16,
    pub salt: u16,
    pub sections: Vec<Section>,
    pub dictionary: Dictionary,
    pub manifest: Manifest,
    pub topology: TopologyMap,
    pub sensor_frames: Vec<SensorFrame>,
    pub plume_tiles: Vec<PlumeTile>,
    pub ledger_ops: Vec<LedgerOp>,
    pub journals: Vec<JournalEntry>,
    pub scripts: Vec<ScriptProgram>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Section {
    pub kind: u8,
    pub flags: u8,
    pub id: u16,
    pub checksum: u32,
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, Default)]
pub struct Dictionary {
    pub entries: Vec<DictionaryEntry>,
    pub index: BTreeMap<String, usize>,
}

#[derive(Clone, Debug)]
pub struct DictionaryEntry {
    pub kind: u8,
    pub key: String,
    pub value: Vec<u8>,
}

#[derive(Clone, Debug, Default)]
pub struct Manifest {
    pub fields: BTreeMap<String, String>,
    pub crews: Vec<String>,
    pub incident_codes: Vec<u16>,
    pub flags: u32,
}

#[derive(Clone, Debug, Default)]
pub struct TopologyMap {
    pub nodes: Vec<TopologyNode>,
    pub edges: Vec<TopologyEdge>,
    pub zones: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct TopologyNode {
    pub id: u16,
    pub kind: u8,
    pub zone: u8,
    pub label: String,
    pub flags: u16,
    pub x: i16,
    pub y: i16,
    pub demand: u16,
}

#[derive(Clone, Debug)]
pub struct TopologyEdge {
    pub from: u16,
    pub to: u16,
    pub material: u8,
    pub flags: u16,
    pub diameter_mm: u16,
    pub length_m: u16,
}

#[derive(Clone, Debug)]
pub struct SensorFrame {
    pub station_id: u16,
    pub channel: u8,
    pub profile: u16,
    pub flags: u16,
    pub base: i32,
    pub scale: i16,
    pub samples: Vec<i32>,
    pub label_hint: Option<String>,
}

#[derive(Clone, Debug)]
pub struct PlumeTile {
    pub width: u16,
    pub height: u16,
    pub flags: u16,
    pub anchor_x: u16,
    pub anchor_y: u16,
    pub phase: u8,
    pub cells: Vec<i16>,
}

#[derive(Clone, Debug)]
pub struct LedgerOp {
    pub opcode: u8,
    pub route: u16,
    pub station: u16,
    pub amount: u32,
    pub flags: u16,
}

#[derive(Clone, Debug)]
pub struct JournalEntry {
    pub ts: u32,
    pub station: u16,
    pub severity: u8,
    pub text: String,
}

#[derive(Clone, Debug, Default)]
pub struct ScriptProgram {
    pub version: u8,
    pub flags: u16,
    pub ops: Vec<ScriptOp>,
}

#[derive(Clone, Debug)]
pub enum ScriptOp {
    PushI32(i32),
    PushStation(u16),
    Add,
    Sub,
    Blend(u8),
    Pin,
    Reserve(u16),
    Compact,
    ReadPins,
    Drop(u8),
    Emit(u8),
    Noop(u8),
}

#[derive(Clone, Debug, Default)]
pub struct AnalysisReport {
    pub score: u64,
    pub findings: Vec<AnalysisFinding>,
    pub counters: BTreeMap<String, u64>,
}

#[derive(Clone, Debug)]
pub struct AnalysisFinding {
    pub code: String,
    pub severity: u8,
    pub detail: String,
}

impl Bundle {
    pub fn push_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }
}

impl AnalysisReport {
    pub fn bump(&mut self, key: &str, by: u64) {
        *self.counters.entry(key.to_string()).or_insert(0) += by;
    }

    pub fn finding(&mut self, code: &str, severity: u8, detail: impl Into<String>) {
        self.findings.push(AnalysisFinding {
            code: code.to_string(),
            severity,
            detail: detail.into(),
        });
    }
}

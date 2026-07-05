//! AquaWard decodes offline water-safety incident bundles.
//!
//! Field teams often collect pressure, chemistry, and topology evidence
//! while disconnected from central services. This crate accepts those
//! structured bundles and produces a compact analysis report.

pub mod analysis;
pub mod catalog;
pub mod checksum;
pub mod codec;
pub mod cursor;
pub mod dictionary;
pub mod envelope;
pub mod error;
pub mod frame;
pub mod journal;
pub mod ledger;
pub mod manifest;
pub mod model;
pub mod plume;
pub mod reassembler;
pub mod report;
pub mod script;
pub mod sensor;
pub mod stream;
pub mod topology;
pub mod validate;

pub use analysis::Analyzer;
pub use envelope::BundleDecoder;
pub use error::{AquaError, Result};
pub use model::{AnalysisFinding, AnalysisReport, Bundle, Manifest, PlumeTile, ScriptProgram};

pub fn parse_bundle(data: &[u8]) -> Result<Bundle> {
    BundleDecoder::new().parse(data)
}

pub fn decode_and_analyze_bundle(data: &[u8]) -> Result<AnalysisReport> {
    let bundle = parse_bundle(data)?;
    Ok(Analyzer::new().analyze(&bundle))
}

pub fn decode_stream(data: &[u8]) -> Result<Bundle> {
    let mut decoder = stream::StreamDecoder::new();
    decoder.feed(data)?;
    decoder.finish()
}

pub fn decode_stream_and_analyze(data: &[u8]) -> Result<AnalysisReport> {
    let bundle = decode_stream(data)?;
    Ok(Analyzer::new().analyze(&bundle))
}

pub fn run_script_bytes(data: &[u8]) -> Result<script::ExecutionReport> {
    script::compile_and_run(data)
}

pub fn analyze_plume_bytes(data: &[u8]) -> Result<u64> {
    let tile = plume::parse_plume_tile(data)?;
    Ok(plume::PlumeModel::new().score_tile_without_topology(&tile))
}

pub fn replay_ledger_bytes(data: &[u8]) -> Result<u64> {
    ledger::replay_ledger_bytes(data)
}

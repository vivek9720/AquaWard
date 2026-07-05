use crate::catalog;
use crate::ledger::LeaseTable;
use crate::model::{AnalysisReport, Bundle};
use crate::plume::PlumeModel;
use crate::script::Vm;
use crate::sensor::CalibrationBank;
use crate::validate;

#[derive(Clone, Debug, Default)]
pub struct Analyzer;

impl Analyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze(&self, bundle: &Bundle) -> AnalysisReport {
        let mut report = AnalysisReport::default();
        report.bump("sections", bundle.sections.len() as u64);
        report.bump("sensors", bundle.sensor_frames.len() as u64);
        report.bump("plumes", bundle.plume_tiles.len() as u64);
        report.score ^= bundle.topology.pressure_score();

        for warning in &bundle.warnings {
            report.finding("bundle.warning", 2, warning.clone());
        }
        for issue in validate::validate_bundle(bundle) {
            report.finding("bundle.validation", issue.0, issue.1);
        }

        let calibration = CalibrationBank::from_frames(
            &bundle.sensor_frames,
            &bundle.dictionary,
            &bundle.manifest,
        );
        report.score ^= calibration.audit_score();

        let plume = PlumeModel::new();
        for tile in &bundle.plume_tiles {
            report.score ^= plume.score_tile(tile, Some(&bundle.topology));
        }

        let mut leases = LeaseTable::default();
        report.score ^= leases.apply_all(&bundle.ledger_ops, Some(&bundle.topology));

        for program in &bundle.scripts {
            let script_report = Vm::default().run(program);
            report.bump("script.steps", script_report.steps as u64);
            report.score ^= script_report.score;
        }

        for code in &bundle.manifest.incident_codes {
            if let Some(pattern) = catalog::incidents::pattern_by_code(*code) {
                report.score ^= pattern.severity as u64 * pattern.persistence_hours as u64;
                if pattern.severity >= 8 {
                    report.finding("incident.high", pattern.severity, pattern.label);
                }
            }
        }

        report
    }
}

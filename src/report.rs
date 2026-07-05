use crate::model::AnalysisReport;

pub fn render_text(report: &AnalysisReport) -> String {
    let mut out = String::new();
    out.push_str(&format!("score={}\n", report.score));
    for (key, value) in &report.counters {
        out.push_str(&format!("counter.{key}={value}\n"));
    }
    for finding in &report.findings {
        out.push_str(&format!(
            "finding.{}[{}]={}\n",
            finding.code, finding.severity, finding.detail
        ));
    }
    out
}

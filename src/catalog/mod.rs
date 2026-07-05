pub mod chemistry;
pub mod hydraulics;
pub mod incidents;
pub mod protocols;
pub mod stations;

pub fn catalogue_score(seed: u16) -> u64 {
    let station = stations::station_by_id(1000 + seed % 1800)
        .map(|s| s.nominal_flow as u64)
        .unwrap_or(0);
    let chemical = chemistry::limit_by_code(2000 + seed % 1450)
        .map(|c| c.alarm_ppb as u64)
        .unwrap_or(0);
    let pipe = hydraulics::class_by_id(3000 + seed % 1500)
        .map(|p| p.friction as u64)
        .unwrap_or(0);
    let incident = incidents::pattern_by_code(4000 + seed % 1400)
        .map(|i| i.severity as u64)
        .unwrap_or(0);
    let field = protocols::field_by_tag(5000 + seed % 1300)
        .map(|f| f.risk as u64)
        .unwrap_or(0);
    station ^ chemical ^ pipe ^ incident ^ field
}

use crate::model::Bundle;

pub fn validate_bundle(bundle: &Bundle) -> Vec<(u8, String)> {
    let mut out = Vec::new();
    if bundle.dictionary.entries.is_empty() {
        out.push((1, "bundle has no dictionary".to_string()));
    }
    if bundle.topology.nodes.is_empty() && !bundle.sensor_frames.is_empty() {
        out.push((3, "sensor frames arrived without topology".to_string()));
    }
    if bundle
        .plume_tiles
        .iter()
        .any(|tile| tile.width == 0 || tile.height == 0)
    {
        out.push((4, "plume tile has an empty dimension".to_string()));
    }
    if bundle.ledger_ops.len() > 4096 {
        out.push((2, "ledger operation count is unusually high".to_string()));
    }
    out
}

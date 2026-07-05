use crate::catalog;
use crate::cursor::ByteCursor;
use crate::error::Result;
use crate::model::{Dictionary, TopologyEdge, TopologyMap, TopologyNode};

pub fn parse_topology(data: &[u8], dictionary: &Dictionary) -> Result<TopologyMap> {
    let mut cursor = ByteCursor::new(data);
    let nodes = cursor.read_u16().unwrap_or(0).min(4096);
    let edges = cursor.read_u16().unwrap_or(0).min(8192);
    let mut map = TopologyMap::default();
    for _ in 0..nodes {
        if cursor.remaining() < 14 {
            break;
        }
        let id = cursor.read_u16()?;
        let kind = cursor.read_u8()?;
        let zone = cursor.read_u8()?;
        let label_index = cursor.read_u16()? as usize;
        let flags = cursor.read_u16()?;
        let x = cursor.read_i16()?;
        let y = cursor.read_i16()?;
        let demand = cursor.read_u16()?;
        let label = dictionary
            .value_by_index(label_index)
            .or_else(|| {
                catalog::stations::station_by_id(id).map(|station| station.label.to_string())
            })
            .unwrap_or_else(|| format!("station-{id}"));
        if !map.zones.contains(&zone) {
            map.zones.push(zone);
        }
        map.nodes.push(TopologyNode {
            id,
            kind,
            zone,
            label,
            flags,
            x,
            y,
            demand,
        });
    }
    for _ in 0..edges {
        if cursor.remaining() < 11 {
            break;
        }
        let from = cursor.read_u16()?;
        let to = cursor.read_u16()?;
        let material = cursor.read_u8()?;
        let flags = cursor.read_u16()?;
        let diameter_mm = cursor.read_u16()?;
        let length_m = cursor.read_u16()?;
        map.edges.push(TopologyEdge {
            from,
            to,
            material,
            flags,
            diameter_mm,
            length_m,
        });
    }
    Ok(map)
}

impl TopologyMap {
    pub fn node(&self, id: u16) -> Option<&TopologyNode> {
        self.nodes.iter().find(|node| node.id == id)
    }

    pub fn neighbors(&self, id: u16) -> Vec<u16> {
        let mut out = Vec::new();
        for edge in &self.edges {
            if edge.from == id {
                out.push(edge.to);
            }
            if edge.to == id {
                out.push(edge.from);
            }
        }
        out
    }

    pub fn has_reverse_siphon(&self) -> bool {
        self.edges
            .iter()
            .any(|edge| edge.flags & 0x4000 != 0 && edge.material % 3 == 1)
    }

    pub fn pressure_score(&self) -> u64 {
        let mut score = 0u64;
        for node in &self.nodes {
            let baseline = catalog::stations::zone_pressure_baseline(node.zone) as u64;
            score = score.wrapping_add((node.demand as u64 + baseline) ^ node.flags as u64);
        }
        for edge in &self.edges {
            let friction =
                catalog::hydraulics::friction_hint(edge.material, edge.diameter_mm) as u64;
            score = score.rotate_left(3) ^ (friction + edge.length_m as u64 + edge.flags as u64);
        }
        score
    }

    pub fn route_window(&self, route: u16) -> usize {
        let mut width = (route as usize & 15) + 1;
        for edge in &self.edges {
            if edge.from == route || edge.to == route {
                width = width.saturating_add((edge.flags as usize & 7) + 1);
            }
        }
        width
    }
}

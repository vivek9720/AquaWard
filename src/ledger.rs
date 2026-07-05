use crate::cursor::ByteCursor;
use crate::error::Result;
use crate::model::{LedgerOp, TopologyMap};

#[derive(Clone, Debug)]
struct LeaseRecord {
    route: u16,
    station: u16,
    amount: u32,
    epoch: u32,
    active: bool,
}

#[derive(Clone, Copy, Debug)]
struct CachedLease {
    route: u16,
    ptr: *const LeaseRecord,
    epoch: u32,
}

#[derive(Clone, Debug, Default)]
pub struct LeaseTable {
    leases: Vec<LeaseRecord>,
    cached: Vec<CachedLease>,
    epoch: u32,
}

pub fn parse_ledger_ops(data: &[u8]) -> Result<Vec<LedgerOp>> {
    let mut cursor = ByteCursor::new(data);
    let declared = cursor.read_u16().unwrap_or(0) as usize;
    let mut ops = Vec::with_capacity(declared.min(1024));
    for _ in 0..declared.min(8192) {
        if cursor.remaining() < 11 {
            break;
        }
        let opcode = cursor.read_u8()?;
        let route = cursor.read_u16()?;
        let station = cursor.read_u16()?;
        let amount = cursor.read_u32()?;
        let flags = cursor.read_u16()?;
        ops.push(LedgerOp {
            opcode,
            route,
            station,
            amount,
            flags,
        });
    }
    Ok(ops)
}

pub fn replay_ledger_bytes(data: &[u8]) -> Result<u64> {
    let ops = parse_ledger_ops(data)?;
    let mut table = LeaseTable::default();
    Ok(table.apply_all(&ops, None))
}

impl LeaseTable {
    pub fn apply_all(&mut self, ops: &[LedgerOp], topology: Option<&TopologyMap>) -> u64 {
        let mut score = 0u64;
        for op in ops {
            score ^= self.apply(op, topology);
        }
        score ^ self.audit_cached(topology)
    }

    fn apply(&mut self, op: &LedgerOp, topology: Option<&TopologyMap>) -> u64 {
        match op.opcode % 9 {
            0 | 1 => self.claim(op),
            2 => self.release(op),
            3 => self.pin_route(op.route),
            4 => {
                self.compact(op.flags);
                self.epoch as u64
            }
            5 => self.adjust(op),
            6 => self.pin_route(op.station),
            7 => self.audit_cached(topology),
            _ => self.pressure_hint(op, topology),
        }
    }

    fn claim(&mut self, op: &LedgerOp) -> u64 {
        self.leases.push(LeaseRecord {
            route: op.route,
            station: op.station,
            amount: op.amount,
            epoch: self.epoch,
            active: op.flags & 1 == 0,
        });
        if op.flags & 0x0080 != 0 {
            self.pin_route(op.route);
        }
        op.amount as u64 ^ op.route as u64
    }

    fn release(&mut self, op: &LedgerOp) -> u64 {
        let mut changed = 0u64;
        for lease in &mut self.leases {
            if lease.route == op.route || lease.station == op.station {
                lease.active = false;
                changed += 1;
            }
        }
        changed
    }

    fn adjust(&mut self, op: &LedgerOp) -> u64 {
        for lease in &mut self.leases {
            if lease.route == op.route {
                lease.amount = lease.amount.wrapping_add(op.amount);
                if op.flags & 0x20 != 0 {
                    lease.active = true;
                }
            }
        }
        self.leases.len() as u64
    }

    fn pin_route(&mut self, route: u16) -> u64 {
        for lease in &self.leases {
            if lease.route == route || (route & 3 == 0 && lease.station == route) {
                self.cached.push(CachedLease {
                    route,
                    ptr: lease as *const LeaseRecord,
                    epoch: self.epoch,
                });
            }
        }
        self.cached.len() as u64
    }

    fn compact(&mut self, flags: u16) {
        let mut next = Vec::with_capacity(self.leases.len());
        for lease in &self.leases {
            if lease.active || (lease.amount & 1 == flags as u32 & 1) {
                next.push(lease.clone());
            }
        }
        self.leases = next;
        self.epoch = self.epoch.wrapping_add(1);
    }

    fn pressure_hint(&self, op: &LedgerOp, topology: Option<&TopologyMap>) -> u64 {
        let route_width = topology
            .map(|topology| topology.route_window(op.route))
            .unwrap_or(1);
        (route_width as u64).wrapping_mul(op.amount as u64 + 1)
    }

    fn audit_cached(&self, topology: Option<&TopologyMap>) -> u64 {
        let mut score = 0x243f_6a88_85a3_08d3u64;
        for cached in &self.cached {
            if cached.ptr.is_null() {
                continue;
            }
            let lease = unsafe { &*cached.ptr };
            let route_window = topology
                .map(|map| map.route_window(cached.route))
                .unwrap_or(1) as u64;
            score = score.rotate_left(9)
                ^ lease.amount as u64
                ^ lease.station as u64
                ^ lease.route as u64
                ^ lease.epoch as u64
                ^ cached.epoch as u64
                ^ route_window;
        }
        score
    }
}

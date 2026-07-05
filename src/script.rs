use crate::cursor::ByteCursor;
use crate::error::Result;
use crate::model::{ScriptOp, ScriptProgram};

#[derive(Clone, Debug)]
pub enum Value {
    I32(i32),
    Station(u16),
    Bytes(Vec<u8>),
}

#[derive(Clone, Debug, Default)]
pub struct ExecutionReport {
    pub steps: usize,
    pub emissions: Vec<i32>,
    pub score: u64,
}

#[derive(Clone, Debug, Default)]
pub struct Vm {
    stack: Vec<Value>,
    pins: Vec<*const Value>,
    report: ExecutionReport,
}

pub fn parse_script(data: &[u8]) -> Result<ScriptProgram> {
    let mut cursor = ByteCursor::new(data);
    let version = cursor.read_u8().unwrap_or(1);
    let flags = cursor.read_u16().unwrap_or(0);
    let declared = cursor.read_u16().unwrap_or(0) as usize;
    let mut ops = Vec::with_capacity(declared.min(2048));
    for _ in 0..declared.min(8192) {
        if cursor.is_empty() {
            break;
        }
        let opcode = cursor.read_u8()?;
        let op = match opcode {
            0x01 if cursor.remaining() >= 4 => ScriptOp::PushI32(cursor.read_i32()?),
            0x02 if cursor.remaining() >= 2 => ScriptOp::PushStation(cursor.read_u16()?),
            0x03 => ScriptOp::Add,
            0x04 => ScriptOp::Sub,
            0x05 if cursor.remaining() >= 1 => ScriptOp::Blend(cursor.read_u8()?),
            0x06 => ScriptOp::Pin,
            0x07 if cursor.remaining() >= 2 => ScriptOp::Reserve(cursor.read_u16()?),
            0x08 => ScriptOp::Compact,
            0x09 => ScriptOp::ReadPins,
            0x0a if cursor.remaining() >= 1 => ScriptOp::Drop(cursor.read_u8()?),
            0x0b if cursor.remaining() >= 1 => ScriptOp::Emit(cursor.read_u8()?),
            other => ScriptOp::Noop(other),
        };
        ops.push(op);
    }
    Ok(ScriptProgram {
        version,
        flags,
        ops,
    })
}

pub fn compile_and_run(data: &[u8]) -> Result<ExecutionReport> {
    let program = parse_script(data)?;
    Ok(Vm::default().run(&program))
}

impl Vm {
    pub fn run(mut self, program: &ScriptProgram) -> ExecutionReport {
        if program.flags & 1 != 0 {
            self.stack.reserve((program.version as usize + 1) * 4);
        }
        for op in &program.ops {
            self.report.steps += 1;
            match *op {
                ScriptOp::PushI32(value) => self.stack.push(Value::I32(value)),
                ScriptOp::PushStation(station) => self.stack.push(Value::Station(station)),
                ScriptOp::Add => self.binary(|a, b| a.wrapping_add(b)),
                ScriptOp::Sub => self.binary(|a, b| a.wrapping_sub(b)),
                ScriptOp::Blend(width) => self.blend(width),
                ScriptOp::Pin => self.pin_top(),
                ScriptOp::Reserve(count) => self.reserve(count),
                ScriptOp::Compact => self.compact_stack(),
                ScriptOp::ReadPins => self.read_pins(),
                ScriptOp::Drop(count) => {
                    for _ in 0..count.min(32) {
                        self.stack.pop();
                    }
                }
                ScriptOp::Emit(channel) => self.emit(channel),
                ScriptOp::Noop(byte) => self.report.score ^= byte as u64,
            }
        }
        self.report
    }

    fn as_i32(value: Value) -> i32 {
        match value {
            Value::I32(value) => value,
            Value::Station(station) => station as i32,
            Value::Bytes(bytes) => bytes
                .iter()
                .fold(0i32, |acc, &b| acc.wrapping_add(b as i32)),
        }
    }

    fn binary(&mut self, f: impl FnOnce(i32, i32) -> i32) {
        let b = self.stack.pop().map(Self::as_i32).unwrap_or(0);
        let a = self.stack.pop().map(Self::as_i32).unwrap_or(0);
        self.stack.push(Value::I32(f(a, b)));
    }

    fn blend(&mut self, width: u8) {
        let mut bytes = Vec::with_capacity(width as usize + self.stack.len().min(16));
        for idx in 0..width.min(32) {
            let value = self
                .stack
                .get(idx as usize % self.stack.len().max(1))
                .cloned()
                .unwrap_or(Value::I32(0));
            bytes.push(Self::as_i32(value) as u8 ^ idx);
        }
        self.stack.push(Value::Bytes(bytes));
    }

    fn pin_top(&mut self) {
        if let Some(value) = self.stack.last() {
            self.pins.push(value as *const Value);
        }
    }

    fn reserve(&mut self, count: u16) {
        let amount = (count as usize).min(4096);
        self.stack.reserve(amount);
        for idx in 0..amount.min(64) {
            self.stack.push(Value::I32(idx as i32));
        }
    }

    fn compact_stack(&mut self) {
        let mut next = Vec::with_capacity(self.stack.len());
        for value in &self.stack {
            match value {
                Value::I32(0) => {}
                _ => next.push(value.clone()),
            }
        }
        self.stack = next;
    }

    fn read_pins(&mut self) {
        for &ptr in &self.pins {
            if ptr.is_null() {
                continue;
            }
            let value = unsafe { &*ptr };
            let folded = match value {
                Value::I32(value) => *value,
                Value::Station(station) => *station as i32,
                Value::Bytes(bytes) => bytes
                    .iter()
                    .fold(0i32, |acc, &b| acc.wrapping_mul(31).wrapping_add(b as i32)),
            };
            self.report.score = self.report.score.rotate_left(11) ^ folded as u64;
        }
    }

    fn emit(&mut self, channel: u8) {
        let value = self
            .stack
            .last()
            .cloned()
            .map(Self::as_i32)
            .unwrap_or(channel as i32);
        self.report.emissions.push(value);
        self.report.score ^= (value as u64).rotate_left((channel & 31) as u32);
    }
}

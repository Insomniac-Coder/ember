//! The evaluator's memory: one flat heap of bytes, where every value lives with the target's
//! layout (`[CT-4]`: its pointer size, endianness and layout). An address is an offset into the
//! heap; addresses below `NULL_PAGE` are no memory, so a null or small pointer read faults
//! instead of reading something. Locals, heap blocks and string literals share the heap.
//!
//! What the program holds a pointer to but cannot read as bytes — a function, a class's type
//! information, an interface table, a callable value's descriptor — has an address above every
//! heap address, in a region of its own, which the evaluator decodes (`Region`).

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

/// A hasher for the evaluator's keys (addresses, sizes, symbols): a multiply per word, as rustc's
/// own `FxHasher`; the keys are the program's, never an adversary's.
#[derive(Default, Clone, Copy)]
pub struct WordHasher(u64);

const SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

impl Hasher for WordHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for chunk in bytes.chunks(8) {
            let mut word = [0u8; 8];
            word[..chunk.len()].copy_from_slice(chunk);
            self.write_u64(u64::from_le_bytes(word));
        }
    }

    fn write_u64(&mut self, n: u64) {
        self.0 = (self.0.rotate_left(5) ^ n).wrapping_mul(SEED);
    }

    fn write_u32(&mut self, n: u32) {
        self.write_u64(n.into());
    }

    fn write_usize(&mut self, n: usize) {
        self.write_u64(n as u64);
    }
}

pub type WordMap<K, V> = HashMap<K, V, BuildHasherDefault<WordHasher>>;

/// The first address that is memory.
pub const NULL_PAGE: u64 = 0x1_0000;

/// The regions of addresses that name something other than bytes, each `REGION_SPAN` wide.
const REGION_SPAN: u64 = 1 << 52;
const CODE: u64 = 1 << 56;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    /// A function: its body's index.
    Code,
    /// A class's type information: its class id.
    TypeInfo,
    /// An interface table: its index in the evaluator's list.
    Table,
    /// A callable value's descriptor: its index in the evaluator's list.
    Descriptor,
    /// An interface's identity.
    InterfaceId,
}

const REGIONS: [Region; 5] = [Region::Code, Region::TypeInfo, Region::Table, Region::Descriptor, Region::InterfaceId];

/// The address of entry `index` of `region`.
pub fn address(region: Region, index: usize) -> u64 {
    let at = REGIONS.iter().position(|r| *r == region).expect("a region") as u64;
    CODE + at * REGION_SPAN + (index as u64) * 16
}

/// The region and entry an address names, if it is one of theirs.
pub fn decode(address: u64) -> Option<(Region, usize)> {
    if address < CODE || (address - CODE) % 16 != 0 {
        return None;
    }
    let offset = address - CODE;
    let region = *REGIONS.get((offset / REGION_SPAN) as usize)?;
    Some((region, ((offset % REGION_SPAN) / 16) as usize))
}

/// A read or write the program's memory cannot serve: a defect of the program's `unsafe` code
/// or of the evaluator, reported rather than crashing the compiler.
#[derive(Debug)]
pub struct Fault(pub String);

pub struct Memory {
    bytes: Vec<u8>,
    /// Each live block: its start and its size.
    blocks: WordMap<u64, u64>,
    /// Freed blocks by size, for reuse.
    free: WordMap<u64, Vec<u64>>,
    /// `[CT-3]` — the most bytes the heap may hold at once.
    pub limit: u64,
    /// Bytes in live blocks.
    pub live: u64,
}

impl Memory {
    pub fn new(limit: u64) -> Memory {
        Memory { bytes: vec![0; NULL_PAGE as usize], blocks: WordMap::default(), free: WordMap::default(), limit, live: 0 }
    }

    /// A zeroed block of `size` bytes aligned to `align`.
    pub fn alloc(&mut self, size: u64, align: u64) -> Result<u64, Fault> {
        let class = size.max(1).div_ceil(16) * 16;
        let align = align.max(16);
        if align == 16
            && let Some(start) = self.free.get_mut(&class).and_then(Vec::pop)
        {
            self.bytes[start as usize..(start + class) as usize].fill(0);
            self.blocks.insert(start, class);
            self.live += class;
            return Ok(start);
        }
        let start = (self.bytes.len() as u64).div_ceil(align) * align;
        let end = start + class;
        if self.live + class > self.limit {
            return Err(Fault(format!("the evaluator's heap limit of {} bytes is reached", self.limit)));
        }
        self.bytes.resize(end as usize, 0);
        self.blocks.insert(start, class);
        self.live += class;
        Ok(start)
    }

    pub fn free(&mut self, start: u64) -> Result<(), Fault> {
        let Some(class) = self.blocks.remove(&start) else {
            return Err(Fault(format!("a free of {start:#x}, which is no block")));
        };
        self.live -= class;
        self.free.entry(class).or_default().push(start);
        Ok(())
    }

    fn range(&self, address: u64, len: u64) -> Result<std::ops::Range<usize>, Fault> {
        let end = address.checked_add(len);
        match end {
            Some(end) if address >= NULL_PAGE && end <= self.bytes.len() as u64 => Ok(address as usize..end as usize),
            _ if len == 0 => Ok(0..0),
            _ => Err(Fault(format!("a {len}-byte access at {address:#x}, outside the evaluator's memory"))),
        }
    }

    pub fn read(&self, address: u64, len: u64) -> Result<&[u8], Fault> {
        let range = self.range(address, len)?;
        Ok(&self.bytes[range])
    }

    pub fn write(&mut self, address: u64, data: &[u8]) -> Result<(), Fault> {
        let range = self.range(address, data.len() as u64)?;
        self.bytes[range].copy_from_slice(data);
        Ok(())
    }

    /// `len` bytes from `from` to `to`; the ranges may overlap (`memmove`).
    pub fn copy(&mut self, from: u64, to: u64, len: u64) -> Result<(), Fault> {
        let source = self.range(from, len)?;
        let target = self.range(to, len)?;
        self.bytes.copy_within(source, target.start);
        Ok(())
    }

    pub fn fill(&mut self, address: u64, len: u64, byte: u8) -> Result<(), Fault> {
        let range = self.range(address, len)?;
        self.bytes[range].fill(byte);
        Ok(())
    }

    /// An unsigned little-endian integer of `size` bytes (at most 16).
    pub fn read_uint(&self, address: u64, size: u64) -> Result<u128, Fault> {
        Ok(uint(self.read(address, size)?))
    }

    /// A word of `size` bytes (at most 8).
    pub fn read_word(&self, address: u64, size: u64) -> Result<u64, Fault> {
        let bytes = self.read(address, size)?;
        let mut word = [0u8; 8];
        word[..bytes.len()].copy_from_slice(bytes);
        Ok(u64::from_le_bytes(word))
    }

    pub fn write_word(&mut self, address: u64, size: u64, value: u64) -> Result<(), Fault> {
        self.write(address, &value.to_le_bytes()[..size as usize])
    }

    pub fn read_u64(&self, address: u64) -> Result<u64, Fault> {
        Ok(self.read_uint(address, 8)? as u64)
    }

    pub fn write_u64(&mut self, address: u64, value: u64) -> Result<(), Fault> {
        self.write(address, &value.to_le_bytes())
    }

    pub fn read_u32(&self, address: u64) -> Result<u32, Fault> {
        Ok(self.read_uint(address, 4)? as u32)
    }

    pub fn write_u32(&mut self, address: u64, value: u32) -> Result<(), Fault> {
        self.write(address, &value.to_le_bytes())
    }
}

/// The unsigned little-endian integer `bytes` hold (the first 16).
pub fn uint(bytes: &[u8]) -> u128 {
    let mut wide = [0u8; 16];
    let n = bytes.len().min(16);
    wide[..n].copy_from_slice(&bytes[..n]);
    u128::from_le_bytes(wide)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_are_zeroed_aligned_reused_and_bounded() {
        let mut memory = Memory::new(1 << 20);
        let a = memory.alloc(10, 8).unwrap();
        assert!(a >= NULL_PAGE && a % 16 == 0);
        memory.write_u64(a, 7).unwrap();
        memory.free(a).unwrap();
        let b = memory.alloc(12, 8).unwrap();
        assert_eq!(a, b, "a freed block of the size is reused");
        assert_eq!(memory.read_u64(b).unwrap(), 0, "and zeroed");
        let c = memory.alloc(64, 64).unwrap();
        assert_eq!(c % 64, 0);
        assert!(memory.read(0, 8).is_err(), "null is no memory");
        assert!(memory.free(c + 16).is_err());
        assert!(memory.alloc(2 << 20, 8).is_err(), "the limit holds");
        for region in REGIONS {
            assert_eq!(decode(address(region, 41)), Some((region, 41)));
        }
        assert_eq!(decode(c), None);
    }
}

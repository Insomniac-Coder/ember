// W2 "lists" in Rust: the same work as plain_lists_*.c (24 frames; 2^18 cell lists made empty,
// 2^20 pushes into random cells, every cell read, every list ended). One binary, two forms, chosen
// by the first argument:
//   vec    idiomatic safe Rust with no arena: `Vec<Vec<u32>>`, the cell array kept across frames,
//          each cell's buffer malloc'd, grown by realloc (4, 8, 16 ...: RawVec's minimum for 4-byte
//          elements is 4, as [HEAP-2]), freed at the frame's end. A list is 24 bytes.
//   avec   the rule's twin: every list an `AVec<'a, u32>` = `Vec<u32, &'a Bump>` (data, len, cap,
//          arena: 32 bytes), growing through its kept `&Bump`; the cell array an
//          `AVec<'a, AVec<'a, u32>>` made in the frame arena each frame (safe Rust's borrow rules
//          force that: `reset(&mut self)` cannot run while a list made in the arena lives, as
//          Ember's [ALC-5]). Indexing is bounds-checked in both forms, as Ember's.
#[path = "rharness.rs"]
mod harness;
#[path = "m3_bump.rs"]
mod bump;

use bump::{AVec, Bump};
use harness::Rng;

const _: () = assert!(std::mem::size_of::<Vec<u32>>() == 24);
const _: () = assert!(std::mem::size_of::<AVec<u32>>() == 32);

const G: usize = 1 << 18;
const E: u32 = 1 << 20;
const FRAMES: u32 = 24;

#[inline(always)]
fn read_cell(cs: u64, l: &[u32]) -> u64 {
    let mut s: u64 = 0;
    for (k, &v) in l.iter().enumerate() {
        s = s.wrapping_add((v as u64).wrapping_mul(k as u64 + 1));
    }
    cs.wrapping_mul(1000003).wrapping_add(s).wrapping_add(l.len() as u64)
}

fn main() {
    let form = std::env::args().nth(1).expect("form: vec | avec");
    let t0 = harness::start();
    let mut rng = Rng::new(42);
    let mut cs: u64 = 0;
    match form.as_str() {
        "vec" => {
            let mut cells: Vec<Vec<u32>> = Vec::with_capacity(G);
            for f in 0..FRAMES {
                for _ in 0..G {
                    cells.push(Vec::new());
                }
                for e in 0..E {
                    let r = rng.next();
                    cells[r as usize % G].push(e.wrapping_mul(2654435761) ^ f);
                }
                for l in &cells {
                    cs = read_cell(cs, l);
                }
                cells.clear();
            }
        }
        "avec" => {
            let mut frame = Bump::new();
            for f in 0..FRAMES {
                {
                    let mut cells: AVec<AVec<u32>> = AVec::with_capacity_in(G, &frame);
                    for _ in 0..G {
                        cells.push(AVec::new_in(&frame));
                    }
                    for e in 0..E {
                        let r = rng.next();
                        cells[r as usize % G].push(e.wrapping_mul(2654435761) ^ f);
                    }
                    for l in cells.iter() {
                        cs = read_cell(cs, l);
                    }
                }
                frame.reset();
            }
        }
        _ => panic!("form: vec | avec"),
    }
    harness::report(&format!("rust_lists_{}", form), t0, cs);
}

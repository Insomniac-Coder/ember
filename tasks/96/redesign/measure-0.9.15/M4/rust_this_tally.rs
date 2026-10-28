// rust_this_tally.rs: workload this_tally in safe Rust: `tally(&mut self)`, `self` noalias for LLVM.
// The walk borrows `self.samples` while the body changes other fields (disjoint field borrows).
// `self.hist[..]` is bounds-checked, as the model's; no overflow checks (release).
// Build: rustc -C opt-level=2 --edition 2021 rust_this_tally.rs
mod harness;

const SAMPLES: usize = 65536;
const FRAMES: usize = 3000;

pub struct Stats { pub samples: Vec<i64>, pub hist: Vec<i64>, pub total: i64, pub max: i64, pub seen: i64 }

impl Stats {
    #[inline(never)]
    fn tally(&mut self) {
        for &x in &self.samples {
            self.hist[(x & 255) as usize] += 1;
            self.total += x;
            if x > self.max { self.max = x; }
        }
        self.seen += self.samples.len() as i64;
    }
}

fn main() {
    let t0 = harness::start();
    let mut rng = harness::Rng::new(23);
    let mut s = Stats { samples: Vec::with_capacity(SAMPLES), hist: vec![0; 256], total: 0, max: 0, seen: 0 };
    for _ in 0..SAMPLES { s.samples.push((rng.next() % 1000000) as i64); }
    for _ in 0..FRAMES { s.tally(); }
    let mut sum = (s.total as u64).wrapping_add(s.max as u64).wrapping_add(s.seen as u64);
    for k in 0..256 { sum = sum.wrapping_add((s.hist[k] as u64).wrapping_mul(k as u64 + 1)); }
    harness::report("rust_this_tally", t0, sum);
}

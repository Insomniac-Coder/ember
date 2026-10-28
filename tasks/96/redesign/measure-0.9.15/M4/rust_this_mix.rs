// rust_this_mix.rs: workload this_mix in safe Rust: `process(&mut self)`, `self` noalias for LLVM.
// The idiom zips the two lists (no bounds checks; `zip` stops at the shorter list, where the model's
// checked body would stop the program: the lists are equal here, so the work is the same).
// Build: rustc -C opt-level=2 --edition 2021 rust_this_mix.rs
mod harness;

const LEN: usize = 65536;
const FRAMES: usize = 3000;

pub struct Mixer { pub inp: Vec<f64>, pub out: Vec<f64>, pub gain: f64, pub peak: f64 }

impl Mixer {
    #[inline(never)]
    fn process(&mut self) {
        for (o, &x) in self.out.iter_mut().zip(&self.inp) {
            let v = x * self.gain;
            *o = *o * 0.5 + v;
            if v > self.peak { self.peak = v; }
        }
    }
}

fn main() {
    let t0 = harness::start();
    let mut rng = harness::Rng::new(29);
    let mut m = Mixer { inp: Vec::with_capacity(LEN), out: vec![0.0; LEN], gain: 1.0, peak: 0.0 };
    for _ in 0..LEN { m.inp.push(((rng.next() % 2001) as i32 - 1000) as f64 * 0.001); }
    for f in 0..FRAMES {
        m.gain = 0.25 * (1 + f % 4) as f64;
        m.process();
    }
    let mut sum = (m.peak * 1e6) as i64 as u64;
    for &o in &m.out { sum = sum.wrapping_add((o * 1e6) as i64 as u64); }
    harness::report("rust_this_mix", t0, sum);
}

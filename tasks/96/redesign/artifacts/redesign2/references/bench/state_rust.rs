// state_rust.rs: safe Rust for state.c, written with the documented idiom
// (https://rust-unofficial.github.io/patterns/idioms/mem-replace.html): mem::replace with a placeholder.
include!("harness.rs");
const NNPC: u32 = 1_000_000;
const TICKS: u32 = 100;
enum State { Idle, Walking(Vec<i32>), Running(Vec<i32>) }
struct Npc { state: State }
impl Npc {
    #[inline]
    fn update(&mut self, i: u32, tick: u32, sum: &mut u64) {
        let old = std::mem::replace(&mut self.state, State::Idle);
        self.state = match old {
            State::Walking(mut path) => {
                let v = path.pop().unwrap();
                *sum += v as u32 as u64;
                if path.is_empty() { State::Idle }
                else if (i + tick) % 3 == 0 { State::Running(path) }
                else { State::Walking(path) }
            }
            State::Running(path) => { *sum += path.len() as u64; State::Walking(path) }
            State::Idle => { *sum += 1; State::Idle }
        };
    }
}
fn main() {
    let mut h = H::start(); h.seed(11);
    let mut npcs: Vec<Npc> = Vec::with_capacity(NNPC as usize);
    for _ in 0..NNPC {
        let n = 1 + h.rand() % 32;
        let mut p: Vec<i32> = Vec::with_capacity(n as usize);
        for _ in 0..n { p.push((h.rand() % 1000) as i32); }
        npcs.push(Npc { state: State::Walking(p) });
    }
    let mut sum = 0u64;
    for t in 0..TICKS {
        for (i, n) in npcs.iter_mut().enumerate() { n.update(i as u32, t, &mut sum); }
    }
    drop(npcs);
    h.report("state-rust", sum);
}

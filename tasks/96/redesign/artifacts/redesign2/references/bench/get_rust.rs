// get_rust.rs: safe Rust for get.c: pools as Box<dyn Any> in a table indexed by type number,
// `downcast_mut::<Pool>()` (a TypeId compare) and checked indexing. `--cfg noinline` keeps get_pos out of line.
include!("harness.rs");
use std::any::Any;
const NE: u32 = 1 << 20;
const NGET: u32 = 32_000_000;
const T_POS: usize = 3;
#[derive(Clone, Copy)]
struct Pos { x: f32, y: f32 }
struct Pool { sparse: Vec<u32>, dense: Vec<Pos> }
struct Reg { pools: Vec<Option<Box<dyn Any>>> }

#[cfg_attr(noinline, inline(never))]
#[cfg_attr(not(noinline), inline)]
fn get_pos(reg: &mut Reg, e: usize) -> &mut Pos {
    let p = reg.pools[T_POS].as_mut().expect("no Pos pool").downcast_mut::<Pool>().expect("cast failed");
    let i = p.sparse[e] as usize;
    &mut p.dense[i]
}

fn main() {
    let mut h = H::start(); h.seed(7);
    let mut sparse: Vec<u32> = (0..NE).collect();
    let mut i = NE - 1;
    while i > 0 {
        let j = h.rand() % (i + 1);
        sparse.swap(i as usize, j as usize);
        i -= 1;
    }
    let dense: Vec<Pos> = (0..NE).map(|i| Pos { x: 0.0, y: i as f32 }).collect();
    let mut reg = Reg { pools: (0..8).map(|_| None).collect() };
    reg.pools[T_POS] = Some(Box::new(Pool { sparse, dense }));
    let t0 = std::time::Instant::now();
    for _ in 0..NGET {
        let e = (h.rand() & (NE - 1)) as usize;
        get_pos(&mut reg, e).x += 1.0;
    }
    print!("work_ms={:.3} ", t0.elapsed().as_secs_f64() * 1000.0);
    let p = reg.pools[T_POS].as_ref().unwrap().downcast_ref::<Pool>().unwrap();
    let mut sum = 0u64;
    for (i, d) in p.dense.iter().enumerate() { sum = sum.wrapping_add((d.x as u64) * (i as u64 + 1)); }
    let _ = p.dense[0].y;
    h.report(if cfg!(noinline) { "get-rust-noinline" } else { "get-rust" }, sum);
}

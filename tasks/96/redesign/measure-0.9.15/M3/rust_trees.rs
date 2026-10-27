// W1 "trees" in Rust: the same work as plain_trees_*.c (16 batches of random expression trees,
// at least 2^20 nodes each, all alive at once; evaluate; end). One binary, four forms, chosen by
// the first argument:
//   box    idiomatic safe Rust with no arena: `Box<Expr>` from the global allocator (malloc/free
//          per node, a drop walk). Node 24 bytes.
//   abox   the rule's twin: each link an `ABox<'a, Expr>` = `Box<Expr, &'a Bump>` (block + arena
//          address, 16 bytes), drop walk giving each block back through the kept `&Bump` (a no-op).
//          Node 40 bytes. Safe code over the hand-written arena (m3_bump.rs).
//   bref   the arena crates' idiom (bumpalo's `alloc` / its `Box<'a, T>`): links are `&'a Expr`,
//          8 bytes, no drop at all; `reset(&mut self)` proves nothing is still used. Node 24 bytes.
//          Salvage (a)'s twin.
//   index  safe Rust's other arena: one `Vec<ExprI>` per batch, links are u32 indices,
//          bounds-checked. Node 16 bytes (a different layout: noted in M3.md).
#[path = "rharness.rs"]
mod harness;
#[path = "m3_bump.rs"]
mod bump;

use bump::{ABox, Bump};
use harness::Rng;

// The layouts M3.md states, checked when compiling.
const _: () = assert!(std::mem::size_of::<Expr>() == 24);
const _: () = assert!(std::mem::size_of::<ExprA>() == 40);
const _: () = assert!(std::mem::size_of::<ExprR>() == 24);
const _: () = assert!(std::mem::size_of::<ExprI>() == 16);
const _: () = assert!(std::mem::size_of::<ABox<ExprA>>() == 16);

const NB: u64 = 1 << 20;
const BATCHES: u32 = 16;
const DEPTH: u32 = 12;

#[inline(always)]
fn apply(op: i32, a: u32, b: u32) -> u32 {
    match op {
        1 => a.wrapping_add(b),
        2 => a.wrapping_mul(b),
        _ => a.wrapping_sub(b),
    }
}

// ---- box
enum Expr {
    Leaf(i32),
    Bin(i32, Box<Expr>, Box<Expr>),
}
fn build_box(rng: &mut Rng, d: u32, count: &mut u64) -> Box<Expr> {
    let r = rng.next();
    let e = if d == 0 || r & 3 == 0 {
        Expr::Leaf(((r >> 2) % 100) as i32)
    } else {
        let l = build_box(rng, d - 1, count);
        let rr = build_box(rng, d - 1, count);
        Expr::Bin((1 + (r >> 2) % 3) as i32, l, rr)
    };
    *count += 1;
    Box::new(e)
}
fn eval_box(e: &Expr) -> u32 {
    match e {
        Expr::Leaf(v) => *v as u32,
        Expr::Bin(op, l, r) => apply(*op, eval_box(l), eval_box(r)),
    }
}

// ---- abox
enum ExprA<'a> {
    Leaf(i32),
    Bin(i32, ABox<'a, ExprA<'a>>, ABox<'a, ExprA<'a>>),
}
fn build_abox<'a>(a: &'a Bump, rng: &mut Rng, d: u32, count: &mut u64) -> ABox<'a, ExprA<'a>> {
    let r = rng.next();
    let e = if d == 0 || r & 3 == 0 {
        ExprA::Leaf(((r >> 2) % 100) as i32)
    } else {
        let l = build_abox(a, rng, d - 1, count);
        let rr = build_abox(a, rng, d - 1, count);
        ExprA::Bin((1 + (r >> 2) % 3) as i32, l, rr)
    };
    *count += 1;
    ABox::new_in(e, a)
}
fn eval_abox(e: &ExprA) -> u32 {
    match e {
        ExprA::Leaf(v) => *v as u32,
        ExprA::Bin(op, l, r) => apply(*op, eval_abox(l), eval_abox(r)),
    }
}

// ---- bref
enum ExprR<'a> {
    Leaf(i32),
    Bin(i32, &'a ExprR<'a>, &'a ExprR<'a>),
}
fn build_bref<'a>(a: &'a Bump, rng: &mut Rng, d: u32, count: &mut u64) -> &'a ExprR<'a> {
    let r = rng.next();
    let e = if d == 0 || r & 3 == 0 {
        ExprR::Leaf(((r >> 2) % 100) as i32)
    } else {
        let l = build_bref(a, rng, d - 1, count);
        let rr = build_bref(a, rng, d - 1, count);
        ExprR::Bin((1 + (r >> 2) % 3) as i32, l, rr)
    };
    *count += 1;
    a.alloc(e)
}
fn eval_bref(e: &ExprR) -> u32 {
    match e {
        ExprR::Leaf(v) => *v as u32,
        ExprR::Bin(op, l, r) => apply(*op, eval_bref(l), eval_bref(r)),
    }
}

// ---- index
enum ExprI {
    Leaf(i32),
    Bin(i32, u32, u32),
}
fn build_index(nodes: &mut Vec<ExprI>, rng: &mut Rng, d: u32, count: &mut u64) -> u32 {
    let r = rng.next();
    let e = if d == 0 || r & 3 == 0 {
        ExprI::Leaf(((r >> 2) % 100) as i32)
    } else {
        let l = build_index(nodes, rng, d - 1, count);
        let rr = build_index(nodes, rng, d - 1, count);
        ExprI::Bin((1 + (r >> 2) % 3) as i32, l, rr)
    };
    *count += 1;
    nodes.push(e);
    (nodes.len() - 1) as u32
}
fn eval_index(nodes: &[ExprI], i: u32) -> u32 {
    match nodes[i as usize] {
        ExprI::Leaf(v) => v as u32,
        ExprI::Bin(op, l, r) => apply(op, eval_index(nodes, l), eval_index(nodes, r)),
    }
}

fn main() {
    let form = std::env::args().nth(1).expect("form: box | abox | bref | index");
    let t0 = harness::start();
    let mut rng = Rng::new(42);
    let mut cs: u64 = 0;
    match form.as_str() {
        "box" => {
            for _ in 0..BATCHES {
                let mut roots: Vec<Box<Expr>> = Vec::new();
                let mut count = 0u64;
                while count < NB {
                    roots.push(build_box(&mut rng, DEPTH, &mut count));
                }
                for e in &roots {
                    cs = cs.wrapping_mul(1000003).wrapping_add(eval_box(e) as u64);
                }
                drop(roots);
                cs = cs.wrapping_add(count);
            }
        }
        "abox" => {
            let mut arena = Bump::new();
            for _ in 0..BATCHES {
                let mut count = 0u64;
                {
                    let mut roots: Vec<ABox<ExprA>> = Vec::new();
                    while count < NB {
                        roots.push(build_abox(&arena, &mut rng, DEPTH, &mut count));
                    }
                    for e in &roots {
                        cs = cs.wrapping_mul(1000003).wrapping_add(eval_abox(e) as u64);
                    }
                }
                arena.reset();
                cs = cs.wrapping_add(count);
            }
        }
        "bref" => {
            let mut arena = Bump::new();
            for _ in 0..BATCHES {
                let mut count = 0u64;
                {
                    let mut roots: Vec<&ExprR> = Vec::new();
                    while count < NB {
                        roots.push(build_bref(&arena, &mut rng, DEPTH, &mut count));
                    }
                    for e in &roots {
                        cs = cs.wrapping_mul(1000003).wrapping_add(eval_bref(e) as u64);
                    }
                }
                arena.reset();
                cs = cs.wrapping_add(count);
            }
        }
        "index" => {
            let mut nodes: Vec<ExprI> = Vec::new();
            for _ in 0..BATCHES {
                let mut roots: Vec<u32> = Vec::new();
                let mut count = 0u64;
                while count < NB {
                    roots.push(build_index(&mut nodes, &mut rng, DEPTH, &mut count));
                }
                for &e in &roots {
                    cs = cs.wrapping_mul(1000003).wrapping_add(eval_index(&nodes, e) as u64);
                }
                nodes.clear(); // the batch's end: capacity kept, as the arena keeps its chunks
                cs = cs.wrapping_add(count);
            }
        }
        _ => panic!("form: box | abox | bref | index"),
    }
    harness::report(&format!("rust_trees_{}", form), t0, cs);
}

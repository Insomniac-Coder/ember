// rust_w2v.rs (M2, workload "w2v"): run 3's safe Rust W2 (target/redesign2/bench/rust_w2.rs: an
// arena `Vec<Node>` with u32 indices, every index bounds-checked) with this workload's one change:
// every step of the walk calls a stored callback that counts the step. The callback is a
// `Box<dyn FnMut(&mut Vec<Node>, u32)>`, chosen at run time, so it may change the tree (a pruner is
// the other choice, never given in the timed runs); the walk reads the parent after the call, through
// a bounds-checked index, which is all safe Rust needs: a slot is never freed, so no step can reach
// freed memory. Checksum W2's: 647034814.
#[path = "rharness.rs"]
mod harness;

const N: usize = 1_000_000;
const NONE: u32 = u32::MAX;

struct Node { value: i32, parent: u32, children: Vec<u32> }

fn main() {
    let t0 = harness::start();
    let mut rng = harness::Rng::new(42);
    let mut nodes: Vec<Node> = Vec::with_capacity(N);
    nodes.push(Node { value: 0, parent: NONE, children: Vec::new() });
    for i in 1..N as u32 {
        let p = rng.next() % i;
        nodes.push(Node { value: (i % 1000) as i32, parent: p, children: Vec::new() });
        nodes[p as usize].children.push(i);
    }
    let mut visits: u64 = 0;
    {
        let prune = std::env::args().count() > 1;
        let on_visit: Box<dyn FnMut(&mut Vec<Node>, u32) + '_> = if prune {
            Box::new(|ns: &mut Vec<Node>, p: u32| {
                // take p out of its parent's list; its slots stay in the arena, unreachable
                let q = ns[p as usize].parent;
                if q == NONE { return; }
                let ch = &mut ns[q as usize].children;
                let k = ch.iter().position(|&c| c == p).unwrap();
                ch.swap_remove(k);
                ns[p as usize].parent = NONE;
            })
        } else {
            Box::new(|ns: &mut Vec<Node>, p: u32| {
                if ns[p as usize].value >= 0 { visits += 1; }
            })
        };
        let mut on_visit = std::hint::black_box(on_visit);
        for pass in 0..10u32 {
            for i in 0..N {
                let mut a = nodes[i].parent;
                while a != NONE {
                    on_visit(&mut nodes, a);
                    a = nodes[a as usize].parent;
                }
            }
            for i in 1..N as u32 {
                if (i + pass) % 100 != 0 { continue; }
                let np = rng.next() % i;
                let op = nodes[i as usize].parent as usize;
                let ch = &mut nodes[op].children;
                let k = ch.iter().position(|&c| c == i).unwrap();
                ch.swap_remove(k);
                nodes[i as usize].parent = np;
                nodes[np as usize].children.push(i);
            }
        }
    }
    let mut sum = visits;
    for n in &nodes { sum = sum.wrapping_add((n.value as i64 * n.children.len() as i64) as u64); }
    harness::report("rust-w2v", t0, sum);
}

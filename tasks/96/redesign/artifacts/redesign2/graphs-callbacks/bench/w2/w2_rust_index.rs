// W2 in safe Rust, the way Rust programmers are told to write trees: all nodes in one Vec, links as
// indices (the arena the phase 1 A8 entry names). The re-parent removes from the old parent keeping
// the order (Vec::remove). Every index is bounds-checked by Rust.
#[path = "../harness.rs"]
mod harness;

const N: u32 = 1_000_000;
const NONE: u32 = u32::MAX;

struct Node {
    value: i32,
    parent: u32,
    children: Vec<u32>,
}

fn main() {
    let t0 = harness::start();
    let mut rng = harness::Rng::new(42);
    let mut nodes: Vec<Node> = Vec::with_capacity(0);
    nodes.push(Node { value: 0, parent: NONE, children: Vec::new() });
    for i in 1..N {
        let p = rng.next() % i;
        nodes.push(Node { value: (i % 1000) as i32, parent: p, children: Vec::new() });
        nodes[p as usize].children.push(i);
    }
    let mut depth: u64 = 0;
    for p in 0..10u32 {
        for j in 0..nodes.len() {
            let mut a = nodes[j].parent;
            while a != NONE {
                depth += 1;
                a = nodes[a as usize].parent;
            }
        }
        for i in 1..N {
            if (i + p) % 100 != 0 { continue; }
            let np = rng.next() % i;
            let op = nodes[i as usize].parent;
            let ch = &mut nodes[op as usize].children;
            let k = ch.iter().position(|&x| x == i).unwrap();
            ch.remove(k);
            nodes[np as usize].children.push(i);
            nodes[i as usize].parent = np;
        }
    }
    let mut sum = depth;
    for n in nodes.iter() {
        sum = sum.wrapping_add((n.value as i64 * n.children.len() as i64) as u64);
    }
    harness::report("w2-rust-index", t0, sum);
}

// rust_tree_arena.rs (M2, workload "tree"): the tree as Rust programmers write it when parent links
// are wanted without `Rc`: an arena `Vec<Node>` with u32 indices (run 3's rust_w2.rs shape). No
// counts at all; every index is bounds-checked. A round clears the arena (dropping every children
// `Vec`) and builds again into the same buffer. Same work and random sequence as plain_tree.c.
#[path = "rharness.rs"]
mod harness;

const N: usize = 1_000_000;
const ROUNDS: u32 = 4;
const NONE: u32 = u32::MAX;

struct Node { value: i32, parent: u32, children: Vec<u32> }

fn main() {
    let t0 = harness::start();
    let mut rng = harness::Rng::new(42);
    let mut sum: u64 = 0;
    let mut nodes: Vec<Node> = Vec::with_capacity(N);
    for round in 0..ROUNDS {
        nodes.clear();
        nodes.push(Node { value: 0, parent: NONE, children: Vec::new() });
        for i in 1..N as u32 {
            let p = rng.next() % i;
            nodes.push(Node { value: (i % 1000) as i32, parent: p, children: Vec::new() });
            nodes[p as usize].children.push(i);
        }
        for i in 1..N as u32 {
            if (i + round) % 4 != 0 { continue; }
            let op = nodes[i as usize].parent as usize;
            let ch = &mut nodes[op].children;
            let k = ch.iter().position(|&c| c == i).unwrap();
            ch.swap_remove(k);
            let np = rng.next() % i;
            nodes[i as usize].parent = np;
            nodes[np as usize].children.push(i);
        }
        for n in &nodes {
            sum = sum.wrapping_add((n.value as i64 * n.children.len() as i64) as u64);
            if n.parent != NONE {
                sum = sum.wrapping_add(nodes[n.parent as usize].value as i64 as u64);
            }
        }
    }
    harness::report("rust-tree-arena", t0, sum);
}

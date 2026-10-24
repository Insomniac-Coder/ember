// Safe Rust, W2 (tree with parent pointers), written the way Rust programmers write trees with
// parent links: an arena (Vec<Node>) and u32 indices, since `&`/`&mut` parents are refused and
// Rc<RefCell<Node>> with Weak parents is the slow spelling. Every index is bounds-checked.
// Same work as plain/w2.c: removal from the old parent is a search and a swap with the last child.
#[path = "harness.rs"]
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
    let mut depth: u64 = 0;
    for pass in 0..10u32 {
        for i in 0..N {
            let mut a = nodes[i].parent;
            while a != NONE { depth += 1; a = nodes[a as usize].parent; }
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
    let mut sum = depth;
    for n in &nodes { sum = sum.wrapping_add((n.value as i64 * n.children.len() as i64) as u64); }
    harness::report("rust-w2", t0, sum);
}

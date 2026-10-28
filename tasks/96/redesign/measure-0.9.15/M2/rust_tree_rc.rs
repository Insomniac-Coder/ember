// rust_tree_rc.rs (M2, workload "tree"): safe Rust's tree with parent links as the Rust book writes it
// (chapter 15.6): `Rc<Node>` owners in the children lists, `Weak<Node>` parent links in a `RefCell`,
// and an index `Vec<Weak<Node>>` by number. The same work as model_tree.c: per round build N nodes,
// move a quarter to new parents, read every node once, drop the index then the tree.
// What Rust runs for the links: `Rc::downgrade` (a weak +1 with its overflow check) for each link
// made; the old `Weak`'s drop (-1, test for 0) when a link is given another target or ends;
// `Weak::upgrade` (a test, a strong +1 with its check, and the -1 when the `Rc` drops) for every open.
// Same random sequence as the C: one rng.next() per built node, one per moved node, in order.
#[path = "rharness.rs"]
mod harness;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

const N: usize = 1_000_000;
const ROUNDS: u32 = 4;

struct Node {
    value: i32,
    parent: RefCell<Weak<Node>>,
    children: RefCell<Vec<Rc<Node>>>,
}

fn node(value: i32) -> Rc<Node> {
    Rc::new(Node { value, parent: RefCell::new(Weak::new()), children: RefCell::new(Vec::new()) })
}

fn main() {
    let t0 = harness::start();
    let mut rng = harness::Rng::new(42);
    let mut sum: u64 = 0;
    for round in 0..ROUNDS {
        let root = node(0);
        let mut nodes: Vec<Weak<Node>> = Vec::with_capacity(N);
        nodes.push(Rc::downgrade(&root));
        for i in 1..N as u32 {
            let n = node((i % 1000) as i32);
            nodes.push(Rc::downgrade(&n));
            let r = (rng.next() % i) as usize;
            if let Some(p) = nodes[r].upgrade() {
                *n.parent.borrow_mut() = Rc::downgrade(&p);
                p.children.borrow_mut().push(n);
            }
        }
        for i in 1..N as u32 {
            if (i + round) % 4 != 0 { continue; }
            let Some(node) = nodes[i as usize].upgrade() else { continue };
            let old = node.parent.borrow().upgrade();
            let Some(old) = old else { continue };
            let moved = {
                let mut ch = old.children.borrow_mut();
                let k = ch.iter().position(|c| Rc::ptr_eq(c, &node)).unwrap();
                ch.swap_remove(k)
            };
            let r = (rng.next() % i) as usize;
            if let Some(np) = nodes[r].upgrade() {
                *moved.parent.borrow_mut() = Rc::downgrade(&np);
                np.children.borrow_mut().push(moved);
            }
        }
        for w in &nodes {
            if let Some(n) = w.upgrade() {
                sum = sum.wrapping_add((n.value as i64 * n.children.borrow().len() as i64) as u64);
                if let Some(p) = n.parent.borrow().upgrade() {
                    sum = sum.wrapping_add(p.value as i64 as u64);
                }
            }
        }
        drop(nodes); // the index's links end first, as the model's locals end
        drop(root);  // then the tree
    }
    harness::report("rust-tree-rc", t0, sum);
}

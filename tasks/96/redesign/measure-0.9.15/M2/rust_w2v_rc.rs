// rust_w2v_rc.rs (M2, workload "w2v"): the same walk in safe Rust with `Rc` owners and `Weak` parent
// links (the Rust book's tree, as rust_tree_rc.rs), which is the mechanism [WK-18] mirrors: the walk
// holds a `Weak` (`up`), and each step upgrades it (a test of the strong count, +1, and -1 when the
// `Rc` drops), upgrades the parent link (a `RefCell` borrow, the same again), and moves `up` to the
// parent (`Rc::downgrade`: a weak +1 with its check; the old `Weak`'s drop: -1 and a test for 0).
// The callback is a `Box<dyn Fn(&Rc<Node>)>` chosen at run time (a pruner is the other choice, never
// given in the timed runs). Same build, passes and re-parenting as W2; checksum 647034814.
#[path = "rharness.rs"]
mod harness;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

const N: usize = 1_000_000;

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
    let visits = Cell::new(0u64);
    {
        let prune = std::env::args().count() > 1;
        let on_visit: Box<dyn Fn(&Rc<Node>) + '_> = if prune {
            Box::new(|p: &Rc<Node>| {
                let q = p.parent.borrow().upgrade();
                if let Some(q) = q {
                    let mut ch = q.children.borrow_mut();
                    let k = ch.iter().position(|c| Rc::ptr_eq(c, p)).unwrap();
                    ch.swap_remove(k);
                }
            })
        } else {
            Box::new(|p: &Rc<Node>| {
                if p.value >= 0 { visits.set(visits.get() + 1); }
            })
        };
        let on_visit = std::hint::black_box(on_visit);
        for pass in 0..10u32 {
            for leaf in &nodes {
                let mut up = leaf.clone();
                while let Some(n) = up.upgrade() {
                    let p = match n.parent.borrow().upgrade() { Some(p) => p, None => break };
                    up = Rc::downgrade(&p);
                    on_visit(&p);
                }
            }
            for i in 1..N as u32 {
                if (i + pass) % 100 != 0 { continue; }
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
        }
    }
    let mut sum = visits.get();
    for w in &nodes {
        if let Some(n) = w.upgrade() {
            sum = sum.wrapping_add((n.value as i64 * n.children.borrow().len() as i64) as u64);
        }
    }
    harness::report("rust-w2v-rc", t0, sum);
    std::mem::forget(root); // as the C programs, nothing is dropped after the report
}

// W2 in safe Rust, the shared-object way: Rc nodes, a Weak parent, RefCell around what changes.
// This is the workaround rustc's E0502/E0505/E0515 lead to (phase1-borrowing.md A8,
// a08_tree_parent_fix.rs). The re-parent removes from the old parent keeping the order (Vec::remove).
#[path = "../harness.rs"]
mod harness;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

const N: u32 = 1_000_000;

struct Node {
    value: i32,
    parent: RefCell<Weak<Node>>,
    children: RefCell<Vec<Rc<Node>>>,
}

fn add_child(p: &Rc<Node>, c: &Rc<Node>) {
    p.children.borrow_mut().push(Rc::clone(c));
    *c.parent.borrow_mut() = Rc::downgrade(p);
}

fn main() {
    let t0 = harness::start();
    let mut rng = harness::Rng::new(42);
    let mut nodes: Vec<Rc<Node>> = Vec::new();
    nodes.push(Rc::new(Node { value: 0, parent: RefCell::new(Weak::new()), children: RefCell::new(Vec::new()) }));
    for i in 1..N {
        let n = Rc::new(Node { value: (i % 1000) as i32, parent: RefCell::new(Weak::new()), children: RefCell::new(Vec::new()) });
        nodes.push(Rc::clone(&n));
        let p = &nodes[(rng.next() % i) as usize];
        add_child(p, &n);
    }
    let mut depth: u64 = 0;
    for p in 0..10u32 {
        for n in nodes.iter() {
            let mut a = n.parent.borrow().upgrade();
            while let Some(x) = a {
                depth += 1;
                a = x.parent.borrow().upgrade();
            }
        }
        for i in 1..N {
            if (i + p) % 100 != 0 { continue; }
            let np = Rc::clone(&nodes[(rng.next() % i) as usize]);
            let c = &nodes[i as usize];
            let old = c.parent.borrow().upgrade().unwrap();
            {
                let mut ch = old.children.borrow_mut();
                let k = ch.iter().position(|x| Rc::ptr_eq(x, c)).unwrap();
                ch.remove(k);
            }
            add_child(&np, c);
        }
    }
    let mut sum = depth;
    for n in nodes.iter() {
        sum = sum.wrapping_add((n.value as i64 * n.children.borrow().len() as i64) as u64);
    }
    harness::report("w2-rust-rc", t0, sum);
}

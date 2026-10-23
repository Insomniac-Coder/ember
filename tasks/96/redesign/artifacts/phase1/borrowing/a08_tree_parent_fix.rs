// A8 workaround 1: Rc<RefCell<..>> with Weak back-pointers.
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub struct Node {
    pub name: String,
    pub parent: RefCell<Weak<Node>>,
    pub children: RefCell<Vec<Rc<Node>>>,
}
pub fn add_child(parent: &Rc<Node>, name: &str) -> Rc<Node> {
    let child = Rc::new(Node {
        name: name.to_string(),
        parent: RefCell::new(Rc::downgrade(parent)),
        children: RefCell::new(vec![]),
    });
    parent.children.borrow_mut().push(Rc::clone(&child));
    child
}
pub fn path(n: &Rc<Node>) -> String {
    match n.parent.borrow().upgrade() {
        Some(p) => format!("{}/{}", path(&p), n.name),
        None => n.name.clone(),
    }
}
// A8 workaround 2: an arena of nodes linked by index.
pub struct Tree { pub names: Vec<String>, pub parent: Vec<Option<usize>>, pub children: Vec<Vec<usize>> }
impl Tree {
    pub fn add(&mut self, parent: Option<usize>, name: &str) -> usize {
        let id = self.names.len();
        self.names.push(name.to_string());
        self.parent.push(parent);
        self.children.push(vec![]);
        if let Some(p) = parent { self.children[p].push(id); }
        id
    }
}

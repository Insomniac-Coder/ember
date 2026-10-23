// A17: applying rustc's `clone` note from a08_tree_parent.rs does not help either.
#[derive(Clone)]
pub struct Node<'a> {
    pub name: String,
    pub parent: Option<&'a Node<'a>>,
    pub children: Vec<Node<'a>>,
}

pub fn build() -> Node<'static> {
    let mut root = Node { name: "root".into(), parent: None, children: vec![] };
    let leaf = Node { name: "leaf".into(), parent: Some(&root.clone()), children: vec![] };
    root.children.push(leaf);
    root
}

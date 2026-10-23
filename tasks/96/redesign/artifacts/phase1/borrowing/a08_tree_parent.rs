// A8: a scene tree whose nodes point back at their parent.
pub struct Node<'a> {
    pub name: String,
    pub parent: Option<&'a Node<'a>>,
    pub children: Vec<Node<'a>>,
}

pub fn build() -> Node<'static> {
    let mut root = Node { name: "root".into(), parent: None, children: vec![] };
    let leaf = Node { name: "leaf".into(), parent: Some(&root), children: vec![] };
    root.children.push(leaf);
    root
}

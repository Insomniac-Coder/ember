// A11: assign a field of a moved-from struct (Ember accepts this; what does rustc say?)
pub struct Node { pub value: i64, pub children: Vec<Node> }
pub fn store_then_use() -> i64 {
    let mut root = Node { value: 0, children: vec![] };
    let mut child = Node { value: 1, children: vec![] };
    root.children.push(child);
    child.value = 5;
    root.children[0].value
}

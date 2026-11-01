# A binary search tree of words: insert with counts, look words up, walk the tree in order and measure its height.

class TreeNode:
    key: String
    count: int = 1
    left: Option[TreeNode] = None
    right: Option[TreeNode] = None

    fn init(owns key: String):
        this.key = key

fn insert_into(node: TreeNode, key: StrView) -> bool:
    if key == node.key:
        node.count += 1
        return false
    if key < node.key:
        if node.left exists as left:
            return insert_into(left, key)
        node.left = Some(TreeNode(key.to_string()))
        return true
    if node.right exists as right:
        return insert_into(right, key)
    node.right = Some(TreeNode(key.to_string()))
    return true

fn in_order(node: Option[TreeNode], ref mut out: Array[String]):
    if node exists as n:
        in_order(n.left, ref mut out)
        out.push(owns n.key.clone())
        in_order(n.right, ref mut out)

fn height(node: Option[TreeNode]) -> int:
    return match node:
        Some(n) => 1 + max(height(n.left), height(n.right))
        None => 0

class Tree:
    root: Option[TreeNode] = None
    size: int = 0

    fn insert(key: StrView):
        match this.root:
            Some(r):
                if insert_into(r, key):
                    this.size += 1
            None:
                this.root = Some(TreeNode(key.to_string()))
                this.size = 1

    fn find(key: StrView) -> Option[TreeNode]:
        current = this.root
        while current exists as n:
            if key == n.key:
                return Some(n)
            current = n.left if key < n.key else n.right
        return None

    fn words() -> Array[String]:
        out: Array[String] = []
        in_order(this.root, ref mut out)
        return out

fn main():
    text = "the quick brown fox jumps over the lazy dog while the cat watches the fox"
    tree = Tree()
    for word in text.split(" "):
        tree.insert(word)
    println(f"{tree.size} distinct words, height {height(tree.root)}")
    println(f"in order: {tree.words().join(", ")}")

    for w in ["the", "fox", "cow"]:
        match tree.find(w):
            Some(n) => println(f"'{w}' appears {n.count} times")
            None => println(f"'{w}' is not in the tree")

    sorted_tree = Tree()
    for w in tree.words():
        sorted_tree.insert(w)
    println(f"inserting in sorted order gives height {height(sorted_tree.root)} for {sorted_tree.size} words")

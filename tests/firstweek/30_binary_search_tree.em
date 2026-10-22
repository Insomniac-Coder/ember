# A binary search tree of words: insert with counts, look words up, walk the tree in order and measure its height.

class TreeNode:
    key: String
    count: int = 1
    left: Option[TreeNode] = None
    right: Option[TreeNode] = None

    fn init(self, owned key: String):
        self.key = key

fn insert_into(node: TreeNode, key: str) -> bool:
    if key == node.key:
        node.count += 1
        return False
    if key < node.key:
        if Some(left) = node.left:
            return insert_into(left, key)
        node.left = Some(TreeNode(key))
        return True
    if Some(right) = node.right:
        return insert_into(right, key)
    node.right = Some(TreeNode(key))
    return True

fn in_order(node: Option[TreeNode], mut out: Array[String]):
    if Some(n) = node:
        in_order(n.left, out)
        out.push(n.key)
        in_order(n.right, out)

fn height(node: Option[TreeNode]) -> int:
    return match node:
        Some(n) => 1 + max(height(n.left), height(n.right))
        None => 0

class Tree:
    root: Option[TreeNode] = None
    size: int = 0

    fn insert(self, key: str):
        match self.root:
            Some(r):
                if insert_into(r, key):
                    self.size += 1
            None:
                self.root = Some(TreeNode(key))
                self.size = 1

    fn find(self, key: str) -> Option[TreeNode]:
        current = self.root
        while Some(n) = current:
            if key == n.key:
                return Some(n)
            current = n.left if key < n.key else n.right
        return None

    fn words(self) -> Array[String]:
        out: Array[String] = []
        in_order(self.root, out)
        return out

fn main():
    text = "the quick brown fox jumps over the lazy dog while the cat watches the fox"
    tree = Tree()
    for word in text.split(" "):
        tree.insert(word)
    println(f"{tree.size} distinct words, height {height(tree.root)}")
    println(f"in order: {', '.join(tree.words())}")

    for w in ["the", "fox", "cow"]:
        match tree.find(w):
            Some(n) => println(f"'{w}' appears {n.count} times")
            None => println(f"'{w}' is not in the tree")

    sorted_tree = Tree()
    for w in tree.words():
        sorted_tree.insert(w)
    println(f"inserting in sorted order gives height {height(sorted_tree.root)} for {sorted_tree.size} words")

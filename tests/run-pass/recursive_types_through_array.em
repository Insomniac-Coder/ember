#$ test: run-pass
#$ rules: TYP-1, OWN-2, DRP-2, CLS-1
#$ profiles: debug, release, shipping
#$ stdout: 1
#$ 3
#$ 2
#$ assert-c: contains("em_drop_glue_0(&")
# D-182 — a type that owns itself through an `Array` (a struct tree, and the
# Part VIII class with a list of children) compiles and drops every level. The
# compiler used to overflow its own stack walking the fields, and then again
# expanding the element drop inline.

struct Tree:
    id: i64
    kids: Array[Tree]

class Node:
    id: i64
    children: Array[Node]

    fn add(mut self, child: Node):
        self.children.push(child)

fn main():
    tree = Tree(1, Array())
    tree.kids.push(Tree(2, Array()))
    tree.kids[0].kids.push(Tree(3, Array()))
    println(tree.kids.len())
    println(tree.kids[0].kids[0].id)
    root = Node(1, Array())
    root.add(Node(2, Array()))
    root.add(Node(3, Array()))
    println(root.children.len())

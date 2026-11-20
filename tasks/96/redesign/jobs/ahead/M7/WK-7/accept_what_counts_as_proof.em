# Clause WK-7.1: "Proved: a loop made and opened again in code the compiler can follow (the same
# function, the functions it calls), or opened by its owning object (a `clear()` it always calls
# before its last owner ends)"; WK-7.2's two proofs that a store closes none: "(1) the stored value
# cannot reach the holder (...) (`root.children.push(shares mut child)` with `child = shares mut
# Node(...)` made above it and given no owner: a tree built from the top down closes nothing)"; "(2)
# the holder was made in this function and no owner of it has been stored anywhere yet"
#$ test: run-pass
#$ rules: WK-7, WK-6, WK-5
#$ stdout: opened by a callee
#$ stdout: ring cleared
#$ stdout: 2
#$ stdout: 1
# One function per proof: a loop opened by a function the maker calls; a loop opened by its owning
# object's `clear`, which its `drop` always calls; a tree built from the top down; a fresh holder no
# owner of which is stored anywhere.

class Node:
    name: String
    next: Option[shares mut Node] = None
    children: Array[shares mut Node] = []

    fn init(owns name: String):
        this.name = name

class Ring:
    nodes: Array[shares mut Node] = []

    fn clear():
        for ref mut n in this.nodes:
            n.next = None

    fn drop():
        this.clear()
        println("ring cleared")

fn unlink(ref mut n: shares mut Node):
    n.next = None

fn by_a_callee():
    a = shares mut Node(String("a"))
    b = shares mut Node(String("b"))
    a.next = Some(shares mut b)
    b.next = Some(shares mut a)
    unlink(ref mut b)
    println("opened by a callee")

fn by_its_owner():
    r = Ring()
    a = shares mut Node(String("a"))
    b = shares mut Node(String("b"))
    a.next = Some(shares mut b)
    b.next = Some(shares mut a)
    r.nodes.push(shares mut a)
    r.nodes.push(shares mut b)

fn top_down() -> int:
    root = shares mut Node(String("root"))
    for name in [String("x"), String("y")]:
        child = shares mut Node(copy of name)
        root.children.push(shares mut child)
    return root.children.len()

fn fresh_holder() -> int:
    old = shares mut Node(String("old"))
    holder = shares mut Node(String("new"))
    holder.next = Some(shares mut old)
    return 1

fn main():
    by_a_callee()
    by_its_owner()
    println(top_down())
    println(fresh_holder())

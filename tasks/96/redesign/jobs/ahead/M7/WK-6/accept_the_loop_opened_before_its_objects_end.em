# Clause WK-6.2: "The same function with `b.next = None` before `a` and `b` end is accepted. Nothing
# runs for it while the program runs."; the message's other fix, `next: weakref Node = None`
#$ test: run-pass
#$ rules: WK-6, WK-1, WK-11
#$ stdout: b
#$ stdout: a ends
#$ stdout: b ends
#$ stdout: c
#$ stdout: c ends
#$ stdout: d ends
# Both of the message's ways out; each node's `drop` runs, so nothing is kept for ever. The locals end
# in reverse order, and a node's `drop` runs before its fields end (`[CLS-6]`).

class Node:
    name: String
    next: Option[shares mut Node] = None

    fn init(owns name: String):
        this.name = name

    fn drop():
        println(f"{this.name} ends")

class Link:
    name: String
    next: weakref Link = None
    forward: Option[shares mut Link] = None

    fn init(owns name: String):
        this.name = name

    fn drop():
        println(f"{this.name} ends")

fn make_pair():
    a = shares mut Node(String("a"))
    b = shares mut Node(String("b"))
    a.next = Some(shares mut b)
    b.next = Some(shares mut a)
    if a.next exists as n:
        println(n.name)
    b.next = None

fn make_linked():
    c = shares mut Link(String("c"))
    d = shares mut Link(String("d"))
    c.forward = Some(shares mut d)
    d.next = weakref c
    if d.next exists as back:
        println(back.name)

fn main():
    make_pair()
    make_linked()

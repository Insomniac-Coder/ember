#$ test: run-pass
#$ rules: WK-11, WK-12, THR-1, OWN-13
#$ profiles: debug, development, release
#$ stdout: 1
#$ 3
#$ alive
#$ 4
#$ gone
# `[WK-11]` — `weakref thread_shared T` is a link to a `thread_shared T`: a type of its own, in a field, a
# local, a parameter and an element. `weakref x` of a `thread_shared` value is one with no second word,
# any thread may hold it, and the name it opens reads as a `thread_shared T`.

import std.thread
from std.sync import Atomic

class Node:
    hits: Atomic[int] = Atomic(0)
    parent: weakref thread_shared Node = None

fn touch(w: weakref thread_shared Node):
    if w exists as node:
        node.hits.fetch_add(1)

fn visit(w: weakref thread_shared Node):
    touch(w)
    if w exists as _node:
        println("alive")

fn seen(n: thread_shared Node) -> int:
    return n.hits.load()

fn main():
    root = thread_shared Node()
    child = thread_shared Node(parent=weakref root)
    touch(child.parent)
    println(seen(root))
    links: Array[weakref thread_shared Node] = [weakref root, weakref root]
    for link in links:
        touch(link)
    println(seen(root))
    far = weakref root
    worker = thread.spawn(owns owned fn() => visit(far))
    worker.join()
    println(seen(root))
    up = weakref root
    links.clear()
    mem.drop(root)
    if up exists as _node:
        println("alive")
    else:
        println("gone")

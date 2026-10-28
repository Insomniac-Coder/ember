#$ test: run-pass
#$ rules: WK-11, GRM-44
#$ stdout: 7
#$ stdout: gone
## design rules: R11, R13
## R11: "Every `weakref` link holds one weak count on its target, with no exception: making one
## (`weakref x`, or `weakref this.keep` for another link to the same target) adds one, and its end, or
## giving it another target, subtracts one". Verdict 11 (run 3's hole G-01): a program that keeps a
## second link, drops the first and then tests the second runs correctly ("gone"). Checked under
## AddressSanitizer: no report (the second link's count keeps the node's block for the test).

class Node:
    v: int = 7

class Holder:
    keep: weakref Node = None

fn main():
    first = Node()
    h = Holder()
    h.keep = weakref first              # one weak count on the node
    a = weakref h.keep                  # another link to the same node: a second weak count
    if a exists as m:
        println(m.v)                    # 7
    h.keep = None                       # the first link ends: one subtract
    first = Node(v=8)                   # the node both links named ends; `a`'s count keeps its block
    if a exists as n:
        println(n.v)
    else:
        println("gone")

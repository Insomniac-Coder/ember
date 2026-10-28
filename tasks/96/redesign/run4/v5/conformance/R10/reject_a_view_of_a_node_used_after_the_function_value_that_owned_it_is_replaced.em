#$ test: compile-fail
#$ rules: BRW-17, LT-3
#$ help: use `p` before line 20: move line 21 above it
## design rules: R10, R12, R5
## R10 point 1, R12 (safety validation (b), hole 3, part 2): replacing a function value ends what its
## state owns, as ending an object runs its `drop` (R5): `h.keep = fn(): pass` ends the function stored
## on line 18 and the node its `owns n` held, so `p`, which reads that node through the link `w`, is not
## used after that line (E3060). Rules section 3's E3060 message for `h.keep`, in this program's lines.

class Node:
    v: int = 1

class Holder:
    keep: fn()

fn main():
    n = Node()
    w = weakref n
    h = Holder(keep=fn() owns n: println(n.v))
    if w exists as p:
        h.keep = fn(): pass
        println(p.v)    #$ error[E3060]: `h.keep = fn(): pass` (line 20) ends the function stored there on line 18 and the node it owns (`owns n`), and `p` reads that node (`if w exists as p:`, line 19) and is used on line 21; use `p` before line 20: move line 21 above it

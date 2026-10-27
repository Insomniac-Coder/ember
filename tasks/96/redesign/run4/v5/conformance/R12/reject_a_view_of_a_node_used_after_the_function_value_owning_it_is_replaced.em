#$ test: compile-fail
#$ rules: BRW-17, LT-3
#$ help: use `p` before line 22
## design rules: R12, R10, R5
## Step 3's hole 3, part 2 (`safety/recheck.md` section 6.2), R12 (rules.md lines 687-691): "Replacing or
## ending a function value ends what its state owns, as ending an object runs its `drop` (R5):
## `h.keep = fn(): pass` ends the function stored there before and the node its `owns n` held, so a view
## of that node is not used after that line (R10, E3060)"; R10 point 1 (lines 564-570). rules.md section
## 3's line 1069 fifth message, in this program's lines. The nearest allowed program is
## accept_a_view_of_a_node_used_before_the_function_value_owning_it_is_replaced.

class Node:
    name: String

class Holder:
    keep: fn()

fn main():
    n = Node(name=String("a"))
    w = weakref n
    h = Holder(keep=fn() owns n: println(n.name))
    if w exists as p:
        h.keep = fn(): pass
        println(p.name)    #$ error[E3060]: `h.keep = fn(): pass` (line 22) ends the function stored there on line 20 and the node it owns (`owns n`), and `p` reads that node (`if w exists as p:`, line 21) and is used on line 23; use `p` before line 22

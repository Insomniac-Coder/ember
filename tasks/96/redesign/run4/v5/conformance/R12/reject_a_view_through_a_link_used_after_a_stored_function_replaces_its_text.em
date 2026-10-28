#$ test: compile-fail
#$ rules: BRW-17
#$ help: use `s` before line 20
## design rules: R12, R8, R10
## Step 3's hole 3, part 1 (`safety/recheck.md` section 6.1), R12 (rules.md lines 681-687): "A change to
## its own state, and the end of what its state owns, are worked out from its body like every change:
## ... `rename = fn() owns n: n.name = String("b")` replaces the node's text, so a view into that text
## reached through a link is not used after `rename()` (R8 (b), E3060)". rules.md section 3's line 1069
## fourth message, in this program's lines. Nothing runs for it while the program runs. The nearest
## allowed program is accept_a_view_through_a_link_used_before_a_stored_function_replaces_its_text.

class Node:
    name: String

fn main():
    n = Node(name=String(" a "))
    w = weakref n
    rename = fn() owns n: n.name = String("b")
    if w exists as p:
        s = p.name.trim()
        rename()
        println(s)    #$ error[E3060]: `rename()` (line 20) replaces the node's name (`n.name = String("b")`, line 17), and `s` views that name's text (line 19) and is used on line 21; use `s` before line 20, or keep your own text: `s = String(p.name.trim())` in place of line 19

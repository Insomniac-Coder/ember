#$ test: compile-fail
#$ rules: R8, R10, R12
#$ help: s = String(p.name.trim())
## R8 (b), R12 (safety validation (b), hole 3, part 1): what a call through a function value changes is
## worked out from the function stored there (verdict 86): `rename` replaces the name of the node its
## state owns, which frees the text `s` views through the link `w`, so `s` is not used after `rename()`
## (E3060). Rules section 3's E3060 message for `rename()`, in this program's lines.

class Node:
    name: String

fn main():
    n = Node(name=String(" a "))
    w = weakref n
    rename = fn() owns n: n.name = String("b")
    if w exists as p:
        s = p.name.trim()
        rename()
        println(s)    #$ error[E3060]: `rename()` (line 18) replaces the node's name (`n.name = String("b")`, line 15), and `s` views that name's text (line 17) and is used on line 19; use `s` before line 18, or keep your own text: write `s = String(p.name.trim())` in place of line 17

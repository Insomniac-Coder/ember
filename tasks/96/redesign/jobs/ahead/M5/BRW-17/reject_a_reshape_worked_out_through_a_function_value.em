# Clause BRW-17.3: "What may reshape is worked out from the code (`[EFF-1]`): the function itself, the
# functions it calls, every function the program can put in a function value or behind a `dyn`
# method it calls, and, at a foreign call, the callbacks C keeps (`[FFI-21]`). So a call through a
# function value whose stored function replaces the text of an object it owns (`rename = fn() owns
# n: n.name = String("b")`) ends a view of that text reached through a link (`s = p.name.trim()`
# under `if w exists as p:`): `s` is not used after `rename()` ("`rename()` (line 9) replaces the
# node's name (`n.name = String("b")`, line 6), and `s` views that name's text (line 8) and is used
# on line 10; use `s` before line 9, or keep your own text: `s = String(p.name.trim())`")."
#$ test: compile-fail
#$ rules: BRW-17, EFF-1, CLO-2, WK-11, GRM-44
# The spec's program and message, with this file's line numbers (the stored function is the one line
# `rename = fn() owns n: n.name = String("b")`, which the message names by its line).

class Node:
    name: String

fn main():
    n = Node(name=String("a"))
    w = weakref n
    rename = fn() owns n: n.name = String("b")
    if w exists as p:
        s = p.name.trim()
        rename()
        println(s)    #$ error[E3060]: `rename()` (line 23) replaces the node's name (`n.name = String("b")`, line 20), and `s` views that name's text (line 22) and is used on line 24; use `s` before line 23, or keep your own text: `s = String(p.name.trim())`

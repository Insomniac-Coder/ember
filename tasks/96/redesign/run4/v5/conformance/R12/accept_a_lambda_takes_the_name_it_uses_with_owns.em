#$ test: run-pass
#$ rules: R12, R3
#$ stdout: ann
## R12 (rules.md lines 681-683, 766-767), R3 (lines 230-236): `owns name` after the brackets makes `name`
## the function value's own state, and says so on the name (verdict 83: no `with`). The nearest allowed
## program to reject_owned_fn_names_its_captures.

fn main():
    name = String("ann")
    f = fn() owns name => println(name)
    f()

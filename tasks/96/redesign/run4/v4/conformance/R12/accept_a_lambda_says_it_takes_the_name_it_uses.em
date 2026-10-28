#$ test: run-pass
#$ rules: R12, R3
#$ stdout: ann
## R12 (rules.md lines 756-757): `with owns name` makes `name` the function value's own state, and says
## so on the name. The nearest allowed program to reject_owned_fn_names_its_captures.

fn main():
    name = String("ann")
    f = fn() with owns name => println(name)
    f()

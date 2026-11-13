#$ test: compile-fail
#$ rules: STD-9, TYP-25
#$ error[E2021]: `println(1, flush=true)` names `flush`, and `println` has no parameter of that name
# `print` and `println` take `sep` and `end`, and nothing else by name. They are library functions
# over a pack (`[STD-9]`, verdict 117), so a name no parameter has is `E2021` as at any call (N171).

fn main():
    println(1, flush=true)

#$ test: compile-fail
#$ rules: STD-9
#$ error[E2020]: `println` has no parameter `flush`
# `print` and `println` take `sep` and `end`, and nothing else by name.

fn main():
    println(1, flush=true)

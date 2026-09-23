#$ test: compile-fail
#$ rules: GRM-25, GRM-23
#$ error[E0102]: `is` and `in` may not appear in a comparison chain
# Only `==`, `!=`, `<`, `>`, `<=` and `>=` chain.

fn main():
    x = 1
    println(x < 2 in [1, 2])

#$ test: compile-fail
#$ rules: TYP-25
#$ profiles: debug, development, release
#$ error[E2021]: `pair(missing=1, right=2)` names `missing`, and `pair` has no parameter of that name: `fn pair(left: i32, right: i32)`
#$ error[E2021]: `pair(left=1, left=2)` gives `left` twice: `fn pair(left: i32, right: i32)`
#$ error[E2021]: `pair(right=2, 1)` gives `1` after the named `right=2`, and positional arguments come first; write `pair(right=2, left=1)`

fn pair(left: i32, right: i32):
    return

fn main():
    pair(missing=1, right=2)
    pair(left=1, left=2)
    pair(right=2, 1)

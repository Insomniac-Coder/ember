#$ test: compile-fail
#$ rules: SPN-3, BRW-1, BRW-2
#$ error[E3022]: `parent` is already mutably borrowed

# `[SPN-3]`, `[BRW-1]` — lending a `ref mut` span with `ref mut` keeps the parent, and reserves its access
# until the child's last use: a second lending while the child is used is `E3022`.

fn main():
    values: Array[i32] = Array[i32]()
    values.push(owns 1)
    parent = ref mut values.as_mut_span()
    child = ref mut parent
    _other = ref mut parent
    println(child[0])

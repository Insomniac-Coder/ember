#$ test: compile-fail
#$ rules: OWN-1, OWN-3
# The other half of "exactly one": after the move, the old name is not a second
# owner, it is not an owner at all.

fn main():
    a: Array[i32] = Array[i32]()
    a.push(owns 1)
    _b = a
    println(a[0])          #$ error[E3040]: `a` was handed over to `_b` on line 9 and is used on line 10; to keep both, write `_b = copy of a` (a copy of your own) or `_b = shares a` (two owners of one value)

#$ test: compile-fail
#$ rules: OWN-1, OWN-3
# The other half of "exactly one": after the move, the old name is not a second
# owner, it is not an owner at all.

fn main():
    a: Array[i32] = Array[i32]()
    a.push(1)
    b = a
    println(a[0])          #$ error[E3040]: `a` has been moved out of

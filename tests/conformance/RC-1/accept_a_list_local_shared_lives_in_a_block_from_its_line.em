#$ test: run-pass
#$ rules: RC-1, RC-7, OWN-13
#$ profiles: debug, development, release
#$ stdout: 4
#$ 5
#$ 5
# `[RC-1]` — a local of this function that owns a value that is not a class object, shared by a later
# line, lives in a block of its own from the line that declares it, as if that line had said
# `shares`: nothing moves at the `shares` line, both names own one list, and the name it was first
# shared from keeps what it could do (`[RC-7]`).

fn main():
    items = [1, 2, 3]
    other = shares mut items
    other.push(owns 4)
    println(items.len())
    items.push(owns 5)
    println(other.len())
    println(other[4])

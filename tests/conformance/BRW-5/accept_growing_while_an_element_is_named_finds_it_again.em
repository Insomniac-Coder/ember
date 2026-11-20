#$ test: run-pass
#$ rules: BRW-5, BRW-16
#$ stdout: 10 2
# `push` may move the list's buffer, and takes `ref mut v` with an empty projection, so it overlaps every
# place under `v`. Since 0.9.15 a name made by indexing is a position (`[BRW-16]`): the push is allowed while
# `a` is used, and `a` finds its element again through the new buffer.

fn main():
    v: Array[i32] = Array[i32]()
    v.push(owns 1)
    a: ref mut i32 = ref mut v[0]
    v.push(owns 2)
    a = 10
    println(v[0], v[1])

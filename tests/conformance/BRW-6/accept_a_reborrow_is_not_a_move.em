#$ test: run-pass
#$ rules: BRW-6, FN-1
# "Passing a `ref mut` local to a `mut` parameter reborrows rather than moving
# it." So `r` survives the first call and is still usable for the second — if
# it moved, the second line would be `E3040`.

fn bump(ref mut n: i32):
    n = n + 1

fn main():
    x: i32 = 1
    r: ref mut i32 = ref mut x
    bump(ref mut r)
    bump(ref mut r)
    println(x)
#$ stdout: 3

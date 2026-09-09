#$ test: run-pass
#$ rules: BRW-2
# "A borrow is live from its creation until the last use of any value derived
# from it… **Scope end is irrelevant.**" `r`'s last use is the `println`, so
# the write afterwards is fine even though `r` is still in scope.

fn main():
    x: i32 = 1
    r: ref i32 = ref x
    println(r)
    x = 5
    println(x)
#$ stdout: 1
#$ 5

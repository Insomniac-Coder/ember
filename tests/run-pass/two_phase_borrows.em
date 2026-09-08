#$ test: run-pass
#$ rules: BRW-3, BRW-1, BRW-2
#$ stdout: 3
#$ stdout: 6

## `[BRW-3]` — the mutable borrow of the receiver is *reserved* where it is
## taken and *activated* at the call, so a shared borrow in the arguments is
## permitted in between. Without it `v.push(v.len())` is illegal, which is the
## example the rule is written around.
##
## The window spans blocks: evaluating an argument that is itself a call ends
## the block, so the reservation and its activation are never adjacent.

fn main():
    xs: Array[i32] = Array[i32]()
    xs.push(1)
    xs.push(2)
    xs.push(xs.len() as i32)
    println(xs.len())

    ys: Array[i32] = Array[i32]()
    ys.push(5)
    ## Nested the other way: the argument reads the receiver twice.
    ys.push((ys.len() as i32) + (ys.len() as i32))
    println(ys.len() + 4)

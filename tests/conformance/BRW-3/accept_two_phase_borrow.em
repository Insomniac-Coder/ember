#$ test: run-pass
#$ rules: BRW-3
# "For `v.push(v.len())`, the mutable auto-borrow of `v` for the receiver is
# *reserved* first and *activated* only when the call happens; shared borrows
# in the arguments are permitted in between." Without two-phase borrows this
# ordinary line is rejected, which is why the rule exists.

fn main():
    v: Array[i32] = Array[i32]()
    v.push(1)
    v.push(v.len() as i32)
    println(v[1])
#$ stdout: 1

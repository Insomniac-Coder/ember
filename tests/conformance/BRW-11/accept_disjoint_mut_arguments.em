#$ test: run-pass
#$ rules: BRW-11, BRW-4
# A call may borrow disjoint fields mutably at the same time.

struct Pair:
    pub left: i32
    pub right: i32

fn update_both(mut left: i32, mut right: i32):
    left = 1
    right = 2

fn main():
    pair = Pair(0, 0)
    update_both(pair.left, pair.right)
    println(pair.left + pair.right)
#$ stdout: 3

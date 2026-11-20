# Clause BCK-3.1: "checked against the views live there whose places it may reach"
#$ test: run-pass
#$ rules: BCK-3, BCK-8
#$ stdout: 1
#$ stdout: a
# The calls change `OTHER`, which no live view reaches.

static mut NUMS: Array[int] = []
static mut OTHER: Array[int] = []

fn bump_other():
    for ref mut x in OTHER:
        x += 1
    OTHER.push(owns 0)

fn main():
    NUMS.push(owns 1)
    names = [String("a")]
    first = ref NUMS[0]
    v = names[0]
    bump_other()
    println(first)
    println(v)

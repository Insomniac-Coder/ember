# Clause EFF-25.3: "A fact worked out from a body never changes what a line does: it decides only
# whether a line compiles, or removes a test that would always pass."
#$ test: run-pass
#$ rules: EFF-25, EFF-24
#$ stdout: 1
#$ stdout: 2
#$ stdout: 2
# The same calls as reject_the_summary_decides_each_rule_through_a_call, placed where they compile:
# each does what its lines say, and nothing the summary found adds or removes a step.

static mut NUMS: Array[int] = []

fn grow():
    NUMS.push(owns 1)

fn bump_all():
    for ref mut x in NUMS:
        x += 1

fn main():
    NUMS.push(owns 1)
    for n in NUMS:
        println(n)
    grow()
    first = ref NUMS[0]
    println(first + 1)
    bump_all()
    println(NUMS.len())

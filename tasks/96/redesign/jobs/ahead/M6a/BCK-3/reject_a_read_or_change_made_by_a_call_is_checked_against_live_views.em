# Clause BCK-3.1: "At each point, a read or a change of a place, made directly or by a call through
# its callee's summary, is checked against the views live there whose places it may reach"
#$ test: compile-fail
#$ rules: BCK-3, BCK-8, BRW-1, BRW-17, EFF-25
# Each read or change is made by a call: a change of what a live read-only view reads (`E3021`, at
# the call), a reshape of what a live address points into (`E3060`, at the later use), a read and a
# change of a live `ref mut` name's place (`E3021`, `E3022`, at the call). No text in the spec for
# these programs: code only.

static mut NUMS: Array[int] = []
static mut NAMES: Array[String] = []

fn bump_all():
    for ref mut x in NUMS:
        x += 1

fn rename_all():
    for ref mut n in NAMES:
        n = String("x")

fn show_first():
    println(NUMS[0])

fn main():
    NUMS.push(owns 1)
    NAMES.push(owns String("a"))
    first = ref NUMS[0]
    bump_all()    #$ error[E3021]
    println(first)
    v = NAMES[0]
    rename_all()
    println(v)    #$ error[E3060]
    r = ref mut NUMS[0]
    show_first()    #$ error[E3021]
    r += 1
    q = ref mut NUMS[0]
    bump_all()    #$ error[E3022]
    q += 1

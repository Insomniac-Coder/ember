#$ test: run-pass
#$ rules: R1, R5
#$ stdout: 1
## R1, R5: the nearest program that is allowed: the `drop` keeps a number read from its object, a value
## of its own, not the object (completeness gap G13). `main` ends `e`, which counts as a call of its
## `drop` (verdict 57); what that `drop` changes is worked out from its body, with no word on any first
## line (verdict 86).

static mut KEPT_HP: Array[int] = []

class Enemy:
    hp: int = 1

    fn drop():
        KEPT_HP.push(this.hp)

fn main():
    e = Enemy()
    println(e.hp)

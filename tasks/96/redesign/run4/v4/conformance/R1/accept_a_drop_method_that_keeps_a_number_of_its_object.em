#$ test: run-pass
#$ rules: R1, R5
#$ stdout: 1
## R1, R5: the nearest program that is allowed: the `drop` keeps a number read from its object, a value
## of its own, not the object (completeness gap G13); `main` ends `e` and names what its `drop` changes
## (verdict 57).

static mut KEPT_HP: Array[int] = []

class Enemy:
    hp: int = 1

    fn drop() changes KEPT_HP:
        KEPT_HP.push(this.hp)

fn main() changes KEPT_HP:
    e = Enemy()
    println(e.hp)

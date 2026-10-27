#$ test: compile-fail
#$ rules: R1, R5
#$ help: keep what you need from it instead: write `static mut KEPT_HP: Array[int] = []` in place of line 10 and `KEPT_HP.push(this.hp)` in place of line 16
## R1, R5: a `drop` ends its object and does not own it to give away: `KEPT.push(this)` would hand over
## the enemy being ended, which would live on in `KEPT` after its memory is freed (E3013, a hand-over out
## of a name that does not own the value; completeness gap G13). The help's two lines are the whole fix:
## what the `drop` changes is worked out from its body, with no word on any first line (verdict 86). The
## nearest program that is allowed is `accept_a_drop_method_that_keeps_a_number_of_its_object`.

static mut KEPT: Array[Enemy] = []

class Enemy:
    hp: int = 1

    fn drop():
        KEPT.push(this)        #$ error[E3013]: `KEPT.push(this)` would hand over the enemy `drop` is ending, which ends when `drop` returns

fn main():
    e = Enemy()
    println(e.hp)

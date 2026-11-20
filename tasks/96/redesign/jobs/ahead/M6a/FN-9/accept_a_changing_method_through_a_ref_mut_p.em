# Clause FN-9.2 and FN-9.4: "`ref mut p: Player` may change the object"
#$ test: run-pass
#$ rules: FN-9, EXC-15
#$ stdout: 10
# The way out of reject_a_changing_method_through_a_p_that_only_reads.

class Player:
    hp: int = 0

    fn add(n: int):
        this.hp += n

    fn heal():
        this.add(10)

fn tend(ref mut p: Player):
    p.heal()

fn main():
    a = Player()
    tend(ref mut a)
    println(a.hp)

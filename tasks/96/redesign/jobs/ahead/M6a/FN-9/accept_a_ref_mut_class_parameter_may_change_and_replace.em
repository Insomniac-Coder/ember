# Clause FN-9.2: "`ref mut p: Player` may change the object, and `p = Player()` gives the caller's
# place another object (the old one ends)"
#$ test: run-pass
#$ rules: FN-9, CLS-6
#$ stdout: drop 1
#$ stdout: 7
#$ stdout: drop 1
#$ stdout: 7
#$ stdout: drop 7
#$ stdout: drop 7
# A local's place and a list element's place each get another object; the old one ends at the
# line that replaces it, and the new ones at the end of `main`.

class Player:
    hp: int = 0

    fn drop():
        println("drop", this.hp)

fn renew(ref mut p: Player):
    p.hp += 1
    p = Player()
    p.hp = 7

fn main():
    a = Player()
    renew(ref mut a)
    println(a.hp)
    team = [Player()]
    renew(ref mut team[0])
    println(team[0].hp)

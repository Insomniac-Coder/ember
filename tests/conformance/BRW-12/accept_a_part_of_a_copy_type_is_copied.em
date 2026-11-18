# Clause BRW-12.3: "except that a part of a `Copy` type (a number, a `bool`, a `char`, a variant with
# no payload, a `StrView`, a `Span[T]`, a `ref T`, a `Copy` struct) is copied, as `=` copies a whole
# name of that type (`[OWN-7]`, `[OWN-9]`): `w = this.rest` for `rest: StrView` is a `StrView` of its
# own that looks at what the field looks at, so `this.rest = w[i + 1..]` while `w` is used changes
# nothing `w` reads"
#$ test: run-pass
#$ rules: BRW-12, OWN-7
#$ stdout: 10 1
#$ stdout: true
#$ stdout: ab
#$ stdout: cd

struct Enemy:
    hp: int
    alive: bool

struct Parser:
    rest: StrView
    fn next_word() -> StrView:
        w = this.rest
        if w.find(" ") exists as i:
            this.rest = w[i + 1..]
            return w[..i]
        this.rest = ""
        return w

fn main():
    enemies = [Enemy(hp=10, alive=true)]
    hp = enemies[0].hp
    alive = enemies[0].alive
    enemies[0].hp = 1
    enemies[0].alive = false
    println(hp, enemies[0].hp)
    println(alive)
    text = String("ab cd")
    p = Parser(rest=text)
    println(p.next_word())
    println(p.next_word())

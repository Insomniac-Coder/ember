#$ test: compile-fail
#$ rules: OWN-3
#$ help: push a player of your own each round: `v.push(copy of template)`
## design rules: R1, R2
## R1, R2: `v.push(template)` hands the whole name `template` over in the first round, and the loop
## comes back to it in the second, after it ended (E3040; AN-35's OLD loop, today's E3041, which
## `V4/rules.md` section 4 does not use: a hand-over in a loop's later round is E3040).

@derive(Clone)
class Player:
    name: String

fn main():
    template = Player(name=String("ann"))
    v: Array[Player] = []
    for _ in 0..3:
        v.push(template)       #$ error[E3040]: `v.push(template)` hands `template` over to `v` on line 16, and the loop comes back to line 16 with `template` ended
    println(v.len())

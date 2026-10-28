#$ test: compile-fail
#$ rules: R3
#$ help: write `g2 = shares game`, or, for two owners that both change the game, write `game = shares mut Game()` instead of line 11
## R3: an owner made from another may do no more than it: `shares mut x` needs `x` to be a name that
## may change the value (E3023; `V4/rules.md` section 3's message).

class Game:
    score: int = 0

fn main():
    game = shares Game()
    g2 = shares mut game       #$ error[E3023]: `g2 = shares mut game` would change the `Game` through `game`, which only reads it (`game = shares Game()`, line 11), and an owner made from it only reads too
    println(g2.score)

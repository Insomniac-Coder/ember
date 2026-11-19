#$ test: compile-fail
#$ rules: OWN-13, RC-7, FN-1
# `[OWN-13]` — a plain parameter of a `shares mut` type only reads the caller's value, so an owner
# made from it only reads too (`E3023`); a function that hands on an owner that may change takes one.

class Game:
    score: int = 0

fn setup(game: shares mut Game) -> shares mut Game:
    return shares mut game            #$ error[E3023]: `return shares mut game` would change the `Game` through `game`, which only reads the caller's game (a plain parameter), and an owner made from it only reads too; write `return shares game`, or, to hand on an owner that may change, take one: write `owns game: shares mut Game` for the parameter, and `shares mut` before the argument at each call

fn main():
    g = shares mut Game()
    println(setup(g).score)

#$ test: compile-fail
#$ rules: R4, R3
#$ help: fn setup(owns game: shares mut Game) -> Holder:
## R4, verdict 32: `game: shares mut Game` only reads the caller's game, so an owner made from it
## that may change the game (`shares mut game`) is refused; a function that hands one on takes one
## (`owns game: shares mut Game`, AN-78's `setup`).

class Game:
    score: int = 0

class Holder:
    game: shares mut Game

fn setup(game: shares mut Game) -> Holder:
    return Holder(game=shares mut game)    #$ error[E3023]: `game` only reads the `Game` that `main` lends to `setup` (`shares mut Game` says what `main`'s game is, not what `setup` may do with it), so an owner made from it only reads too; to hand on an owner that may change the game, take one: write `fn setup(owns game: shares mut Game) -> Holder:`, call it `setup(shares mut game)`, and write `return Holder(game=game)`

fn main():
    game = shares mut Game(score=5)
    h = setup(game)
    println(h.game.score)

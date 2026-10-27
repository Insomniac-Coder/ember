#$ test: run-pass
#$ rules: R4, R3
#$ stdout: 5
## R4, verdict 32: `owns game: shares mut Game` makes `setup` one more owner that may change the game;
## the call writes the word before the value, `setup(shares mut game)`, and `setup` hands its owner on
## with no word (AN-78's `setup`).

class Game:
    score: int = 0

class Holder:
    game: shares mut Game

fn setup(owns game: shares mut Game) -> Holder:
    return Holder(game=game)

fn main():
    game = shares mut Game(score=5)
    h = setup(shares mut game)
    println(h.game.score)

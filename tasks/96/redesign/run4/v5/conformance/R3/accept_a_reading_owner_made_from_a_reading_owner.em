#$ test: run-pass
#$ rules: OWN-8
#$ stdout: 0
## design rules: R3
## R3: the nearest program that is allowed: an owner that reads, made from one that reads.

class Game:
    score: int = 0

fn main():
    game = shares Game()
    g2 = shares game
    println(g2.score)

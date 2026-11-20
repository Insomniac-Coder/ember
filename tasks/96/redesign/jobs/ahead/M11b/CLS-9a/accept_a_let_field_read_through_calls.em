# Clause CLS-9a.1: "After `init`, nothing is changed through a `let` field"
#$ test: run-pass
#$ rules: CLS-9a, BRW-10
#$ stdout: 1
# A method whose body only reads, called through a `let` field, and the field given to a plain
# parameter, change nothing.

class Board:
    cells: Array[int] = []

    fn size() -> int:
        return this.cells.len()

fn count(xs: Span[int]) -> int:
    return xs.len()

class Game:
    let board: Board
    let items: Array[int]

    fn init():
        this.board = Board()
        this.items = [1]

    fn report() -> int:
        return this.board.size() + count(this.items)

fn main():
    g = Game()
    println(g.report())

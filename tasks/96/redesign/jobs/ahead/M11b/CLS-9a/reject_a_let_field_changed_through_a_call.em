# Clause CLS-9a.1: "After `init`, nothing is changed through a `let` field, for every type (...): a
# number or struct field never changes, a list field is never added to, and a class-object field is
# never pointed at another object nor its object changed through it (`E2104`"
#$ test: compile-fail
#$ rules: CLS-9a, BRW-10, EFF-24
# The changes come through calls: a method of the field's object whose body changes it (by its
# summary), and the field given `ref mut` to a function. The existing CLS-9a tests change the field
# directly. No text in the spec for these: code only.

class Board:
    cells: Array[int] = []

    fn clear():
        this.cells.clear()

    fn size() -> int:
        return this.cells.len()

fn grow(ref mut xs: Array[int]):
    xs.push(owns 1)

class Game:
    let board: Board
    let items: Array[int]

    fn init():
        this.board = Board()
        this.items = [1]

    fn tidy():
        this.board.clear()    #$ error[E2104]
        grow(ref mut this.items)    #$ error[E2104]

    fn report() -> int:
        return this.board.size() + this.items.len()

fn main():
    g = Game()
    g.tidy()
    println(g.report())

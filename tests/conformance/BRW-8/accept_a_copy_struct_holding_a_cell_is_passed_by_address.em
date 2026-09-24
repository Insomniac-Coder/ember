#$ test: run-pass
#$ rules: BRW-8, CELL-4
#$ stdout: 2
# `[BRW-8]` (ODR-024) — a `Copy` value holding a `Cell` is passed by address,
# so a write through the cell reaches the caller's value; as a copy it printed
# 0, which the old "cannot be observed" did not allow.

@derive(Copy)
struct Ticker:
    count: Cell[int]

fn tick(t: Ticker):
    t.count.set(t.count.get() + 1)

fn main():
    ticker = Ticker(count=Cell(0))
    tick(ticker)
    tick(ticker)
    println(ticker.count.get())

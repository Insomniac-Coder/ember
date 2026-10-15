# Call `call_once` 100 million times on a `Once` that has already run, adding under it: the cost of
# the check every later call makes (`[THR-9]`). The C twin makes the same acquire load and branch.

from std.sync import Once

fn main():
    init = Once()
    total = Cell(0)
    n = 100_000_000
    for i in range(n):
        init.call_once(fn() => total.set(total.get() + 1))
        total.set(total.get() + i % 3)
    println(total.get())

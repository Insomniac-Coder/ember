# Clause BRW-21.6: "A `ref mut` parameter counts the same way: with `fn rename(ref mut s: String):`
# whose body is `LOG.clear()` then `s = String("renamed")`, `rename(ref mut LOG[0])` is refused"
#$ test: compile-fail
#$ rules: BRW-21, BCK-8
# The spec's message, with this file's line numbers.

static mut LOG: Array[String] = []

fn rename(ref mut s: String):
    LOG.clear()
    s = String("renamed")

fn main():
    LOG.push(owns String("a"))
    rename(ref mut LOG[0])    #$ error[E3031]: `rename` clears `LOG` (its line 10) and then writes `s` (its line 11), and `LOG[0]` is part of `LOG`, so `s` would point at freed memory; rename a copy of your own: write `mine = copy of LOG[0]` and `rename(ref mut mine)` in place of this line, or in `rename` write `s` before `LOG.clear()`

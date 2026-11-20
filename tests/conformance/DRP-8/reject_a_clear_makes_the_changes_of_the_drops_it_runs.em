# Clause DRP-8.1: "Anything that may end a value whose type has a `drop` (a list cleared, a value
# replaced, a local ending with its function, (...)) makes the changes that `drop` makes, wherever a
# value of its type may end, for every rule that needs them (`[EFF-1]`). With `class Enemy:` whose
# `fn drop():` pushes to `LOG`, `enemies.clear()` inside `for line in LOG:` is `E3020`"
#$ test: compile-fail
#$ rules: DRP-8, BRW-18, EFF-1
# The spec's message, with this file's line numbers.

static mut LOG: Array[String] = []

fn main():
    enemies = [Enemy(), Enemy()]
    LOG.push(owns String("start"))
    for line in LOG:
        println(line)
        enemies.clear()    #$ error[E3020]: `enemies.clear()` (line 16) ends every `Enemy` in `enemies`, and an `Enemy`'s `drop` pushes to `LOG` (its line 22), which this loop walks (line 14); clear after the loop: move line 16, `enemies.clear()`, below the loop

class Enemy:
    hp: int = 1

    fn drop():
        LOG.push(owns String("enemy gone"))

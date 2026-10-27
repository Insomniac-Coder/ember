#$ test: compile-fail
#$ rules: BRW-18, CTL-2
#$ help: call `spawn_one()` after the loop: move line 26 below the loop
## design rules: R1, R5, R8
## R1, R5, R8 (c): a local's value ends with its function, and ending an object counts as a call of its
## `drop` (verdict 57): `e` ends when `spawn_one` returns, and an `Enemy`'s `drop` pushes to `LOG`, so a
## call of `spawn_one()` adds to `LOG`. The compiler works that out from the bodies, with no word on any
## first line (verdict 86), and refuses the call inside a loop over `LOG` (E3020), naming the chain. The
## nearest program that is allowed is `accept_a_function_whose_local_ends_with_a_drop_called_after_the_loop`.

static mut LOG: Array[String] = []

class Enemy:
    hp: int = 1

    fn drop():
        LOG.push(String("gone"))

fn spawn_one():
    e = Enemy()
    println(e.hp)

fn main():
    LOG.push(String("start"))
    for line in LOG:
        println(line)
        spawn_one()            #$ error[E3020]: `spawn_one()` (line 26) adds to `LOG` (its local `e`, line 19, ends when it returns, and an `Enemy`'s `drop` pushes to `LOG`, line 16) while this loop walks `LOG` (line 24)
    println(LOG.len())

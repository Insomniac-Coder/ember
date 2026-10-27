#$ test: compile-fail
#$ rules: R2, R5, R8
#$ help: call `swap_in()` after the loop: move line 28 below the loop
## R2, R5, R8 (c): `=` on a name that has a value ends its old value, and ending an object counts as a
## call of its `drop` (verdict 57): the old `Res` ends on line 21, and a `Res`'s `drop` pushes to `LOG`,
## so a call of `swap_in()` adds to `LOG`. The compiler works that out from the bodies, with no word on
## any first line (verdict 86), and refuses the call inside a loop over `LOG` (E3020), naming the chain.
## The nearest program that is allowed is
## `accept_a_function_that_replaces_an_object_whose_drop_pushes_to_log_called_after_the_loop`.

static mut LOG: Array[String] = []

class Res:
    name: String

    fn drop():
        LOG.push(String("ended"))

fn swap_in():
    a = Res(name=String("one"))
    a = Res(name=String("two"))
    println(a.name)

fn main():
    LOG.push(String("start"))
    for line in LOG:
        println(line)
        swap_in()              #$ error[E3020]: `swap_in()` (line 28) adds to `LOG` (`a = Res(...)`, line 21, ends the old `Res`, and a `Res`'s `drop` pushes to `LOG`, line 17) while this loop walks `LOG` (line 26)
    println(LOG.len())

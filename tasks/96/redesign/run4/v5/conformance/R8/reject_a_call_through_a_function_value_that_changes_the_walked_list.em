#$ test: compile-fail
#$ rules: R8, R12
#$ help: move line 12 below the loop
## R8 (c), R12: a call through a function value makes the changes of every function the program can
## put there (verdict 86): `hook` may run `note`, which pushes to `LOG`, so a loop over `LOG` may not
## call it. The message names the chain (R18).

static mut LOG: Array[String] = []

fn replay(hook: fn()):
    for line in LOG:
        hook()    #$ error[E3020]: `hook()` (line 12) adds to `LOG` (it may run `note`, given on line 20, which pushes to `LOG`, line 16) while this loop walks `LOG` (line 11); call `hook()` after the loop: move line 12 below the loop
        println(line)

fn note():
    LOG.push(String("x"))

fn main():
    LOG.push(String("start"))
    replay(note)

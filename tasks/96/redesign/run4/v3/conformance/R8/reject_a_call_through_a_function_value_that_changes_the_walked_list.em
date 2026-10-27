#$ test: compile-fail
#$ rules: R8, R12
#$ help: hook()
## R8 (c), R12: a call through a function value makes the changes its type says; `hook` is a
## `fn() changes LOG`, so calling it inside a loop over `LOG` is refused (verdict 17).

static mut LOG: Array[String] = []

fn replay(hook: fn() changes LOG) changes LOG:
    for line in LOG:
        hook()    #$ error[E3020]: this loop walks `LOG` (line 10), and `hook()` (line 11) changes `LOG` (its type says `fn() changes LOG`); write `hook()` below the loop instead of in it
        println(line)

fn note() changes LOG:
    LOG.push(String("x"))

fn main() changes LOG:
    LOG.push(String("start"))
    replay(note)

#$ test: compile-fail
#$ rules: R4, R5, R8
#$ help: note(copy of LOG[0])
## R4, verdict 53: the fact is worked out through the functions the callee calls: `note` calls `wipe`,
## which clears `LOG`, and then uses `s`, so `note(LOG[0])` is refused as if `note` cleared `LOG`
## itself (E3031).

static mut LOG: Array[String] = []

fn wipe() changes LOG:
    LOG.clear()

fn note(s: StrView) changes LOG:
    wipe()
    println(s)

fn main() changes LOG:
    LOG.push(String("hello"))
    note(LOG[0])    #$ error[E3031]: `note` changes `LOG` and uses `s` after `wipe()` (line 14), which clears `LOG` (line 11), and `LOG[0]` is part of `LOG`; give it a copy: `note(copy of LOG[0])`, or in `note` use `s` before `wipe()`

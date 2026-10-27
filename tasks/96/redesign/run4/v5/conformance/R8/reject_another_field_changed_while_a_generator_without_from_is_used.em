#$ test: compile-fail
#$ rules: BRW-1
#$ help: gen fn replay() -> Generator[int] from log:
## design rules: R8, R6
## R6, R8 (g): a generator method without `from` gives a view of its whole object, so no field of `d`
## changes while `g` will still be used (E3021, at the change); `from log` narrows the view to `d.log`.
## AN-77, the v3 note's point 2; settled in step 4 (`V5/changes.md` section 8, item 8). The allowed
## neighbour is `accept_another_field_changes_while_a_from_log_generator_is_used.em`.

class Door:
    angle: float = 0.0
    log: Array[String] = []

    gen fn replay() -> Generator[int]:
        for entry in this.log:
            yield entry.len()

fn main():
    d = Door()
    d.log.push(String("shut"))
    g = d.replay()
    d.angle = 90.0    #$ error[E3021]: `d.angle = 90.0` (line 21) changes `d`, which `g` views (`g = d.replay()`, line 20: `replay` names no `from`, so its generator views all of `d`), and `g` is used on line 22; change it after `g`'s last use: move line 21 below line 22; or let `replay` view only its log: write `gen fn replay() -> Generator[int] from log:` in place of line 13
    println(g.next().unwrap())

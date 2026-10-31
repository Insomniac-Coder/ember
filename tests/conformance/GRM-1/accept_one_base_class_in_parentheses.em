#$ test: run-pass
#$ rules: GRM-1
#$ stdout: 2
## `[GRM-1]` — a class names its one base class in parentheses.

open class Script:
    fn priority() -> int:
        return 1

class Door(Script):
    fn open_priority() -> int:
        return this.priority() + 1

fn main():
    println(Door().open_priority())

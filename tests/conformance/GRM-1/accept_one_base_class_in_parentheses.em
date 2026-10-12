#$ test: run-pass
#$ rules: GRM-1
#$ stdout: 2
## `[GRM-1]` — a class names its one base class in parentheses.

open class Script:
    fn priority(self) -> int:
        return 1

class Door(Script):
    fn open_priority(self) -> int:
        return self.priority() + 1

fn main():
    println(Door().open_priority())

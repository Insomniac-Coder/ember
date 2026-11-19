#$ test: run-pass
#$ rules: OWN-3, DRP-6
#$ stdout: kept
#$ stdout: ended
#$ stdout: ended
# `[OWN-3]` — a value handed over on only some paths is tracked with a hidden flag.

class Note:
    v: int = 0

    fn drop():
        println("ended")

fn pick(give: bool):
    n = Note()
    if give:
        _kept = n
        println("kept")

fn main():
    pick(true)
    pick(false)

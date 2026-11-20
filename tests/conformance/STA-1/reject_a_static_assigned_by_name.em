#$ test: compile-fail
#$ rules: STA-1, GRM-4
#$ help: for a module variable that functions change declare it `static mut HOOK`; for a local of your own use another name
# `[STA-1]` — a `static` never changes: `HOOK = v` in a function is `E3023`, whose help names
# `static mut HOOK` or another name for a local; it never makes a local that hides the `static`
# (`[GRM-4]`: `HOOK` is in scope, so the line assigns). An augmented assignment is the same.

static HOOK: int = 3
static LIMIT: int = 10

fn set():
    HOOK = 4    #$ error[E3023]: `HOOK` is a `static`, which never changes

fn grow():
    LIMIT += 1    #$ error[E3023]: `LIMIT` is a `static`, which never changes

fn main():
    set()
    grow()

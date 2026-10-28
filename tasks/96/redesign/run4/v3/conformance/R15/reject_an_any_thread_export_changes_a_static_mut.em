#$ test: compile-fail
#$ rules: R15
#$ help: guard it with a lock: write `static HITS_LOCK: Lock = Lock()` and `static mut HITS: int guarded by HITS_LOCK = 0`, and put `bump`'s body inside `with HITS_LOCK:`; or count it with an atomic: `static HITS: Atomic[int] = Atomic(0)`, and `HITS.fetch_add(1)` in `bump`
## R15: a module variable of the main thread reached by code that runs on another thread, here an
## `@export(threads=any)` function, is E7012 (AN-84's message).

static mut HITS: int = 0

@export(threads=any)
fn bump() changes HITS:
    HITS += 1    #$ error[E7012]: `bump` may be called from any thread (`@export(threads=any)`), and it changes `HITS`, a module variable of the main thread

fn main():
    println(HITS)

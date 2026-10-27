#$ test: compile-fail
#$ rules: R15
#$ help: guard it with a lock: write `from std.sync import Lock` at the top, `static HITS_LOCK: Lock = Lock()` and `static mut HITS: int guarded by HITS_LOCK = 0` instead of line 8, put line 12 inside `with HITS_LOCK:`, and line 15 inside `with HITS_LOCK:` too; or count it with an atomic: write `from std.sync import Atomic` at the top, `static HITS: Atomic[int] = Atomic(0)` instead of line 8, `HITS.fetch_add(1)` instead of line 12 and `println(HITS.load())` instead of line 15
## R15: a module variable of the main thread reached by code that runs on another thread, here an
## `@export(threads=any)` function, is E7012 (AN-84's message; its help with the imports and every line
## it replaces, check-V6's fix).

static mut HITS: int = 0

@export(threads=any)
fn bump() changes HITS:
    HITS += 1    #$ error[E7012]: `bump` may be called from any thread (`@export(threads=any)`), and it changes `HITS`, a module variable of the main thread

fn main():
    println(HITS)

#$ test: run-pass
#$ rules: THR-9, THR-3, STA-1
#$ profiles: debug, release
#$ stdout: false
#$ setting up for a
#$ true
#$ ['first']
# `[THR-3]` — `Once` runs an initialiser exactly once: the first `call_once` runs its function,
# every later one returns without running its own. `[THR-9]` — a `Once` is `Sync`, so a `static`
# may be one (`[STA-1]`).

from std.sync import Once

static INIT: Once = Once()

fn setup(tag: str):
    INIT.call_once(fn() => println("setting up for", tag))

fn main():
    println(INIT.is_completed())
    setup("a")
    setup("b")
    println(INIT.is_completed())
    local = Once()
    names: Array[String] = []
    local.call_once(fn() => names.push("first".to_string()))
    local.call_once(fn() => names.push("second".to_string()))
    println(names)

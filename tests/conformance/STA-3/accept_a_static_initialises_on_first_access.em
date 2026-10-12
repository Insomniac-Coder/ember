#$ test: run-pass
#$ rules: STA-3, STA-1, STA-2
#$ stdout: start
#$ stdout: init
#$ stdout: 42
#$ stdout: 42
#$ stdout: b
#$ stdout: 45
## `[STA-3]` — a static whose initialiser compile time cannot evaluate runs
## it once, on first access: after `start`, and only once however often it is
## read. One static may read another (D-586: anything but a literal was
## `E2130`).

fn compute() -> int:
    println("init")
    return 41

static ANSWER: int = compute() + 1
static NAMES: Array[str] = ["a", "b", "c"]
static TOTAL: int = ANSWER + NAMES.len()

fn main():
    println("start")
    println(ANSWER)
    println(ANSWER)
    println(NAMES[1])
    println(TOTAL)

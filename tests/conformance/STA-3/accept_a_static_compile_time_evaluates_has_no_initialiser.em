#$ test: run-pass
#$ rules: STA-3
#$ stdout: 42 8
#$ assert-c: !contains("_static(void)")
## `[STA-3]` — a static whose initialiser compile time evaluates is placed in
## the image with no initialisation code: nothing runs on first access.

static DOUBLE: int = 2 * 21
static SHIFTED: int = DOUBLE // 5

fn main():
    println(DOUBLE, SHIFTED)

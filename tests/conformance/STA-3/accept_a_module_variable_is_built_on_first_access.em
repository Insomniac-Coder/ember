#$ test: run-pass
#$ rules: STA-3, STA-1, CT-5
#$ stdout: 6
#$ stdout: first second
#$ stdout: 2
#$ assert-c: _static(void)
# `[STA-3]`, `[CT-5]` — a `static mut` whose value owns heap memory is built on first access, in
# writable memory, so that it may grow or replace its strings; one whose initialiser reads another
# `static mut` runs on first access too, seeing the value the program gave it (`[CT-1a]`: what
# compile time evaluates reaches no module variable).

static mut START: int = 1
static mut NEXT: int = START + 1
static mut NAMES: Array[String] = [String("start")]

fn main():
    START = 5
    println(NEXT)
    NAMES[0] = String("first")
    NAMES.push(owns String("second"))
    println(NAMES[0], NAMES[1])
    println(NAMES.len())

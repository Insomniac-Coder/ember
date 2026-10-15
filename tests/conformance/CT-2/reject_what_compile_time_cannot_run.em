#$ test: compile-fail
#$ rules: CT-2, CT-1
#$ error[E6010]: `GREETING` cannot run while compiling: it reaches console output
#$ error[E6010]: `comptime.env` reads a build input, which only compile time has
#$ help: use it inside `comptime(…)`, a `comptime:` block, or a constant's or static's initialiser
# `[CT-2]` — an operation that cannot run at compile time (I/O, a foreign call, a thread, a clock,
# `unsafe`) is `E6010`, naming the call chain; and the two inputs exist only at compile time.

fn greet() -> int:
    println("hello")
    return 1

const GREETING: int = greet()

fn main():
    home = comptime.env("HOME")
    println(GREETING)

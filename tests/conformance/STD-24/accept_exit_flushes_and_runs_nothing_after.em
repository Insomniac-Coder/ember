#$ test: run-pass
#$ rules: STD-24, FN-8
#$ args: héllo
#$ stdout: 2 héllo
#$ stdout: 2 104 195 169
#$ stdout: true true
#$ stdout: before
#$ exit: 3
# `[STD-24]` — `process.args()` is the command line as `main`'s `args` would hold it; `args_os()`
# its own bytes (`é` is two). `env.var(name)` is `None` for a name that is not set.
# `process.exit(code)` flushes the standard streams and ends the process at once: neither the
# `defer` block nor `Noisy`'s `drop` runs, and the status is `code`.

import std.process
import std.env

struct Noisy:
    value: int

extend Noisy:
    fn drop(mut self):
        println("dropped")

fn main():
    args = process.args()
    println(args.len(), args[1])
    raw = process.args_os()
    println(raw.len(), raw[1][0], raw[1][1], raw[1][2])
    println(env.var("PATH").is_some(), env.var("EMBER_STD_24_NOT_SET").is_none())
    _noisy = Noisy(value = 1)
    defer:
        println("deferred")
    println("before")
    process.exit(3)

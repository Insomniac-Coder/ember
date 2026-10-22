#$ test: run-pass
#$ rules: CLO-1, CLO-3, CLO-6, OWN-3, TST-19
#$ stdout: 7

# `[CLO-6]` — an `owned fn` parameter is a callable bound like any other (`[CLO-3]`),
# taken over by the callee. A capturing closure proves this is a static generic callable
# boundary, not a function-pointer-only special case; it may be called again (D-850).

fn invoke(owned f: fn() -> i32) -> i32:
    return f()

fn main():
    values: Array[i32] = Array[i32]()
    values.push(7)
    println(invoke(owned fn() => values[0]))

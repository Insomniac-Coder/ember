#$ test: run-pass
#$ rules: CLO-1, CLO-3, CLO-6, OWN-3, TST-19
#$ stdout: 7

# `[CLO-6]` supplies an implicit `CallableOnce` bound for an `owned fn`
# parameter. A capturing closure proves this is a static generic callable
# boundary, not a function-pointer-only special case. The binding is consumed
# by the indirect call even when its concrete representation is `Copy`.

fn invoke(owned f: fn() -> i32) -> i32:
    return f()

fn main():
    values: Array[i32] = Array[i32]()
    values.push(7)
    println(invoke(owned fn() => values[0]))

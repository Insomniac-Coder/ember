#$ test: run-pass
#$ rules: FFI-9
#$ profiles: debug, development
#$ stdout: 3
#$ stdout: 12
#$ stdout: 7
#$ assert-c: contains("int32_t EMBER_CC_SYSTEM add(int32_t")
#$ assert-c: contains("int32_t EMBER_CC_FASTCALL mul(int32_t")
#$ assert-c: contains("(EMBER_CC_SYSTEM *")
# `[FFI-9]` — an Ember function is defined in any of the conventions, and a
# function pointer type names one: `"system"` is `stdcall` on 32-bit Windows
# and C elsewhere, `"fastcall"` exists on x86. A pointer keeps the convention
# in its type, so it is called the way the function expects.

pub extern "system" fn add(a: i32, b: i32) -> i32:
    return a + b

pub extern "fastcall" fn mul(a: i32, b: i32) -> i32:
    return a * b

pub extern "C" fn sub(a: i32, b: i32) -> i32:
    return a - b

fn apply(f: extern "system" fn(i32, i32) -> i32, a: i32, b: i32) -> i32:
    return f(a, b)

fn main():
    plus: extern "system" fn(i32, i32) -> i32 = add
    times: extern "fastcall" fn(i32, i32) -> i32 = mul
    minus: extern "C" fn(i32, i32) -> i32 = sub
    println(apply(plus, 1, 2))
    println(times(3, 4))
    println(minus(9, 2))

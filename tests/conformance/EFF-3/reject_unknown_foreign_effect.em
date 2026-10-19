#$ test: compile-fail
#$ rules: EFF-3
# `[EFF-3]` — `@ffi(effects=[…])` names effects of Part X.1's table.

unsafe extern "C":
    @ffi(effects=[Alloc, Heap])    #$ error[E0104]: `effects` lists effects by name
    safe fn abs(x: i32) -> i32
    @ffi(effects=[Nondet], deterministic)    #$ error[E0104]: a `deterministic` foreign function has no `Nondet` effect
    safe fn labs(x: i64) -> i64

fn main():
    pass

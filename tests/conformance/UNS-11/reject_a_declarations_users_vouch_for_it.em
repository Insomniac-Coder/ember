#$ test: compile-fail
#$ rules: UNS-11, UNS-1
# `[UNS-11]` — `unsafe` before a declaration says that whoever uses it must vouch: implementing an
# `unsafe interface` without `unsafe extend`, calling through an `unsafe fn(…)` value and calling an
# `unsafe fn` outside unsafe code are each `E3100`.

unsafe interface Raw:
    fn address() -> usize

struct Slot:
    at: usize

extend Slot implements Raw:    #$ error[E3100]: implementing `Raw` is `unsafe`: its implementer promises what the compiler cannot check
    fn address() -> usize:
        return this.at

@safety("`p` points to one writable `i32`")
unsafe fn set(p: *mut i32, v: i32):
    write(p, 0, v)

fn apply(f: unsafe fn(*mut i32, i32), p: *mut i32):
    f(p, 1)    #$ error[E3100]: calling a value of type `unsafe fn(*mut i32, i32) -> void` requires an `unsafe` block

fn main():
    p = null[*mut i32]()
    set(p, 5)    #$ error[E3100]: `set(p, 5)` (line 26) calls an `unsafe fn`, whose callers must make true what its `@safety` says
    apply(set, p)

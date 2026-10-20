#$ test: compile-fail
#$ rules: GRM-28, UNS-1
## `[GRM-28]`, `[UNS-1]` — an `unsafe` local function is called only inside `unsafe`, whether it
## captures or not, and is never a safe callable (`[FN-6]`).

fn apply(f: fn(int) -> int, x: int) -> int:
    return f(x)

fn main():
    extra = 2
    # SAFETY: no obligation; the test is about the call
    unsafe fn plain(x: int) -> int:
        return x + 1
    # SAFETY: no obligation; the test is about the call
    unsafe fn capturing(x: int) -> int:
        return x + extra
    println(plain(1))    #$ error[E3100]: calling a value of type `unsafe fn(i64) -> i64` requires an `unsafe` block
    println(capturing(1))    #$ error[E3100]: calling an `unsafe` local function requires an `unsafe` block
    println(apply(plain, 1))    #$ error[E2020]: `apply` cannot take `unsafe fn(i64) -> i64` where it expects `fn(i64) -> i64`
    println(apply(capturing, 1))    #$ error[E2020]: `apply` cannot take `unsafe fn(i64) -> i64` where it expects `fn(i64) -> i64`

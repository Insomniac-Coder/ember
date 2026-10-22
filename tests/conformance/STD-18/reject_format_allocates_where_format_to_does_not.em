#$ test: compile-fail
#$ rules: STD-18, EFF-1
#$ error[E4001]: reaches an allocation
# `[STD-18]` — `format` builds a `String`, so a `@noalloc` function may not call it (`E4001`);
# `format_to` writes into the caller's buffer and may be called there.

from std.fmt import format_to, FmtError

@noalloc
fn into(buf: MutSpan[u8], n: int) -> Result[str, FmtError]:
    return format_to(buf, n)

@noalloc
fn built(n: int) -> int:
    return format(n).len()

fn main():
    println(built(3))

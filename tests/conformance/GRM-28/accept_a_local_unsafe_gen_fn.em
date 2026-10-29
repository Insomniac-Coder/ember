#$ test: run-pass
#$ rules: GRM-28, GRM-21, UNS-7, UNS-12
#$ stdout: 1
#$ 2
# `[GRM-28]` — a local function may be anything `fn_decl` is, `unsafe gen fn` too (D-927), with its
# `@safety`; its body vouches for its own steps block by block (`[UNS-12]`).

fn main():
    a: Array[u8] = [1, 2]
    p = a.as_span().as_ptr()
    @safety("q points to two readable bytes")
    unsafe gen fn walk(q: *u8) -> Generator[u8]:
        first: u8 = 0
        second: u8 = 0
        first = q.read()
        second = q.add(1).read()
        yield first
        yield second
    # SAFETY: `a` holds two bytes
    unsafe:
        for x in walk(p):
            println(x)

#$ test: run-pass
#$ rules: ERR-10, ERR-8, ERR-4
#$ profiles: debug, release, shipping
#$ stdout: loading app.cfg: reading the disk: disk -2
#$ AnyError(Disk(code=-2), context=['reading the disk', 'loading app.cfg'])
#$ true
#$ 30
#$ calls 1
#$ no 9
#$ lazy
# `[ERR-10]` — `r.context(msg)` converts `r`'s error into an `AnyError` and
# adds `msg` outermost, so it displays as `msg: <original>`; messages stack,
# and `with_context` builds its message only on failure. An `Option`'s
# `context` (`[ERR-4]`) makes a `None` an `AnyError` showing the message.

@derive(Error)
@error("disk {code}")
struct Disk:
    code: int

fn disk(n: int) -> Result[int, Disk]:
    if n < 0:
        return Err(Disk(n))
    return Ok(n)

fn load(n: int) -> Result[int]:
    v = disk(n).context("reading the disk")?
    return Ok(v)

fn outer(n: int, calls: ref Cell[int]) -> Result[int]:
    name = "app.cfg"
    v = load(n).with_context(fn() => message(name, calls))?
    return Ok(v * 10)

fn message(name: str, calls: ref Cell[int]) -> String:
    calls.set(calls.get() + 1)
    return f"loading {name}"

fn find(xs: Array[int], x: int) -> Result[int]:
    return xs.index_of(x).context(f"no {x}")

fn main():
    calls = Cell(0)
    match outer(-2, ref calls):
        Ok(v):
            println(v)
        Err(e):
            println(e)
            println(f"{e!r}")
            println(e.is[Disk]())
    match outer(3, ref calls):
        Ok(v):
            println(v)
        Err(e):
            println(e)
    println(f"calls {calls.get()}")
    match find([1, 2], 9):
        Ok(i):
            println(i)
        Err(e):
            println(e)
    o: Option[int] = None
    match o.with_context(fn() => "lazy"):
        Ok(v):
            println(v)
        Err(e):
            println(e)

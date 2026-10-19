#$ test: run-pass
#$ rules: ERR-8, ERR-9, ERR-2
#$ profiles: debug, development, release
#$ stdout: load failed
#$ AnyError(Load.Io(Disk(code=-3)))
#$ true false
#$ as a load: load failed / Load.Io(Disk(code=-3))
#$ not a disk
#$ caused by: disk -3
#$ 10
# `[ERR-8]` — `AnyError` holds any `Error`, boxed: `?` converts one into it
# with its `From`, and an `AnyError` passed on through `?` into another is
# moved. It shows as its error does (`Display`, `Debug`), its `source` is its
# error's cause, and `is[E]()` and `downcast_ref[E]()` find what type it is.

@derive(Error)
@error("disk {code}")
struct Disk:
    code: int

@derive(Error)
enum Load:
    @error("load failed")
    @from
    Io(Disk)
    @error("bad format")
    Format

fn disk(n: int) -> Result[int, Disk]:
    if n < 0:
        return Err(Disk(n))
    return Ok(n)

fn load(n: int) -> Result[int, Load]:
    v = disk(n)?
    if v == 0:
        return Err(Load.Format)
    return Ok(v)

fn any(n: int) -> Result[int, AnyError]:
    v = load(n)?
    return Ok(v + 1)

fn twice(n: int) -> Result[int, AnyError]:
    return Ok(any(n)? * 2)

fn main():
    match twice(-3):
        Ok(v):
            println(v)
        Err(e):
            println(f"{e}")
            println(f"{e!r}")
            println(e.is[Load](), e.is[Disk]())
            match e.downcast_ref[Load]():
                Some(l):
                    println(f"as a load: {l} / {l!r}")
                None:
                    println("not a load")
            match e.downcast_ref[Disk]():
                Some(d):
                    println(f"disk {d.code}")
                None:
                    println("not a disk")
            match e.source():
                Some(c):
                    println(f"caused by: {c}")
                None:
                    println("no cause")
    match twice(4):
        Ok(v):
            println(v)
        Err(e):
            println(f"{e}")

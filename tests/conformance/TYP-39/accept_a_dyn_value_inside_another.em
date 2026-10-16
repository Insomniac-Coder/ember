#$ test: run-pass
#$ rules: TYP-39, TYP-36, ERR-3
#$ profiles: debug, release
#$ stdout: [Fail.Disk, Fail.Net(host='example.org')]
#$ Holder(e=Fail.Disk, tries=3)
#$ Some(Fail.Net(host='x'))
#$ disk full
# `[TYP-39]` (ADR-191) — a value inside a collection, an `Option` or a struct is written by its
# `Debug`; a `dyn` one (`Box[dyn Error]`) through its table, as it is written alone by its `Display`.

@derive(Error)
enum Fail:
    @error("disk full")
    Disk
    @error("no route to {host}")
    Net(host: String)

struct Holder:
    e: Box[dyn Error]
    tries: int

fn main():
    errs: Array[Box[dyn Error]] = []
    errs.push(Box(Fail.Disk))
    errs.push(Box(Fail.Net("example.org")))
    println(errs)
    h = Holder(Box(Fail.Disk), 3)
    println(f"{h!r}")
    maybe: Option[Box[dyn Error]] = Some(Box(Fail.Net("x")))
    println(maybe)
    println(errs[0])

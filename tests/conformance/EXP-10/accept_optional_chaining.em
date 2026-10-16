#$ test: run-pass
#$ rules: EXP-10, GRM-13, ERR-8
#$ profiles: debug, release
#$ stdout: Some(3) None Some(6) None
#$ Some(5) None None
#$ Some('pat') Some(3) None
#$ pat
#$ Some(3)
#$ Some(true) None
# `[EXP-10]` — `a?.f` and `a?.m(…)` are `None` when `a` is `None`, and the member of what it
# holds otherwise: wrapped in `Some`, or itself when it is an `Option` (a chain is one `Option`), or
# nothing when the method gives nothing. `a` is read as `[GRM-13]` reads a place it does not
# consume, so a `String` field is lent (`Option[ref String]`) and `a` stays usable.

@derive(Error)
enum Fail:
    @error("disk")
    Disk
    @error("net")
    Net

struct P:
    x: int
    name: String
    fn get(self) -> int:
        return self.x * 2
    fn find(self, k: int) -> Option[int]:
        return Some(k) if k > 0 else None
    fn shout(self):
        println(self.name)

fn through(o: ref Option[P]) -> Option[int]:
    return o?.x

fn main():
    p: Option[P] = Some(P(3, "pat"))
    q: Option[P] = None
    println(p?.x, q?.x, p?.get(), q?.get())
    println(p?.find(5), p?.find(-1), q?.find(5))
    println(p?.name, through(ref p), through(ref q))
    p?.shout()
    q?.shout()
    println(p?.x)
    e: Option[AnyError] = Some(AnyError.from(Fail.Disk))
    none: Option[AnyError] = None
    println(e?.is[Fail](), none?.is[Fail]())

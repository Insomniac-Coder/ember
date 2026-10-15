#$ test: run-pass
#$ rules: ERR-12, ERR-3, FN-8
#$ profiles: debug, release, shipping
#$ stderr: error: load failed
#$ stderr: caused by: disk 3
#$ exit: 1
# `[ERR-12]` — an `Err` of an `Error` type returned from `main` prints
# `error: <Display>`, then one `caused by: <Display>` line for each `source`
# in its chain, and exits with status 1.

@derive(Error)
@error("disk {code}")
struct Disk:
    code: int

@derive(Error)
enum Load:
    @error("load failed")
    @from
    Io(Disk)

fn disk(n: int) -> Result[int, Disk]:
    return Err(Disk(n))

fn main() -> Result[void, Load]:
    v = disk(3)?
    println(v)

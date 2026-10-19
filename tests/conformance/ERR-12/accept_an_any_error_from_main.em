#$ test: run-pass
#$ rules: ERR-12, ERR-8, ERR-10
#$ profiles: debug, development, release
#$ stderr: error: starting: load failed
#$ stderr: caused by: disk 3
#$ exit: 1
# `[ERR-12]` — an `AnyError` returned from `main` prints its text, context
# messages first, then one `caused by:` line for each cause of its error.

@derive(Error)
@error("disk {code}")
struct Disk:
    code: int

@derive(Error)
enum Load:
    @error("load failed")
    @from
    Io(Disk)

fn load(n: int) -> Result[int, Load]:
    return Err(Load.Io(Disk(n)))

fn main() -> Result[void]:
    v = load(3).context("starting")?
    println(v)

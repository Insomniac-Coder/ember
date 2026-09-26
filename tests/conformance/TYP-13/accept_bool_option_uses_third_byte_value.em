#$ test: run-pass
#$ rules: TYP-13, TYP-1
#$ profiles: debug, release, shipping
#$ stdout: 1 1
#$ stdout: false true none
# bool has only two valid values, leaving byte value 2 for None.
fn show(value: Option[bool]) -> str:
    match value:
        Some(flag):
            return "true" if flag else "false"
        None:
            return "none"

fn main():
    println(mem.size_of[Option[bool]](), mem.size_of[bool]())
    println(show(Some(false)), show(Some(true)), show(None))

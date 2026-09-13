#$ test: run-pass
#$ rules: TYP-15, LT-3, HEAP-1, DRP-6
# Static provenance is a region fact, not an expression-shape exception. It
# survives a local binding, a named static, and a call with no borrowed input.

static NAMED: str = "named"

fn produced(_selector: i32) -> str:
    return "produced"

fn main():
    local: str = "local"
    from_local: Box[str] = Box(local)
    from_static: Box[str] = Box(NAMED)
    from_call: Box[str] = Box(produced(1))
    println(from_local.get())
    println(from_static.get())
    println(from_call.get())
#$ stdout: local
#$ named
#$ produced

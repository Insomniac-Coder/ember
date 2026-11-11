#$ test: run-pass
#$ rules: TYP-15, LT-3, HEAP-1, DRP-6
# Static provenance is a region fact, not an expression-shape exception. It
# survives a local binding, a named static, and a call with no borrowed input.

static NAMED: StrView = "named"

fn produced(_selector: i32) -> StrView:
    return "produced"

fn main():
    local: StrView = "local"
    from_local: owns StrView = owns local
    from_static: owns StrView = owns NAMED
    from_call: owns StrView = owns produced(1)
    println(from_local)
    println(from_static)
    println(from_call)
#$ stdout: local
#$ named
#$ produced

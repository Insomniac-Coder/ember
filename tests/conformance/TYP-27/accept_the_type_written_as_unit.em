#$ test: run-pass
#$ rules: TYP-27, GRM-8
#$ stdout: () Ok(()) [(), ()] true
# `[TYP-27]` (ODR-071) — the type `()` is `void`, whose one value `()` is. A tuple type has
# two or more elements, so the grammar's `"(" ")"` names no other type: an `Array[()]` and
# an `Array[void]` are one type (D-355).

fn nothing() -> ():
    pass

fn ok() -> Result[(), str]:
    return Ok(())

fn main():
    x: () = nothing()
    written: Array[()] = [(), ()]
    named: Array[void] = written
    inferred = [(), ()]
    println(x, ok(), named, named == inferred)

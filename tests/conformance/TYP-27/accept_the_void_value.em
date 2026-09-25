#$ test: run-pass
#$ rules: TYP-27, FN-10, TYP-36
#$ stdout: ok
#$ stdout: Ok(()) Err('negative') Some(()) [(1, ())]
#$ stdout: true 1
# `[TYP-27]` — `()` is the value of type `void`, and `Ok(())` the success
# value of `Result[void, E]`. It may be written anywhere a `void` is wanted,
# held in an `Option`, a tuple or an `Array`, and prints as `()` (`[TYP-36]`).

fn nothing() -> void:
    return ()

fn ok(n: int) -> Result[void, String]:
    if n < 0:
        return Err("negative")
    return Ok(())

fn main():
    _x: void = ()
    nothing()
    match ok(1):
        Ok(_): println("ok")
        Err(e): println(e)
    println(ok(1), ok(-1), Some(()), [(1, ())])
    o: Option[void] = Some(nothing())
    pairs = [(1, ())]
    println(o.is_some(), pairs.len())

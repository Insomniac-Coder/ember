#$ test: run-pass
#$ rules: TYP-23
#$ stdout: Some(5)
#$ 7 Some(1) Ok(1)
## `[TYP-23]` - `x = None` followed by `x = v` with `v: T` gives `x` the type `Option[T]`; `1 + x` takes `x`'s
## type; expected types flow into `Ok(...)`, `Err(...)` and `None`.

fn pick(b: bool) -> Option[int]:
    return None if b else Some(1)

fn main():
    x = None
    v = 5
    x = v
    println(x)
    y: i64 = 6
    z = 1 + y
    r: Result[int, String] = Ok(1)
    println(z + 0, pick(false), r)

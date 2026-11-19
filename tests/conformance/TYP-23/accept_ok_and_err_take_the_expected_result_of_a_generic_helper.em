#$ test: run-pass
#$ rules: TYP-23
#$ stdout: Ok(1)
#$ stdout: Err('no')
#$ stdout: Ok(8)
#$ stdout: Some(10)
#$ stdout: Err('late')
# `[TYP-23]` — expected types flow into `Ok(…)`, `Err(…)` and generic calls: a lambda whose whole body is
# `Ok(v)` or `Err(e)`, given to a helper whose result has an open part (`and_then`'s `Result[U, E]`), takes
# the part that is known (`E`) from the expected result and the open one (`U`) from its value, or from the
# result the call is expected to give (`kept`); one nothing fixes is `E2060` (the rejecting test).

fn main():
    good: Result[int, String] = Ok(1)
    println(good.and_then(owns fn(owns v) => Ok(v)))
    other: Result[int, String] = Ok(2)
    kept: Result[int, String] = other.and_then(owns fn(owns v) => Err(String.from("no")))
    println(kept)
    third: Result[int, String] = Ok(4)
    println(third.and_then(owns fn(owns v) => Ok(v * 2)))
    some: Option[int] = Some(5)
    println(some.and_then(owns fn(owns v) => Some(v * 2)))
    failed: Result[int, String] = Err(String.from("early"))
    println(failed.or_else(owns fn(owns e) => Err(String.from("late"))))

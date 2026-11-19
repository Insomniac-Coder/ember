#$ test: run-pass
#$ rules: TYP-23
#$ stdout: Ok(3)
#$ stdout: Err('no')
#$ stdout: Ok(3)
#$ stdout: Ok(0)
#$ stdout: Some(5)
#$ stdout: Ok(0)
# `[TYP-23]` — expected types flow into `Ok(…)` and `Err(…)`, and a branch of a conditional that cannot type
# itself takes the other branch's type: a lambda given to a helper whose result has an open part (`and_then`'s
# `Result[U, E]`) whose body is a conditional of `Ok(…)` and `Err(…)`, in either order, nested or not, takes `E` from
# the expected result and `U` from the branch that fixes it.

fn main():
    a: Result[int, String] = Ok(3)
    println(a.and_then(owns fn(owns v) => Ok(v) if v > 0 else Err(String("no"))))
    b: Result[int, String] = Ok(0)
    println(b.and_then(owns fn(owns v) => Ok(v) if v > 0 else Err(String("no"))))
    c: Result[int, String] = Ok(3)
    println(c.and_then(owns fn(owns v) => Err(String("first")) if v > 5 else Ok(v)))
    d: Result[int, String] = Ok(1)
    println(d.and_then(owns fn(owns v) => Ok(v + 1) if v > 5 else Err(String("deep")) if v > 2 else Ok(0)))
    opt: Option[int] = Some(5)
    println(opt.and_then(owns fn(owns v) => Some(v) if v > 0 else None))
    failed: Result[int, String] = Err(String("early"))
    println(failed.or_else(owns fn(owns e) => Err(String("late")) if e.len() > 9 else Ok(0)))

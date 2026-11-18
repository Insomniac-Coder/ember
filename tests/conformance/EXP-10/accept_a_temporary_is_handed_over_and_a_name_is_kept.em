#$ test: run-pass
#$ rules: EXP-10
#$ stdout: Some('ann') ann
#$ stdout: Some(1) Some(1)
# `[EXP-10]` — `a` is read as `[GRM-13]` reads a place it does not hand over: a field that is not
# `Copy` comes as a view (`p?.name` is an `Option[ref String]`) and `a` keeps its value; from a
# temporary it is handed over (ODR-107), so the chain owns the field.

struct P:
    name: String
    n: int

fn make() -> Option[P]:
    return Some(P(name="ann", n=1))

fn main():
    held = make()
    view = held?.name
    println(view, held?.name.unwrap())
    taken: Option[String] = make()?.name
    println(taken.map(owns fn(owns s: String) => s.len() - 2), make()?.n)

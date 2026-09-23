#$ test: compile-fail
#$ rules: TYP-5
#$ error[E2020]: expected `Option[Option[i64]]`, found `Option[i64]`
# `[TYP-5]` rule 11 is one level only: an `Option` never becomes an
# `Option[Option[T]]`.

fn main():
    inner: Option[int] = 5
    outer: Option[Option[int]] = inner
    println(outer.is_some())

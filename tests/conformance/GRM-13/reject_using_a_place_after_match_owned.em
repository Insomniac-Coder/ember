#$ test: compile-fail
#$ rules: GRM-13, GRM-15
# `[GRM-15]` — `match owned a:` consumes `a`; it cannot be used afterwards.

fn take(owned s: String) -> int:
    return s.len()

fn main():
    a: Option[String] = Some("abc")
    match owned a:
        Some(s): println(take(s))
        None: pass
    println(a)  #$ error[E3040]

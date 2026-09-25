#$ test: compile-fail
#$ rules: GRM-13, EXP-6
# `[GRM-13]` — a non-`Copy` part of a place that is not consumed is bound by
# reference, and the place still owns it: moving it out is `E3013`
# (`[EXP-6]`). `match owned a:` is how to take it.

fn take(owned s: String) -> int:
    return s.len()

fn main():
    a: Option[String] = Some("abc")
    match a:
        Some(s): println(take(s))  #$ error[E3013]: cannot move out of a reference
        None: pass

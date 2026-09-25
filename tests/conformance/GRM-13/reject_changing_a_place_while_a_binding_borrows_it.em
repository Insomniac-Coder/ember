#$ test: compile-fail
#$ rules: GRM-13, BRW-1
# `[GRM-13]` — a reference binding borrows the matched place for as long as it
# is used, so the place cannot be assigned while the binding is live.

fn main():
    a: Option[String] = Some("abc")
    match a:
        Some(s):
            a = None  #$ error[E3021]
            println(s)
        None: pass

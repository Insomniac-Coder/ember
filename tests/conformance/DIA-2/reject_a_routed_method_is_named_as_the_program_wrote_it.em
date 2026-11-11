#$ test: compile-fail
#$ rules: DIA-2, ERR-4, FN-2a
# `[DIA-2]` — a refusal names what was refused in the program's own names: `x.map(f)` on an
# `Option` is routed to a library function (`[ERR-4]`), and the message names `map`, as written,
# and quotes no first line the program never wrote.

fn main():
    name: Option[String] = Some(String("ann"))
    upper = name.map(fn(s: String) -> String => s.to_upper())  #$ error[E3028]: `map` takes `fn(s: String) -> String => s.to_upper()` over
    println(upper)

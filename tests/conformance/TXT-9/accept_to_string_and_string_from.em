#$ test: run-pass
#$ rules: TXT-9, STD-9
#$ stdout:
#$ 42 2.5 true c P(x=1)
#$ lit Ada ab
# `[TXT-9]` — a `str` value never converts implicitly; `s.to_string()` or
# `String.from(s)` makes a `String`. `x.to_string()` on any value with text is
# `f"{x}"`: its `Display` text, else its `Debug` text (`[STD-9]`).

struct P:
    x: int

n = 42
f = 2.5
println(n.to_string(), f.to_string(), true.to_string(), 'c'.to_string(), P(x=1).to_string())
name: String = "Ada"
copy = String.from("lit")
dup = name.to_string()
view = "ab"
made: String = view.to_string()
println(copy, dup, made)

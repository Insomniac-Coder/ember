#$ test: run-pass
#$ rules: STD-8b
#$ stdout: true true false
#$ true
## `[STD-8b]` — `StrView` implements `Contains[char]` and `Contains[StrView]` only; matching happens
## at character boundaries. `x not in c` is `not c.contains(x)`.

fn main():
    t = "héllo"
    println('é' in t, "ll" in t, 'z' in t)
    println('z' not in t)

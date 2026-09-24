#$ test: run-pass
#$ rules: TXT-10
#$ stdout:
#$ [name] [ = ] [Ada]
#$ [no separator] [] []
#$ Some(('k', 'v=w')) None
#$ x 1
# `[TXT-10]` — `partition(sep)` is Python's: the text before the first `sep`,
# `sep`, and the rest, or `(s, "", "")`; `split_once(sep)` is the two sides,
# or `None`. Every part is a view of the text.

key, sep, value = "name = Ada".partition(" = ")
println(f"[{key}] [{sep}] [{value}]")
a, b, c = "no separator".partition("=")
println(f"[{a}] [{b}] [{c}]")
println("k=v=w".split_once("="), "kv".split_once("="))
match "x:1".split_once(":"):
    Some((left, right)):
        println(left, right)
    None:
        println("none")

#$ test: run-pass
#$ rules: STD-15
#$ stdout: 1, 2, 3 a-b xy []
# `[STD-15]` — `xs.join(sep)` is Python's `sep.join(xs)` for elements that
# are `Display`: their text, with `sep` between each two.

xs = [1, 2, 3]
names = ["a", "b"]
owned_ = [String.from("x"), String.from("y")]
empty: Array[int] = []
none = empty.join(",")
println(xs.join(", "), names.join("-"), owned_.join(""), f"[{none}]")

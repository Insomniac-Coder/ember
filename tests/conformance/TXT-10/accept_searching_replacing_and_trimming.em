#$ test: run-pass
#$ rules: TXT-10, TXT-11
#$ stdout:
#$ [héllo, wörld] [héllo, wörld  ] [  héllo, wörld]
#$ true true Some(3) Some(12) None
#$ 2 4 1
#$ a+b+c -a-b- xyzxyzxyz []
#$ Some(2) Ada
# `[TXT-10]` — `trim`, `trim_start` and `trim_end` drop Unicode white space
# and return a view of the text; `starts_with`, `ends_with`, `find` and
# `rfind` search by bytes over valid UTF-8 (`find` gives a byte offset);
# `count` counts non-overlapping matches and `replace` replaces them all, with
# Python's meaning for an empty needle; `repeat` of a count below one is
# empty. A `String` has them through its `str` (`[TXT-11]`).

s = "  héllo, wörld  "
t = s.trim()
println(f"[{t}] [{s.trim_start()}] [{s.trim_end()}]")
println(t.starts_with("hé"), t.ends_with("ld"), t.find("l"), t.rfind("l"), t.find("zz"))
println("banana".count("an"), "abc".count(""), "aaa".count("aa"))
empty = "q".repeat(-2)
println("a-b-c".replace("-", "+"), "ab".replace("", "-"), "xyz".repeat(3), f"[{empty}]")
name: String = " Ada "
println(name.find("d"), name.trim())

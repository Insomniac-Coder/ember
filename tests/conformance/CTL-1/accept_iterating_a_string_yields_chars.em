#$ test: run-pass
#$ rules: CTL-1, CTL-2
#$ stdout:
#$ h|é|l|l|o|,| |世|界|
#$ b 2
#$ 3
# `[CTL-1]` — a `str` yields its `char`s, decoded from UTF-8; a `String`
# yields the same through its `str`. `continue` moves to the next character.

for c in "héllo, 世界":
    print(c, end="|")
println()
s = "ab"
k = 0
for c in s:
    k += 1
    if c == 'a':
        continue
    println(c, k)
name: String = "abc"
count = 0
for _ in name:
    count += 1
println(count)

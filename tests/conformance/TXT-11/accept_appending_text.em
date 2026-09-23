#$ test: run-pass
#$ rules: TXT-11
#$ profiles: debug, release, shipping
#$ stdout: abc
#$ abcdef
#$ x!
# D-195 — `s += t` appends text to a `String`, and `a + b` extends the `String`
# on the left, consuming it.

fn main():
    out: String = "a"
    out += "b"
    more: String = "c"
    out += more
    println(out)
    out += f"{"d"}ef"
    println(out)
    shout: String = "x"
    loud = shout + "!"
    println(loud)

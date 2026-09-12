#$ test: run-pass
#$ rules: SPN-1, BRW-2
# The implicit spelling is the same borrowed view as `s.as_str()`: it points
# into `s`, performs no allocation or copy, and remains usable while `s` is
# unchanged.

fn show(text: str):
    println(text)

fn main():
    s: String = String()
    s.push_str("ember")
    show(s)
    # The temporary call borrow ends at the call's last use.
    s.push_str("!")
    view: str = s
    show(view)
#$ stdout: ember
#$ ember!

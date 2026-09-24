#$ test: run-pass
#$ rules: LEX-19, TYP-39
#$ profiles: debug, release, shipping
#$ stdout: [    [1, 2]] [[1, 2]   |] [[1, ] [[1, 2]]
#$ [ Some('hi') ]
# `[LEX-19]` follows Python: `!r` or `!s` makes a collection text, and the
# spec then pads or cuts that text.

fn main():
    xs = [1, 2]
    println(f"[{xs!r:>10}]", f"[{xs!s:<9}|]", f"[{xs!r:.4}]", f"[{xs}]")
    o: Option[str] = Some("hi")
    println(f"[{o!r:^12}]")

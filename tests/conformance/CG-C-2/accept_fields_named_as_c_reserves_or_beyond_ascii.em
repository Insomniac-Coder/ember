#$ test: run-pass
#$ rules: CG-C-2, MNG-3
#$ stdout: 4 x true
#$ stdout: Settings(default=1, inline='x', errno=3)
#$ stdout: 4
#$ stdout: 6 r
#$ stdout: x 7
## `[CG-C-2]` (D-775) — a field or variant named as C reserves (a keyword, or a macro of the headers
## the C includes) still compiles, and so does one beyond ASCII (`[MNG-3]`); what the program prints
## names each as written.

@derive(Debug, Eq, Clone)
struct Settings:
    default: int
    inline: String
    errno: int

enum Mode:
    auto
    switch(int)

class Node:
    volatile: int
    register: String

struct Größe:
    maß: int

fn main():
    s = Settings(1, String.from("x"), 3)
    t = s.clone()
    println(s.default + s.errno, s.inline, s == t)
    println(f"{s!r}")
    match Mode.switch(4):
        Mode.switch(n):
            println(n)
        Mode.auto:
            println("auto")
    n = Node(5, String.from("r"))
    n.volatile += 1
    println(n.volatile, n.register)
    o: Option[Settings] = Some(t)
    if Some(v) = o:
        println(v.inline, Größe(7).maß)

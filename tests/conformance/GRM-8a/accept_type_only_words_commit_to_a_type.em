#$ test: run-pass
#$ rules: GRM-8a, GRM-8
#$ stdout: true
#$ stdout: 0
#$ stdout: true
#$ stdout: true
#$ stdout: true
## `[GRM-8a]` — inside `[ ]` in expression position the parser commits to a type argument when the next
## token is `ref`, `*`, `fn` or `void` (among the type-only words), so what follows is read as a type
## where an expression would not parse (`fn() -> int`, `ref int`, `*int`).

fn main():
    println(mem.size_of[ref int]() == mem.size_of[*int]())
    println(mem.size_of[void]())
    f = mem.size_of[fn() -> int]()
    println(f == f)
    more()

struct Pt:
    x: int = 1
    y: int = 2

fn more():
    println(mem.size_of[owns Pt]() == mem.size_of[*Pt]())
    g = mem.size_of[once fn() -> int]()
    println(g == g)

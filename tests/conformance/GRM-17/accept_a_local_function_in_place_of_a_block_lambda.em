#$ test: run-pass
#$ rules: GRM-17, GRM-28
#$ stdout: 5
#$ stdout: 7
## `[GRM-17]` — the lambda's one-statement body on its own line (`h = fn(x): total += x` assigns the
## lambda to `h`), and the local function that replaces a block (`[GRM-28]`): declared on the line
## above with the block below it, then `h = add`.

fn main():
    total = 0
    h = fn(x: int): total += x
    h(2)
    h(3)
    println(total)
    fn add(rhs: int):
        y = rhs * 2
        total += y
    g = add
    g(1)
    println(total)

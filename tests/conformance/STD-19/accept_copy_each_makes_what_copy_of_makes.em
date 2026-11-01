#$ test: run-pass
#$ rules: STD-19, OWN-8
#$ stdout: 6
#$ stdout: a b
#$ stdout: ab
## `[STD-19]` — `copy_each()` over an iterator of views gives, for each item, what `copy of` the item
## makes: a number copied, a `String` of the program's own, kept past changes to what it came from.

fn main():
    xs = [1, 2, 3]
    total = xs.iter().copy_each().sum()
    println(total)
    m: Map[String, int] = {}
    m.insert(owns String("b"), owns 2)
    m.insert(owns String("a"), owns 1)
    names = sorted(m.keys().copy_each())
    m.clear()
    println(names[0], names[1])
    joined = String()
    for name in names.iter().copy_each():
        joined.push_str(name)
    println(joined)

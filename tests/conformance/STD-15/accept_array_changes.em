#$ test: run-pass
#$ rules: STD-15, TYP-37
#$ profiles: debug, release, shipping
#$ stdout: 1 3 3 5 7 9
#$ 9 7 5 3 3 1
#$ 1 4 7 3
#$ 42 9 5 3 3 99
#$ 0 -1
#$ apple fig pear
#$ -1.0 -0.0 0.0 2.5
# `sort` (stable, in `Ord`'s order: totalOrder for floats, bytes for text),
# `reverse`, `sorted`, `pop`, `remove`, `insert` and `clear`.

fn line(xs: Span[int]) -> String:
    out: String = ""
    for i, x in enumerate(xs):
        out += f"{x}" if i == 0 else f" {x}"
    return out

fn main():
    xs = [5, 3, 9, 1, 3, 7]
    xs.sort()
    println(line(xs))
    xs.reverse()
    ys = xs.sorted()
    println(line(xs))
    top = xs.pop()
    gone = xs.remove(1)
    println(top.unwrap(), len(xs), gone, len(xs) - 1)
    xs.insert(0, 42)
    xs.insert(len(xs), 99)
    println(line(xs))
    xs.clear()
    println(len(xs), xs.pop().unwrap_or(-1))
    words: Array[String] = ["pear", "apple", "fig"]
    words.sort()
    println(words[0], words[1], words[2])
    fs = [2.5, -1.0, 0.0, -0.0]
    fs.sort()
    println(fs[0], fs[1], fs[2], fs[3])
    _ = ys

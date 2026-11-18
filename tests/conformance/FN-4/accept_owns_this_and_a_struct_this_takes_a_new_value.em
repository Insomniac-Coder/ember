#$ test: run-pass
#$ rules: FN-4
#$ stdout: 5 0
#$ stdout: 9
# `[FN-4]` — `owns this` after the parameters takes the object over (`fn finish() owns this ->
# u64:`); a struct's method may give `this` a whole new value (`this = Point(x=0.0, y=0.0)`).

struct Point:
    x: f64
    y: f64

    fn reset():
        this = Point(x=0.0, y=0.0)

struct Counter:
    n: u64

    fn finish() owns this -> u64:
        return this.n

fn main():
    p = Point(x=5.0, y=2.0)
    println(p.x as int, 0)
    p.reset()
    c = Counter(n=9)
    println(c.finish())

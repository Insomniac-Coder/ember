#$ test: run-pass
#$ rules: CLO-4, BRW-2
# And the other half: once the closure's last use has passed, the borrow it
# held is over and the captured container is the owner's again. `[BRW-2]` —
# scope end is irrelevant, the last use is what matters.

fn apply(f: fn(i32) -> i32, v: i32) -> i32:
    return f(v)

fn main():
    m: Array[i32] = Array[i32]()
    m.push(1)
    println(apply(fn(x) => x + m[0], 10))
    m.push(2)
    println(m[1])
#$ stdout: 11
#$ 2

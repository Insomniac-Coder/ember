# A12: closures made in a loop, each capturing the loop variable.
fn main():
    fs: Array[fn() -> int] = []
    for i in 0..3:
        fs.push(fn() => i)
    total = 0
    for f in fs.iter_mut():
        total += f()
    println(total)

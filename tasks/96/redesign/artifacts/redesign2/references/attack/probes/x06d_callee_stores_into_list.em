# Attack on A6's "a container of views is a view": a callee stores a view of its own local into the
# caller's list. Today's [TYP-15] already tracks "this call stores its argument" across calls.
fn put(mut v: Array[str], s: str):
    v.push(s)

fn fill(mut v: Array[str]):
    word: String = "short lived"
    put(v, word)

fn main():
    vs: Array[str] = []
    fill(vs)
    println(vs[0])

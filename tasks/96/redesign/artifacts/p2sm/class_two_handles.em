class Pool:
    dense: Array[int] = []

fn main():
    p = Pool()
    q = p
    p.dense.push(1)
    for x in p.dense:
        q.dense.push(x)
    println(p.dense.len())

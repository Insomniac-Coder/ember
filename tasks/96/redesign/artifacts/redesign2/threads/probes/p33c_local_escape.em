# A33: a reference from inside the lock kept in a local of the same function past the block.
from std.sync import Mutex

fn main():
    m: Mutex[Array[String]] = Mutex(["ann".to_string()])
    r: ref String = ref m.lock()[0]
    with g = m.lock():
        g.clear()
    println(r)

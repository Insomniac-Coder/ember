# A9 follow-up: a reference returned from inside a RefCell guard, then the cell is changed.

fn first(c: RefCell[Array[String]]) -> ref String:
    with g = c.borrow():
        return ref g[0]

fn clear(c: RefCell[Array[String]]):
    with g = c.borrow_mut():
        g.clear()

fn main():
    c: RefCell[Array[String]] = RefCell(["ann".to_string()])
    r = first(c)
    clear(c)
    println(r)

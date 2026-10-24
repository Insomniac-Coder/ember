# Attack on A23's "move at overwrite" when the overwritten value has its own `drop`. Python releases
# the old guard at `g = other` (it prints "released ann" before "ann"). Keeping g.name alive for `old`
# means either moving a field out of a type with `drop` (E3010 today) or delaying the whole old
# guard's drop to old's last use, which moves the "released" line.
struct Guard:
    name: String

    fn drop(mut self):
        println(f"released {self.name}")

fn swap_guard(mut g: Guard, owned other: Guard):
    old = g.name
    g = other
    println(old)

fn main():
    g = Guard(name="ann")
    swap_guard(g, Guard(name="bob"))

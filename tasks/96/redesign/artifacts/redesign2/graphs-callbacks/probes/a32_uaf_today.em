# Found in passing: a `ref mut` into a class's handler list held across a call that grows the list.
# `ember check` accepts it; the emitted C (probes/a32_emit*) holds a raw pointer into the old buffer
# across the call and has no access check, so the closure reads its captured String after the buffer
# was moved by `push` (a use after free, by reading the emitted C; not run).
class Ui:
    handlers: Array[fn(Ui)] = []
    clicks: int = 0

    fn fire(self):
        i = 0
        while i < self.handlers.len():
            h = self.handlers.get_mut(i).unwrap()
            h(self)
            i += 1

fn count(ui: Ui):
    ui.clicks += 1

fn main():
    ui = Ui()
    label = "first handler".to_string()
    grow = owned fn(u: Ui):
        for _ in 0..100:
            u.handlers.push(count)
        println(label)
    ui.handlers.push(grow)
    ui.fire()
    println(ui.clicks)

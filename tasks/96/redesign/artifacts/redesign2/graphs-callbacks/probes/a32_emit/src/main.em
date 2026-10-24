# A32: the same dispatch written by index, re-reading the length each step (Python's list iterator).
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

fn add_more(ui: Ui):
    ui.handlers.push(count)

fn main():
    ui = Ui()
    ui.handlers.push(count)
    ui.handlers.push(add_more)
    ui.fire()
    println(ui.clicks)

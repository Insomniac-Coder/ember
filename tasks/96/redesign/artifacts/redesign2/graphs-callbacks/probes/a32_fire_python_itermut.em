# A32: phase 1's spelling, with the `.iter_mut()` today's E3021 help asks for.
class Ui:
    handlers: Array[fn(Ui)] = []
    clicks: int = 0

    fn fire(self):
        for h in self.handlers.iter_mut():
            h(self)

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

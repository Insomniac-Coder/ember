# A32: handlers receive the UI that holds them; one adds another handler during the dispatch.
class Ui:
    handlers: Array[fn(Ui)] = []
    clicks: int = 0

    fn fire(self):
        for h in self.handlers:
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

# A32: the struct version (a value, not a shared object).
struct Ui:
    handlers: Array[fn(mut Ui)]
    clicks: int

    fn fire(mut self):
        for h in self.handlers:
            h(self)

fn count(mut ui: Ui):
    ui.clicks += 1

fn main():
    ui = Ui(handlers=[], clicks=0)
    ui.handlers.push(count)
    ui.fire()
    println(ui.clicks)

# A32: a UI whose handlers get the UI itself (to count clicks, add more handlers).

class Ui:
    handlers: Array[fn(Ui)] = []
    clicks: int = 0

    fn fire(self):
        for h in self.handlers.iter_mut():
            h(self)

fn count(ui: Ui):
    ui.clicks += 1

# Compiles; at run time this handler's push is a write access to Ui.handlers while `fire`
# holds one: an exclusivity panic ([EXC-1]).
fn add_more(ui: Ui):
    ui.handlers.push(count)

struct UiS:
    handlers: Array[fn(mut UiS)]
    clicks: int

    fn fire(mut self):
        for h in self.handlers.iter_mut():
            h(self)

fn main():
    ui = Ui()
    ui.handlers.push(count)
    ui.fire()
    println(ui.clicks)

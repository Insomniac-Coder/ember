# A32: a one-shot handler removes itself from inside its own call.
class Bus:
    handlers: Array[fn(Bus, int)] = []
    current: int = 0
    seen: int = 0

    fn emit(self, v: int):
        self.current = 0
        while self.current < self.handlers.len():
            h = self.handlers.get_mut(self.current).unwrap()
            h(self, v)
            self.current += 1

    fn remove_current(self):
        _h = self.handlers.remove(self.current)
        self.current -= 1

fn once(b: Bus, v: int):
    b.seen += v
    b.remove_current()

fn main():
    b = Bus()
    b.handlers.push(once)
    b.emit(5)
    b.emit(7)
    println(b.seen)

# Found in passing: a stateful handler removes itself during its own call. `ember check` accepts it.
# The emitted C (same trampoline as a32_emit_uaf/c/release/c/main.c lines 126-132) drops the handler's
# state inside the call, then copies the state back into the slot after the call returns.
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

fn main():
    b = Bus()
    tag = "once".to_string()
    once = owned fn(bus: Bus, v: int):
        bus.seen += v + tag.len()
        bus.remove_current()
    b.handlers.push(once)
    b.handlers.push(owned fn(bus: Bus, v: int): bus.seen += 1)
    b.emit(5)
    b.emit(7)
    println(b.seen)

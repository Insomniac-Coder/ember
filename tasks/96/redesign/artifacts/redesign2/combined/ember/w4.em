# W4 in the combined model's proposed syntax (phase3b-combined.md 6.4). Checked with today's
# `ember check` only to record what today's compiler says.
class Dispatcher:
    handlers: Array[fn(Dispatcher, int, int) -> int] = []
    current: int = 0

    fn register(self, h: fn(Dispatcher, int, int) -> int):
        self.handlers.push(h)

    fn unregister_current(self):
        self.handlers.remove(self.current)

    fn dispatch(self, k: int):
        self.current = k % len(self.handlers)
        h = self.handlers[self.current]
        h(self, k, k % 1000)

fn main():
    total = 0
    fn make_handler(m: int) -> fn(Dispatcher, int, int) -> int:
        counter = 0
        fn handler(d: Dispatcher, k: int, v: int) -> int:
            if v >= 0:
                counter += v * m
                total += 1
                if k % 100_000 == 99_999:
                    d.unregister_current()
                    d.register(make_handler(k % 17 + 1))
            return counter
        return handler
    d = Dispatcher()
    for i in 0..10_000:
        d.register(make_handler(i % 13 + 1))
    for k in 0..10_000_000:
        d.dispatch(k)
    s = 0
    for h in d.handlers:
        s += h(d, 0, -1)
    println(total + s)

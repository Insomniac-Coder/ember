# W4 with a log callback, in Python: graphs-callbacks/python/w4.py with `d.log(k)` called by every
# handler before it uses its state. Every C and Rust model of it must print the same checksum.
# Run: python -I w4log.py
class Dispatcher:
    def __init__(self, log):
        self.handlers = []
        self.current = 0
        self.log = log
    def register(self, h):
        self.handlers.append(h)
    def unregister_current(self):
        del self.handlers[self.current]
    def dispatch(self, k):
        self.current = k % len(self.handlers)
        self.handlers[self.current](self, k, k % 1000)

def main():
    total = 0
    logged = 0
    def log(k):
        nonlocal logged
        logged += k & 1
    def make_handler(m):
        counter = 0
        def handler(d, k, v):
            nonlocal counter, total
            if v >= 0:
                d.log(k)
                counter += v * m
                total += 1
                if k % 100_000 == 99_999:
                    d.unregister_current()
                    d.register(make_handler(k % 17 + 1))
            return counter
        return handler
    d = Dispatcher(log)
    for i in range(10_000):
        d.register(make_handler(i % 13 + 1))
    for k in range(10_000_000):
        d.dispatch(k)
    s = 0
    for h in list(d.handlers):
        s += h(d, 0, -1)
    print(total + s + logged)

main()

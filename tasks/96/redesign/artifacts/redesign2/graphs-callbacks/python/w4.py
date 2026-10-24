# W4 written in Python, to check that the workload the C models time is this program: it must print
# the same checksum as every model (34946993000). Run: python -I w4.py
class Dispatcher:
    def __init__(self):
        self.handlers = []
        self.current = 0
    def register(self, h):
        self.handlers.append(h)
    def unregister_current(self):
        del self.handlers[self.current]
    def dispatch(self, k):
        self.current = k % len(self.handlers)
        self.handlers[self.current](self, k, k % 1000)

def main():
    total = 0
    def make_handler(m):
        counter = 0
        def handler(d, k, v):
            nonlocal counter, total
            if v >= 0:
                counter += v * m
                total += 1
                if k % 100_000 == 99_999:
                    d.unregister_current()
                    d.register(make_handler(k % 17 + 1))
            return counter
        return handler
    d = Dispatcher()
    for i in range(10_000):
        d.register(make_handler(i % 13 + 1))
    for k in range(10_000_000):
        d.dispatch(k)
    s = 0
    for h in list(d.handlers):
        s += h(d, 0, -1)
    print(total + s)

main()

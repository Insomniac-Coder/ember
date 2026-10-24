# py_checks.py: what Python does with the programs in the threads report (run: python -I py_checks.py).
import threading, asyncio, ctypes
from concurrent.futures import ThreadPoolExecutor

# A15: total += from threads under a lock: the sum, whatever the order.
total = 0; lock = threading.Lock()
def work(t):
    global total
    with lock:
        total += len(t.split())
texts = ["a b", "c", "d e f"]
ts = [threading.Thread(target=work, args=(t,)) for t in texts]
for t in ts: t.start()
for t in ts: t.join()
assert total == 6; print("A15 lock total:", total)

# A15: results.append from threads: every value arrives; the order is not promised by Python.
results = []
ts = [threading.Thread(target=lambda i=i: results.append(i * i)) for i in range(4)]
for t in ts: t.start()
for t in ts: t.join()
assert sorted(results) == [0, 1, 4, 9]; print("A15 appended (this run's order):", results)

# A19: a module global is one variable for every thread (not one per thread).
FRAME = 41
seen = []
t = threading.Thread(target=lambda: seen.append(FRAME)); t.start(); t.join()
assert seen == [41]; print("A19 a worker thread sees the main thread's FRAME:", seen[0])

# A19: a worker's write is seen by the main thread at once, and the worker reads its own write.
LOG = []
def worker():
    LOG.append("w"); seen.append(len(LOG))
t = threading.Thread(target=worker); t.start(); t.join()
assert LOG == ["w"] and seen[-1] == 1; print("A19 worker reads its own append:", seen[-1])

# A33: a name returned out of a lock stays usable after the list is cleared (strings are immutable).
scores = ["ann"]; slock = threading.Lock()
def top():
    with slock:
        return scores[0]
def clear():
    with slock:
        scores.clear()
r = top(); clear()
assert r == "ann"; print("A33 name after clear:", r)

# A33: a list returned out of a lock is the shared list: changing it outside the lock changes the guarded one.
board = {"names": ["ann"]}
def names():
    with slock:
        return board["names"]
n = names(); n.append("bob")
assert board["names"] == ["ann", "bob"]; print("A33 the returned list is the guarded one:", board["names"])

# A14: a generator walking its own local list across yield.
def lines():
    buf = ["a", "b"]
    for s in buf:
        yield s
assert list(lines()) == ["a", "b"]; print("A14 generator over its own local list:", list(lines()))

# A14: asyncio with a task group (Python 3.11+): tasks borrow nothing, they share objects.
async def fetch():
    await asyncio.sleep(0); return 1
async def serve(cache, out):
    n = await fetch(); out.append(len(cache) + n)
async def main():
    out = []; cache = [1, 2, 3]
    async with asyncio.TaskGroup() as tg:
        tg.create_task(serve(cache, out)); tg.create_task(serve(cache, out))
    return out
assert asyncio.run(main()) == [4, 4]; print("A14 task group results:", [4, 4])

# A20: a ctypes callback that captures an object (no C library needed: ctypes calls it directly).
class Game: score = 0
game = Game()
CB = ctypes.CFUNCTYPE(None, ctypes.c_void_p)
cb = CB(lambda ud: setattr(game, "score", game.score + 1))
for _ in range(3): cb(None)
assert game.score == 3; print("A20 callback ran three times, score:", game.score)

print("all Python checks passed")

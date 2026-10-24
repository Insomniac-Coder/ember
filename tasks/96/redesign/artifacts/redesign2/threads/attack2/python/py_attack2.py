# py_attack2.py: what Python does with the programs of the second attack on the threads group's answers.
# Run: python -I py_attack2.py   (every behaviour quoted in phase3b-threads-attack.md is an assert here)
import asyncio
import threading
from concurrent.futures import ThreadPoolExecutor
from contextlib import contextmanager

out = []

# 1 (y14): the parent holds the lock; threads started inside append to the guarded list without it.
class Stats:
    def __init__(self):
        self.lock = threading.Lock(); self.words = 0; self.tags = []
s = Stats()
with s.lock:
    ts = [threading.Thread(target=lambda w=w: s.tags.append(w)) for w in ("#a", "#b")]
    for t in ts: t.start()
    for t in ts: t.join()
assert sorted(s.tags) == ["#a", "#b"]
out.append(f"lock_held_by_parent_threads_append: {sorted(s.tags)}")

# 2 (y15): a generator consumed by a generator; the outer consumer clears the list in between.
class Door:
    def __init__(self): self.log = ["open", "shut"]
    def replay(self):
        for entry in self.log:
            yield len(entry)
def lengths(door):
    for n in door.replay():
        yield n
d = Door(); got = []
for n in lengths(d):
    d.log.clear()
    got.append(n)
assert got == [4]
out.append(f"nested_generator_list_cleared: {got} (stops, no crash)")

# 3 (y16/x06): a callback run during a call grows the list the caller is walking.
LOG = ["start"]
def on_key():
    if len(LOG) < 4: LOG.append(f"key {len(LOG)}")
def poll_events(): on_key()
seen = []
for line in LOG:
    poll_events()
    seen.append(line)
assert seen == ["start", "key 1", "key 2", "key 3"]
out.append(f"reentrant_callback_grows_walked_list: {seen}")

# 4 (y19): two locks, each guarding its own fields.
class Server:
    def __init__(self):
        self.stats_lock = threading.Lock(); self.hits = 0
        self.cache_lock = threading.Lock(); self.cache = {}
    def hit(self):
        with self.stats_lock: self.hits += 1
    def remember(self, k, v):
        with self.cache_lock: self.cache[k] = v
sv = Server()
with ThreadPoolExecutor(4) as ex:
    for i in range(100):
        ex.submit(sv.hit); ex.submit(sv.remember, str(i % 10), str(i))
assert sv.hits == 100 and len(sv.cache) == 10
out.append(f"two_locks: hits={sv.hits} cache={len(sv.cache)}")

# 5 (y20): a re-entrant lock; a plain Lock is not re-entrant.
class Account:
    def __init__(self): self.lock = threading.RLock(); self.balance = 0
    def deposit(self, n):
        with self.lock: self.balance += n
    def deposit_all(self, amounts):
        with self.lock:
            for n in amounts: self.deposit(n)
a = Account(); a.deposit_all([1, 2, 3])
plain = threading.Lock(); plain.acquire()
again = plain.acquire(blocking=False); plain.release()
assert a.balance == 6 and again is False
out.append(f"rlock_helper: balance={a.balance}; plain Lock taken twice: {again}")

# 6 (y21): a dict of counts and a histogram from threads, under a lock.
texts = ["a b a", "b c", "a"]
counts = {}; clock = threading.Lock()
def count(t):
    for w in t.split():
        with clock: counts[w] = counts.get(w, 0) + 1
with ThreadPoolExecutor(3) as ex: list(ex.map(count, texts))
assert counts == {"a": 3, "b": 2, "c": 1}
data = bytes(range(256)) * 4
hist = [0] * 256; hlock = threading.Lock()
def hpart(chunk):
    for b in chunk:
        with hlock: hist[b] += 1
with ThreadPoolExecutor(4) as ex: list(ex.map(hpart, [data[i::4] for i in range(4)]))
assert hist == [4] * 256
out.append(f"dict_reduction: {dict(sorted(counts.items()))}; histogram ok")

# 7 (y22): each thread re-targets its own players.
class Enemy:
    def __init__(self, x): self.x = x
class Player:
    def __init__(self, x): self.x = x; self.target = None
enemies = [Enemy(float(i * 10)) for i in range(5)]
players = [Player(float(i)) for i in range(40)]
def aim(p): p.target = min(enemies, key=lambda e: abs(e.x - p.x))
with ThreadPoolExecutor(4) as ex: list(ex.map(aim, players))
assert players[0].target is enemies[0] and players[39].target is enemies[4]
out.append("parallel_retarget: every player has its nearest enemy")

# 8 (y23): asyncio tasks change shared state between awaits, no lock.
async def crawl(url, seen, res):
    await asyncio.sleep(0)
    if url not in seen:
        seen.add(url)
        if len(seen) <= 3: res.append(url)
async def amain():
    seen, res = set(), []
    async with asyncio.TaskGroup() as tg:
        for u in ["a", "b", "a", "c", "d"]:
            tg.create_task(crawl(u, seen, res))
    return res
res = asyncio.run(amain())
assert res == ["a", "b", "c"]
out.append(f"asyncio_shared_state: {res}")

# 9 (y24): a module lock and a module list, appended to by worker threads.
RESULTS_LOCK = threading.Lock(); RESULTS = []
def worker(x):
    r = x * x
    with RESULTS_LOCK: RESULTS.append(r)
hs = [threading.Thread(target=worker, args=(i,)) for i in range(4)]
for h in hs: h.start()
for h in hs: h.join()
assert sorted(RESULTS) == [0, 1, 4, 9]
out.append(f"module_lock_worker: {sorted(RESULTS)}")

# 10 (section on waiting): a condition variable; wait() releases the lock while it sleeps.
cond = threading.Condition(); items = []
def consumer(box):
    with cond:
        while not items: cond.wait()
        box.append(items.pop(0))
box = []
c = threading.Thread(target=consumer, args=(box,)); c.start()
with cond:
    items.append("job"); cond.notify()
c.join()
assert box == ["job"]
out.append(f"condition_wait: {box}")

for line in out: print(line)
print("all Python checks passed")

# py_attack.py: what Python does with each program the attack report uses. Run: python -I py_attack.py
import threading, time
from concurrent.futures import ThreadPoolExecutor

out = {}

# 1. init escape: a thread started in __init__ reads a field that __init__ sets afterwards.
class Worker:
    def __init__(self, name):
        self.name = ""
        seen = []
        t = threading.Thread(target=lambda: seen.append(self.name)); t.start()
        self.name = name
        t.join()
        self.seen = seen[0]
w = Worker("bob")
assert w.seen in ("", "bob")
out["init_escape_reads"] = "'' or 'bob' (either; no crash)"

# 2. configure, then share with a thread.
class Config:
    def __init__(self): self.scale = 2
cfg = Config(); cfg.scale = 3
res = []
t = threading.Thread(target=lambda: res.append(cfg.scale)); t.start(); t.join()
assert res == [3]
out["configure_then_share"] = res[0]

# 3. a helper called with the lock held.
class Stats:
    def __init__(self): self.lock = threading.Lock(); self.words = 0
    def _bump(self): self.words += 1          # touches the field with no `with` of its own
    def add(self):
        with self.lock:
            self._bump()
s = Stats()
ts = [threading.Thread(target=s.add) for _ in range(8)]
for t in ts: t.start()
for t in ts: t.join()
assert s.words == 8
out["helper_under_lock"] = s.words

# 4. a worker thread reads a module-level constant.
DEBUG = True
res = []
t = threading.Thread(target=lambda: res.append(DEBUG)); t.start(); t.join()
assert res == [True]
out["worker_reads_module_constant"] = res[0]

# 5. a generator walks its own local dict across yield.
def keys_of():
    d = {1: 10, 2: 20}
    for k in d:
        yield k
assert list(keys_of()) == [1, 2]
out["generator_over_local_dict"] = list(keys_of())

# 6. a parallel map makes result objects that point at the input objects.
class Player:
    def __init__(self, name): self.name = name; self.hp = 10
class Match:
    def __init__(self, player, score): self.player = player; self.score = score
players = [Player(f"p{i}") for i in range(4)]
with ThreadPoolExecutor(4) as ex:
    matches = list(ex.map(lambda p: Match(p, len(p.name)), players))
assert all(m.player is p for m, p in zip(matches, players))
out["results_point_at_inputs"] = [m.player.name for m in matches]

# 7. a handle taken out of a lock-guarded field, used after the lock.
class Board:
    def __init__(self): self.lock = threading.Lock(); self.best = Player("ann")
b = Board()
with b.lock:
    best = b.best
with b.lock:
    b.best = Player("bob")
assert best.name == "ann"
out["handle_out_of_lock_field"] = best.name

# 8. module-level initialisers run when the module is imported, in order, not on first use.
order = []
def stamp(tag): order.append(tag); return len(order)
A = stamp("A")
B = stamp("B")
def main(): order.append("main")
main()
assert order == ["A", "B", "main"]
out["module_init_order"] = order

# 9. a helper that adds to a module-level total, called from worker threads (with a lock).
WORDS = 0
words_lock = threading.Lock()
def add_words(t):
    global WORDS
    with words_lock:
        WORDS += len(t.split())
ts = [threading.Thread(target=add_words, args=(x,)) for x in ["a b", "c d e", "f"]]
for t in ts: t.start()
for t in ts: t.join()
assert WORDS == 6
out["helper_adds_to_module_total"] = WORDS

# 10. one object in a list twice; each worker changes "its" element: both changes land (GIL).
p = Player("dup")
lst = [p, Player("x"), p, Player("y")]
def hurt(q): q.hp -= 1
with ThreadPoolExecutor(4) as ex:
    list(ex.map(hurt, lst))
assert p.hp == 8
out["duplicate_in_list_both_changes_land"] = p.hp

for k, v in out.items():
    print(f"{k}: {v}")
print("all Python checks passed")

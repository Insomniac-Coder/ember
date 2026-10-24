# threads_side.py: the Python twin of threads_side.c (problem 2 of phase3b-combined-attack.md).
import weakref, sys
from concurrent.futures import ThreadPoolExecutor

class Player:
    def __init__(self, i): self.id = i; self.hp = 100
class Watcher:
    def __init__(self, t): self.target = t
class Camera:
    def __init__(self): self._t = None
    @property
    def target(self): return self._t() if self._t is not None else None
    @target.setter
    def target(self, p): self._t = weakref.ref(p) if p is not None else None

players = [Player(i) for i in range(8)]
cams = [Camera() for _ in range(64)]
for i, c in enumerate(cams): c.target = players[i % 8]
ws = [Watcher(players[0]) for _ in range(64)]
def aim(i):
    ws[i].target = players[1 + i % 7]
    cams[i].target = players[(i + 3) % 8]
with ThreadPoolExecutor(4) as ex: list(ex.map(aim, range(64)))
# references to each player: the list, the watchers, and getrefcount's own argument
counts = [sys.getrefcount(p) - 1 for p in players]
print("python: references per player (list + watchers):", counts)

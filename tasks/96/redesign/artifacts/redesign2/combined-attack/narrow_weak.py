# narrow_weak.py: the Python twin of narrow_weak.c (problem 1 of phase3b-combined-attack.md).
import weakref

class Player:
    def __init__(self):
        self.hp = 100
        self.inventory = []

class Camera:
    def __init__(self):
        self._target = None
    @property
    def target(self):
        return self._target() if self._target is not None else None
    @target.setter
    def target(self, p):
        self._target = weakref.ref(p)

players = [Player()]
players[0].inventory.append(5)
cam = Camera()
cam.target = players[0]
if cam.target is not None:
    players.pop()
    try:
        cam.target.inventory.append(1)
        print("python: appended")
    except AttributeError as e:
        print("python: AttributeError:", e)

# the spelling that keeps the object: a strong name taken before the pop
players = [Player()]
cam.target = players[0]
t = cam.target
if t is not None:
    players.pop()
    t.inventory.append(1)
    print("python, t taken first: inventory", t.inventory, "camera sees", "a Player" if cam.target is not None else "None")

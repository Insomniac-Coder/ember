# py_attack2.py: the Python behaviour of every program in the second attack on phase3b-references.md.
# Run: python -I py_attack2.py   (prints "all Python checks passed")
import sys


class Idle: pass
class Walking:
    def __init__(self, path): self.path = path
class Running:
    def __init__(self, path): self.path = path
class Npc:
    def __init__(self, state): self.state = state


# A23 (answers report 7.3): the state machine in the loop body, on the loop's element.
def tick(npcs, hurry):
    for npc in npcs:
        if isinstance(npc.state, Walking):
            npc.state.path.pop()
            if not npc.state.path:
                npc.state = Idle()
            elif hurry:
                npc.state = Running(npc.state.path)

npcs = [Npc(Walking([1, 2])), Npc(Walking([7]))]
tick(npcs, True)
assert isinstance(npcs[0].state, Running) and npcs[0].state.path == [1]
assert isinstance(npcs[1].state, Idle)


# A23 (answers report 7.4): assigning to the whole binding only rebinds the name.
def peek_or_default(npc):
    if isinstance(npc.state, Walking):
        path = npc.state.path
        if len(path) > 3:
            path = [0]
        return len(path)
    return 0

n = Npc(Walking([1, 2, 3, 4, 5]))
assert peek_or_default(n) == 1
assert len(n.state.path) == 5          # the stored path is untouched


class Person:
    def __init__(self, name): self.name, self.log = name, []

p = Person("ann")
old = p.name
old = old.upper()                       # rebinds `old`; p.name is unchanged
assert p.name == "ann" and old == "ANN"


# A7 (answers report 5.3): the viewed field written through an index, the words read through another index.
class Document:
    def __init__(self, text):
        self.text = text
        self.words = text.split()

docs = [Document("the quick brown fox")]
i = j = 0
docs[i].text += " and more words"
assert docs[j].words[0] == "the"        # stale, but a valid string: Python never frees what is still held
docs[i].words = docs[i].text.split()
assert len(docs[i].words) == 7


# A22 (answers report 6.4): the outer variable is a parameter. Python rebinds the parameter only;
# the caller's variable keeps its old value.
def pick(best, lines):
    for line in lines:
        w = line.lower().strip()
        if len(w) > len(best):
            best = w
    return best

b = ""
assert pick(b, ["Hello", "World!"]) == "world!" and b == ""


# A22 (answers report 6.5, as saved by the first attack): a name read from a reader's line buffer.
# Python's result is the size of the name, whatever buffer the line came from.
import io
f = io.StringIO("  ann  \n" + "x" * 65536 + "\n")
name = f.readline().strip()
assert name == "ann" and sys.getsizeof(name) < 100


print("all Python checks passed")

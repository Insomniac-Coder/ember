# What Python does with the programs of c/attacks.c, and with the readability attacks of the report.
# Run: python -I py_attack.py   (output in py_attack_output.txt)
import gc, weakref

def check(name, got, want):
    assert got == want, (name, got, want)
    print(f"{name}: {got!r}")

class Node:
    def __init__(self, name): self.name, self.parent, self.children = name, None, []
    def add(self, c): c.parent = self; self.children.append(c)

# attacks.exe 1/2: an element assigned from the same list
p = Node("p"); a, b = Node("a"), Node("b"); p.add(a); p.add(b)
p.children[1] = p.children[0]
check("assign_same_list", [c.name for c in p.children], ["a", "a"])

# attacks.exe 3: the swap idiom
p = Node("p"); a, b, c = Node("a"), Node("b"), Node("c"); p.add(a); p.add(b); p.add(c)
p.children[0], p.children[2] = p.children[2], p.children[0]
check("swap_idiom", [x.name for x in p.children], ["c", "b", "a"])

# attacks.exe 4/5: a weak reference whose holder dies first
class Player: pass
class Camera:
    def __init__(self, t): self._t = weakref.ref(t)
pl = Player(); cam = Camera(pl); del cam; del pl; gc.collect()
check("weak_holder_dies_first", "no error", "no error")

# attacks.exe 6/7: the narrowed parent changes under a re-parent
class Folder(Node):
    def __init__(self, name): super().__init__(name); self.icon = "folder.png"
def reparent(n, new):
    if n.parent is not None: n.parent.children.remove(n)
    new.add(n)
root, docs, other, n = Node("root"), Folder("docs"), Node("other"), Node("n")
root.add(docs); root.add(other); docs.add(n)
try:
    if isinstance(n.parent, Folder):
        reparent(n, other)
        print(n.parent.icon)
    got = "printed"
except AttributeError as e:
    got = "AttributeError"
check("narrowed_parent_changes", got, "AttributeError")

# attacks.exe 8/9: a captured list cleared by a stored callback while a loop walks it
def boxed_loop():
    xs = ["ab", "cde", "f"]
    handlers = {}
    handlers["reset"] = lambda: xs.clear()
    total = 0
    for x in xs:
        handlers["reset"]()
        total += len(x)
    return total
check("boxed_list_cleared_in_loop", boxed_loop(), 2)

# attacks.exe 10/11: a closure replaced (and dropped) while it runs
def captured_closure():
    out = []
    name = "first"
    on_tick = lambda: out.append("noop")
    def reset():
        nonlocal on_tick
        on_tick = lambda: out.append("second")
    def tick():
        reset()
        out.append(name)
    on_tick = tick
    on_tick()
    return out
check("closure_replaced_while_running", captured_closure(), ["first"])

# readability: Python's own tree idiom writes both ends of the link
p = Node("p"); c = Node("c"); c.parent = p; p.children.append(c)
check("python_add_writes_both_ends", (c.parent.name, [x.name for x in p.children]), ("p", ["c"]))

# readability: the detach-all loop writes the back link while walking the list
p = Node("p"); kids = [Node("x"), Node("y")]
for k in kids: p.add(k)
for k in p.children: k.parent = None
p.children = []
check("detach_all_loop", [k.parent for k in kids], [None, None])

# readability: isinstance with early exit, and assert, narrow in mypy's reading
def area(s):
    if not isinstance(s, Folder):
        return None
    return s.icon
check("early_exit_isinstance", area(Folder("f")), "folder.png")
print("all Python checks passed")

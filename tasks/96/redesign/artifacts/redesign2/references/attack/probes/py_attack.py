# Python behaviour for every attack in phase3b-references-attack.md. Run: python -I py_attack.py
class Hero:
    def __init__(self, hp): self.hp = hp
    def heal(self, n):
        self.hp += n
        return self.hp > 50
class Registry:
    def __init__(self): self.heroes = [Hero(10)]
    def get(self, i): return self.heroes[i]
reg = Registry()
if reg.get(0).heal(100): pass
assert reg.heroes[0].hp == 110                    # x22a: Python changes the stored hero

class Person:
    def __init__(self, name): self.name, self.log = name, []
p = Person("ann")
old = p.name; alias = old; p.name = "bob"; p.log.append(f"renamed {alias}")
assert p.log[0] == "renamed ann" and old == "ann"  # x23a

out = []
class Guard:
    def __init__(self, name): self.name = name
    def __del__(self): out.append(f"released {self.name}")
def swap_guard(holder, other):
    old = holder[0].name
    holder[0] = other
    out.append(old)
h = [Guard("ann")]
swap_guard(h, Guard("bob"))
assert out == ["released ann", "ann"], out          # x23c: the old guard is released at the overwrite

words = "the quick brown fox".split()
w = words[0]; docs = [words]; docs.clear()
assert w == "the"                                   # x07a: a kept word survives its list

import sys
def first_word(line): return line.lower().split(" ")[0]
big = ["Word " + "x" * 1_000_000 for _ in range(10)]
kept = [first_word(l) for l in big]
assert kept[0] == "word" and sys.getsizeof(kept[0]) < 100   # x22b: the kept word is its own small string
import weakref
class Lexer:
    def __init__(self, src): self.src = src
class Parser:
    def __init__(self, lexer): self.lexer, self.me = lexer, None
    def describe(self): return self.lexer.src
def reset(p, s): p.lexer = Lexer(s)
def make_and_reset(p):
    short = "short lived"
    reset(p, short)
text = "let x = 1"; p = Parser(Lexer(text)); q = p
make_and_reset(p)
assert q.lexer.src == "short lived"                 # x06a
class Reg:
    def __init__(self): self.items, self.seen = [], []
def add_parser(reg):
    t = "let x = 1"
    reg.items.append(Parser(Lexer(t)))
    pp = Parser(Lexer(t)); pp.me = pp; reg.seen.append(weakref.ref(pp))
reg2 = Reg(); add_parser(reg2)
assert reg2.items[0].describe() == "let x = 1"      # x06b
assert reg2.seen[0]() is not None and reg2.seen[0]().lexer.src == "let x = 1"   # x06c: the cycle keeps it alive
def put(v, s): v.append(s)
def fill(v):
    word = "short lived"
    put(v, word)
vs = []; fill(vs)
assert vs[0] == "short lived"                       # x06d
class Document:
    def __init__(self, text): self.text, self.words = text, text.split()
class Holder:
    def __init__(self, doc): self.doc = doc
h1 = Holder(Document("the quick brown fox")); h2 = h1
h2.doc.text = "zzz"
assert h1.doc.words[0] == "the"                     # x07b: stale but valid
class Tok:
    def __init__(self, buf, cur): self.buf, self.cur = buf, cur
    def refill(self):
        self.buf = "fresh words"; self.cur = self.buf[0:5]
t = Tok("", "hello"); t.refill(); t.buf = "x"
assert t.cur == "fresh"                             # x07c
def maybe_rename(p, new, really):
    old = p.name
    if really: p.name = new
    return old
assert maybe_rename(Person("ann"), "bob", True) == "ann" and maybe_rename(Person("ann"), "bob", False) == "ann"  # x23b
print("all Python attack checks passed")

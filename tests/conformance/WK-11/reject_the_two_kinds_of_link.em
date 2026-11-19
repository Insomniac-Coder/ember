#$ test: compile-fail
#$ rules: WK-11, OWN-13, THR-1, THR-2
# `[WK-11]`, `[OWN-13]` — the two kinds of link do not convert; a `weakref T` link is made only to what no
# `thread_shared` value reaches; the two words stand together before `this` alone; and a link to a value
# on one thread stays on that thread.

import std.thread

class Item:
    n: int = 0

class Tree:
    item: Item = Item()

class Keeper:
    plain: weakref Item = None
    shared: weakref thread_shared Item = None

fn peek(_w: weakref Item) -> int:
    return 0

fn main():
    item = thread_shared Item()
    one = Item()
    k = Keeper()
    k.plain = weakref item    #$ error[E2020]: expected `weakref Item`, found `weakref thread_shared Item`
    k.shared = weakref one    #$ error[E2020]: expected `weakref thread_shared Item`, found `weakref Item`
    tree = thread_shared Tree()
    _a = weakref tree.item    #$ error[E7015]: is reached through a value threads share
    _b = weakref thread_shared item    #$ error[E0109]: a link to it is a `weakref thread_shared` link without the word
    _c = weakref thread_shared one    #$ error[E7017]: was not created `thread_shared`
    plain = weakref one
    _t = thread.spawn(owns owned fn() => peek(plain))    #$ error[E7015]: captures `plain`, a `weakref Item`

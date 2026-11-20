# Clause RC-5.1: "a write to a place that owns one (...) and a place of a function type or of `owns
# dyn I` whose value's state owns one: `h.keep = fn(): pass` ends the function stored there before
# and the node its `owns n` held), (...) a call through a function value that may run `fn() owns
# players: players.clear()`"
#$ test: compile-fail
#$ rules: RC-5, CLO-17, EFF-1, DRP-8
# The spec's two messages, with this file's line numbers. Alone because capture words are M8's.

class Player:
    hp: int = 5

class Camera:
    target: weakref Player = None

class Node:
    v: int = 1

class Holder:
    keep: fn() = noop

fn noop():
    pass

static mut ON_RESET: fn() = noop

fn reset():
    ON_RESET()

fn through_a_stored_function():
    players = [Player()]
    cam = Camera()
    cam.target = weakref players[0]
    ON_RESET = fn() owns players: players.clear()
    if cam.target exists as p:
        reset()
        println(p.hp)    #$ error[E3060]: `reset()` (line 35) may end `Player` objects (it runs the function stored on line 33, which clears the players it owns), and `p` reads one (`if cam.target exists as p:`, line 34) and is used on line 36; use `p` before line 35, or open the link again after it: write `if cam.target exists as q:` above line 36, indent line 36 under it and use `q` there

fn through_a_replaced_function():
    h = Holder()
    n = Node()
    w = weakref n
    h.keep = fn() owns n: println(n.v)
    if w exists as p:
        h.keep = fn(): pass
        println(p.v)    #$ error[E3060]: `h.keep = fn(): pass` (line 44) ends the function stored there on line 42 and the node it owns (`owns n`), and `p` reads that node (`if w exists as p:`, line 43) and is used on line 45; use `p` before line 44

fn main():
    through_a_stored_function()
    through_a_replaced_function()

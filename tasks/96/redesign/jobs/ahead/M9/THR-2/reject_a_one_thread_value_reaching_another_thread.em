# Clause THR-2.1: "no such owner, and no `weakref T` link, reaches another thread by any route"; "in
# a scoped task, a job, or any function that may run on another thread than its target's owner, it
# is allowed only through a `weakref thread_shared T`"; "`weakref x` where the place `x` is reached
# through a value every thread may reach, (...) a `static` (...) is `E7015`"
#$ test: compile-fail
#$ rules: THR-2, THR-10, RC-4, WK-11
# A thread given a `shares` owner (code only), a task that makes a link (the spec's message), and a
# link to a `static` (the spec's message), with this file's line numbers.

import std.thread

class Player:
    hp: int = 5

class Node:
    x: int = 0

static ORIGIN: Node = Node(x=0)

fn main():
    a = shares Player()
    h = thread.spawn(owns fn() owns a: a.hp)    #$ error[E7015]
    println(h.join())
    players = [Player()]
    links: Array[weakref Player] = []
    with scope = thread.scope():
        scope.spawn(owns fn(): links.push(weakref players[0]))    #$ error[E7015]: `players[0]` counts its links without atomics, and the task on line 27 makes one too; make the links after the scope, or make the player `thread_shared`
    w = weakref ORIGIN    #$ error[E7015]: `ORIGIN` is a `static`, which every thread may read, and `weakref ORIGIN` would count its links on this thread alone; read `ORIGIN` by its name, or share it between threads: write `static ORIGIN: thread_shared Node = thread_shared Node(x=0)` in place of line 18, after which `weakref ORIGIN` is a `weakref thread_shared Node`
    println(links.len())

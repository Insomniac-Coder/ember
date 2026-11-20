# Clause THR-2.2: "**An object handed whole to another thread, a scoped task or a job included,
# leaves its `weakref` links behind** (verdict 114 (...)). (...) after `w = weakref names` and `h =
# thread.spawn(owns fn() owns names: names.items.len())`, `if w exists as p:` is skipped and
# `h.join()` gives 0"; "so after `cam.target = weakref level.enemies[0]`, handing `level` over
# leaves `cam.target` finding enemy 0 ended"
#$ test: run-pass
#$ rules: THR-2, WK-12, OBJ-1, THR-10
#$ stdout: gone
#$ stdout: 0
#$ stdout: enemy gone
#$ stdout: 1
#$ stdout: kept
# The spec's two cases; and an object with no link goes as it is.

import std.thread

class Names:
    items: Array[String] = []

class Enemy:
    hp: int = 3

class Level:
    enemies: Array[Enemy] = []

class Camera:
    target: weakref Enemy = None

fn main():
    names = Names()
    w = weakref names
    h = thread.spawn(owns fn() owns names: names.items.len())
    if w exists as p:
        println(p.items.len())
    else:
        println("gone")
    println(h.join())
    level = Level()
    level.enemies.push(owns Enemy())
    cam = Camera()
    cam.target = weakref level.enemies[0]
    g = thread.spawn(owns fn() owns level: level.enemies.len())
    if cam.target exists as e:
        println(e.hp)
    else:
        println("enemy gone")
    println(g.join())
    plain = Names()
    k = thread.spawn(owns fn() owns plain: plain.items.len())
    k.join()
    println("kept")

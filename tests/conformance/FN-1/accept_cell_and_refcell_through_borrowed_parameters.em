#$ test: run-pass
#$ rules: FN-1, BRW-8, CELL-1, CELL-5
#$ stdout: 2
#$ 24 1
# IX.7's example: a borrowed parameter is the caller's value (ODR-024), so a
# `Cell` set through it and a `RefCell` pushed through it change the caller's.
# Passed as a copy, the first printed 0 and the second corrupted the heap.

struct Sprite:
    frame: Cell[int]

fn advance(s: Sprite):
    s.frame.set(s.frame.get() + 1)

struct Scene:
    entities: RefCell[Array[int]]

fn add(s: Scene, e: int):
    with list = s.entities.borrow_mut():
        list.push(e)

fn main():
    sprite = Sprite(frame=Cell(0))
    advance(sprite)
    advance(sprite)
    println(sprite.frame.get())
    scene = Scene(entities=RefCell([1, 2, 3, 4]))
    for i in range(20):
        add(scene, i)
    with list = scene.entities.borrow():
        println(list.len(), list[0])

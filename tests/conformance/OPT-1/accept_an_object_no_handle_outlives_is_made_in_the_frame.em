#$ test: run-pass
#$ rules: OPT-1, DRP-2
#$ profiles: debug, release
#$ stdout: drop local 0
#$ drop local 1
#$ drop local 2
#$ 6
#$ 2
#$ 2
#$ assert-c: contains("(struct em_obj_5Local*)ember_obj_new_frame(&_fo")
#$ assert-c: contains("((struct em_obj_4Kept*)ember_obj_new(&em_ti_4Kept))")
#$ assert-c: !contains("(struct em_obj_4Kept*)ember_obj_new_frame(")
#$ assert-c: contains("((struct em_obj_7Carried*)ember_obj_new(&em_ti_7Carried))")
# `[OPT-1]`: when escape analysis proves that no handle to an object outlives the function that
# made it, the object is made in that function's frame, with the same header, and its `drop` runs
# at its last release as it would. This cannot be observed: the output is what the heap gives.
# A `Local` made, used (through a function whose parameter does not escape) and dropped in one turn
# of a loop is in the frame, made again each turn once the last is released. A `Kept` pushed onto
# a list escapes; a `Carried` that `last` holds past its turn would be made again while still held:
# both stay on the heap.

class Local:
    id: int

    fn twice(self) -> int:
        return self.id * 2

    fn drop(mut self):
        println(f"drop local {self.id}")

class Kept:
    id: int

class Carried:
    id: int

fn read(p: Local) -> int:
    return p.twice()

fn store(mut list: Array[Kept], k: Kept):
    list.push(k)

fn turns(n: int) -> int:
    total = 0
    for i in 0..n:
        p = Local(i)
        total += read(p)
    return total

fn kept(n: int) -> int:
    list: Array[Kept] = []
    for i in 0..n:
        store(list, Kept(i))
    return list.len()

fn carried(n: int) -> int:
    last = Carried(-1)
    for i in 0..n:
        c = Carried(i)
        last = c
    return last.id

fn main():
    println(turns(3))
    println(kept(2))
    println(carried(3))

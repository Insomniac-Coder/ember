#$ perf: counted
## `[BEN-6]` — counted-object code against a C++ intrusive count, atomic: as `counted_handles`, its
## objects `thread_shared`, so their counts are atomic, against a twin counting with `std::atomic`.

class Cell:
    value: int

    fn init(value: int):
        this.value = value

@overflow(wrap)
fn main():
    cells: Array[thread_shared Cell] = []
    for i in 0..1000:
        cells.push(owns thread_shared Cell(i))
    ring: Array[thread_shared Cell] = []
    for i in 0..1000:
        ring.push(owns thread_shared cells[i])
    total = 0
    for round in 0..20000:
        for i in 0..ring.len():
            ring[i] = thread_shared cells[(i * 7 + round) % cells.len()]
        for i in 0..ring.len():
            total = total + ring[i].value
    println(total)

#$ test: run-pass
#$ rules: UNS-10, OWN-2, TYP-18

from std.mem import UnsafeCell

struct Bag:
    pub values: Array[i32]

    fn drop():
        println(this.values.len())

fn make() -> Bag:
    values: Array[i32] = Array[i32]()
    values.push(owns 1)
    values.push(owns 2)
    return Bag(values)

[T]
fn extract(owns cell: UnsafeCell[T]) -> T:
    return cell.into_inner()

fn discard():
    _cell = UnsafeCell(make())

fn main():
    println(size_of[UnsafeCell[i64]]())
    discard()
    direct = UnsafeCell(make())
    moved: Bag = extract(owns direct)
    println(moved.values.len())
#$ stdout: 8
#$ 2
#$ 2
#$ 2

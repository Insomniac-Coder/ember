#$ test: run-pass
#$ rules: UNS-10, OWN-2, TYP-18

from std.mem import UnsafeCell

struct Bag:
    pub values: Array[i32]

    fn drop(mut self):
        println(self.values.len())

fn make() -> Bag:
    values: Array[i32] = Array[i32]()
    values.push(1)
    values.push(2)
    return Bag(values)

fn extract[T](owned cell: UnsafeCell[T]) -> T:
    return cell.into_inner()

fn discard():
    _cell = UnsafeCell(make())

fn main():
    println(size_of[UnsafeCell[i64]]())
    discard()
    direct = UnsafeCell(make())
    moved: Bag = extract(direct)
    println(moved.values.len())
#$ stdout: 8
#$ 2
#$ 2
#$ 2

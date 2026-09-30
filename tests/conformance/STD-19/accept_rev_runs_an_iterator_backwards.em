#$ test: run-pass
#$ rules: STD-19, CTL-3
#$ profiles: debug, release, shipping
#$ stdout: [30, 20, 10] [70, 60, 50] [70, 40, 10]
#$ stdout: [(30, 3), (20, 2), (10, 1)]
#$ stdout: [3, 2, 1, 20, 10]
#$ stdout: Some(10) Some(70) 5
#$ stdout: Some(3) Some(1) Some(2) None 0
#$ stdout: [] [255, 254, 253] [-126, -127, -128]
#$ stdout: 3 7 3
#$ stdout: [(6, 70), (5, 60), (4, 50)]
# `[STD-19]` (ODR-091) — `rev` where an iterator can run backwards
# (`DoubleEndedIterator`): the ranges with an end, the views' element
# iterators, and each adapter over them where it can tell which item is last
# (`take`, `skip`, `step_by`, `zip` and `enumerate` by the length below,
# `ExactSizeIterator`). The items are the ones `next` gives, last first; the
# two ends meet and give no item twice; a range at a type's top or bottom
# never steps past it.

fn main():
    xs: Array[int] = [10, 20, 30, 40, 50, 60, 70]
    ys: Array[int] = [1, 2, 3]
    t: Array[int] = xs.iter().copied().take(3).rev().to_array()
    s: Array[int] = xs.iter().copied().skip(4).rev().to_array()
    k: Array[int] = xs.iter().copied().step_by(3).rev().to_array()
    z: Array[(int, int)] = xs.iter().copied().zip(ys.iter().copied()).rev().to_array()
    c: Array[int] = xs.iter().copied().take(2).chain(ys.iter().copied()).rev().to_array()
    println(t, s, k)
    println(z)
    println(c)
    it = xs.iter()
    front = it.next()
    back = it.next_back()
    println(front, back, it.len())
    r = (1..=3).iter()
    println(r.next_back(), r.next(), r.next_back(), r.next_back(), r.len())
    e: Array[int] = (5..5).iter().rev().to_array()
    top: Array[u8] = (253..=255 as u8).iter().rev().to_array()
    low: Array[i8] = (-128 as i8..=-126 as i8).iter().rev().to_array()
    println(e, top, low)
    println((0..10).iter().step_by(4).rev().len(), (0..10).iter().skip(3).len(), (0..3).iter().take(10).len())
    numbered: Array[(int, int)] = xs.iter().copied().enumerate().rev().take(3).to_array()
    println(numbered)

#$ test: run-fail
#$ rules: SIMD-9
#$ profiles: debug, development, release
#$ stdout: [11, 13, 10, 12]
#$ panics: lane 2: index 4 is out of bounds for a length of 4
# `[SIMD-9]` — a gather checks every lane's index against the view's length, and a failure is a
# `Bounds` panic naming the lane.
from std.simd import i32x4

fn main():
    xs: Array[i32] = [10, 11, 12, 13]
    println(i32x4.gather(xs, i32x4.from_array([1, 3, 0, 2])).to_array())
    v = i32x4.gather(xs, i32x4.from_array([1, 3, 4, 2]))
    println(v.to_array())

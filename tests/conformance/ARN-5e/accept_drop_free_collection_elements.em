#$ test: run-pass
#$ rules: ARN-5e, TST-24

from std.collections import ArenaArray, ArenaMap

struct Point:
    x: i32
    y: i32

fn main():
    arena = Arena.with_capacity(2048)
    points: ArenaArray[Point] = ArenaArray[Point].with_capacity(arena, 1)
    _point = points.push(Point(3, 4))
    pairs: ArenaMap[i32, Point] = ArenaMap[i32, Point].with_capacity(arena, 1)
    _pair = pairs.insert(1, Point(5, 6))
    println(points.len() + pairs.len())
#$ stdout: 2

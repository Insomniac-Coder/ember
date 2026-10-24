# The loop form: Python halves every x (copy_struct_loop [0.5, 1.0]); today a loop element only reads.
@derive(Copy)
struct Vec2:
    x: f64
    y: f64

fn main():
    vels: Array[Vec2] = [Vec2(x=1.0, y=1.0), Vec2(x=2.0, y=2.0)]
    for v in vels:
        v.x *= 0.5
    println(vels[0].x)        # Python 0.5

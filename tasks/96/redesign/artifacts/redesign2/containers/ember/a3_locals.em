# A3, the same with two names: a = bodies[i], b = bodies[j], then change both.
struct Body:
    x: float
    vx: float

fn step(mut bodies: Array[Body], i: int, j: int):
    a = ref mut bodies[i]
    b = ref mut bodies[j]
    t = a.vx
    a.vx = b.vx
    b.vx = t

fn main():
    bs = [Body(x=0.0, vx=1.0), Body(x=1.0, vx=-1.0)]
    step(bs, 0, 1)
    println(bs[0].vx)

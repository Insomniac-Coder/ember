# A3 as a Python programmer writes it: two bodies of one list given to one call.
struct Body:
    x: float
    vx: float

fn collide(mut a: Body, mut b: Body):
    t = a.vx
    a.vx = b.vx
    b.vx = t

fn step(mut bodies: Array[Body], i: int, j: int):
    collide(bodies[i], bodies[j])

fn main():
    bs = [Body(x=0.0, vx=1.0), Body(x=1.0, vx=-1.0)]
    step(bs, 0, 1)
    println(bs[0].vx)

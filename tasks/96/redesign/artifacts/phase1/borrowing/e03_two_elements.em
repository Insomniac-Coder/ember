# A3: two mutable references into one array.
struct Body:
    pos: float
    vel: float

fn collide(mut a: Body, mut b: Body):
    mem.swap(a.vel, b.vel)

fn step(mut bodies: Array[Body], i: int, j: int):
    collide(bodies[i], bodies[j])

fn main():
    bs = [Body(pos=0.0, vel=1.0), Body(pos=1.0, vel=-1.0)]
    step(bs, 0, 1)
    println(bs[0].vel)

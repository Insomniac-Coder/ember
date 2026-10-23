# A3 with Ember's sanctioned spelling: get_pair_mut ([BRW-5]).
struct Body:
    pos: float
    vel: float

fn collide(mut a: Body, mut b: Body):
    mem.swap(a.vel, b.vel)

fn step(mut bodies: Array[Body], i: int, j: int):
    match bodies.get_pair_mut(i, j):
        Some((a, b)):
            collide(a, b)
        None:
            pass

fn main():
    bs = [Body(pos=0.0, vel=1.0), Body(pos=1.0, vel=-1.0)]
    step(bs, 0, 1)
    println(bs[0].vel)

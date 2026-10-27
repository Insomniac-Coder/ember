#$ test: compile-fail
#$ rules: R4
#$ help: write `if i != j:` above this line and indent it under it
## R4 (verdict 98): two elements of one list whose positions are not proved different may not be given
## to one call that may change both; `i` and `j` may be equal, so the call is refused when compiling
## (E3022), with `if i != j:` as the way out (R4/accept_two_bodies_inside_if_i_is_not_j.em).

struct Body:
    pos: float
    vel: float

fn collide(ref mut a: Body, ref mut b: Body):
    t = a.vel
    a.vel = b.vel
    b.vel = t

fn step(ref mut bodies: Array[Body], i: int, j: int):
    collide(ref mut bodies[i], ref mut bodies[j])    #$ error[E3022]: `collide` changes `a` and `b` as two different bodies, and `bodies[i]` and `bodies[j]` are the same body when `i == j`; write `if i != j:` above this line and indent it under it

fn main():
    bs = [Body(pos=0.0, vel=1.0), Body(pos=1.0, vel=-1.0)]
    step(ref mut bs, 0, 1)
    println(bs[0].vel, bs[1].vel)

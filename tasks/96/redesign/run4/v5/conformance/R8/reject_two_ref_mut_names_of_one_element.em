#$ test: compile-fail
#$ rules: BRW-1
#$ help: b = ref mut bodies[1]
## design rules: R8
## R8 (f): two `ref mut` names of one place known when compiling are refused (E3022), as R4 refuses
## `collide(ref mut bodies[i], ref mut bodies[i])`.

struct Body:
    vel: float

fn main():
    bodies = [Body(vel=1.0), Body(vel=2.0)]
    a = ref mut bodies[0]
    b = ref mut bodies[0]    #$ error[E3022]: `a` is the only name for `bodies[0]` until its last use on line 14, and line 13 names it again as `b`; use `a` itself in place of `b`, or name another body: `b = ref mut bodies[1]`
    a.vel = b.vel + 1.0

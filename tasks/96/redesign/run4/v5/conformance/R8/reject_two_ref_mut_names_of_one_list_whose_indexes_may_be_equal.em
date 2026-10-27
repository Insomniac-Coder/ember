#$ test: compile-fail
#$ rules: R8, R4
#$ help: write `if i != j:` above line 11 and indent lines 11 to 15 under it
## R8 (f) (verdict 98): two `ref mut` names of elements of one list whose indexes are not proved
## different are refused as R4 refuses two such arguments (E3022): `i` and `j` may be equal.

struct Body:
    vel: float

fn swap_vel(ref mut bodies: Array[Body], i: int, j: int):
    a = ref mut bodies[i]
    b = ref mut bodies[j]    #$ error[E3022]: `a` and `b` both change elements of `bodies`, and `bodies[i]` and `bodies[j]` are the same body when `i == j`; write `if i != j:` above line 11 and indent lines 11 to 15 under it
    t = a.vel
    a.vel = b.vel
    b.vel = t

fn main():
    bs = [Body(vel=1.0), Body(vel=2.0)]
    swap_vel(ref mut bs, 0, 1)
    println(bs[0].vel, bs[1].vel)

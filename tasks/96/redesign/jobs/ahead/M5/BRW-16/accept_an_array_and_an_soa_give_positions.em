# Clause BRW-16.6: "A list here is an `Array[T]`, whose `xs[i]` and `xs.get(i)` are its own built-in
# indexing (`[STD-15]`, `[STD-17]`), and an `SoA[T]`'s element proxy is a position likewise
# (`[SOA-6]`)"
#$ test: run-pass
#$ rules: BRW-16, SOA-6, SOA-1, STD-15
#$ stdout: 5.0 2
#$ stdout: 7 2
# A proxy `ref mut ps[0]`, and a name from `xs.get(0)`, each survive an add.

struct Particle:
    life: float

struct Enemy:
    hp: int

fn main():
    ps = SoA[Particle].with_capacity(1)
    ps.push(owns Particle(life=1.0))
    p = ref mut ps[0]
    ps.push(owns Particle(life=2.0))
    p.life = 5.0
    println(ps[0].life, ps.len())
    xs = [Enemy(hp=7)]
    e = xs.get(0).unwrap()
    xs.push(owns Enemy(hp=1))
    println(e.hp, xs.len())

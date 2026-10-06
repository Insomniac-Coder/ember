struct Particle:
    x: float
    y: float
    vx: float
    vy: float

fn main():
    ps: Array[Particle] = []
    for i in 0..100000:
        ps.push(Particle(i as float, 0.0, 1.0, 0.5))
    for step in 0..2000:
        for p in ps.iter_mut():
            p.x = p.x + p.vx
            p.y = p.y + p.vy
    total = 0.0
    for p in ps:
        total = total + p.x + p.y
    println(total as int)

# A particle fountain: spawn particles with a seeded generator, apply gravity and bounces, and drop the dead ones.

from std.math import Vec3

const GRAVITY: float = -9.81
const DT: float = 1.0 / 60.0

@derive(Copy)
struct Particle:
    position: Vec3
    velocity: Vec3
    age: float = 0.0
    lifetime: float

struct Rng:
    state: int

    fn next_float() -> float:
        this.state = (this.state * 1103515245 + 12345) % 2147483648
        return float(this.state) / 2147483648.0

    fn range(low: float, high: float) -> float:
        return low + (high - low) * this.next_float()

class Fountain:
    particles: Array[Particle] = []
    rng: Rng = Rng(state=7)
    bounces: int = 0

    fn spawn(count: int):
        for _ in 0..count:
            vx = this.rng.range(-1.0, 1.0)
            vy = this.rng.range(8.0, 12.0)
            vz = this.rng.range(-1.0, 1.0)
            this.particles.push(owns Particle(
                position=Vec3.ZERO,
                velocity=Vec3(vx, vy, vz),
                lifetime=this.rng.range(1.0, 3.0)))

    fn update(dt: float):
        for p in this.particles.iter_mut():
            p.velocity.y += GRAVITY * dt
            p.position += p.velocity * dt
            if p.position.y < 0.0:
                p.position.y = 0.0
                p.velocity.y = -p.velocity.y * 0.5
                this.bounces += 1
            p.age += dt
        this.particles.retain(owns fn(p) => p.age < p.lifetime)

    fn highest() -> float:
        best = 0.0
        for p in this.particles:
            if p.position.y > best:
                best = p.position.y
        return best

fn main():
    fountain = Fountain()
    for frame in 0..300:
        if frame < 120 and frame % 10 == 0:
            fountain.spawn(50)
        fountain.update(DT)
        if frame % 60 == 0:
            t = float(frame) * DT
            println(f"t={t:.1f}s alive={fountain.particles.len()} highest={fountain.highest():.2f}")
    println(f"bounces: {fountain.bounces}, alive at the end: {fountain.particles.len()}")

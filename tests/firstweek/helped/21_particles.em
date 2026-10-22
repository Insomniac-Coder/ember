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

    fn next_float(mut self) -> float:
        self.state = (self.state * 1103515245 + 12345) % 2147483648
        return self.state as float / 2147483648.0

    fn range(mut self, low: float, high: float) -> float:
        return low + (high - low) * self.next_float()

class Fountain:
    particles: Array[Particle] = []
    rng: Rng = Rng(state=7)
    bounces: int = 0

    fn spawn(self, count: int):
        for _ in 0..count:
            vx = self.rng.range(-1.0, 1.0)
            vy = self.rng.range(8.0, 12.0)
            vz = self.rng.range(-1.0, 1.0)
            self.particles.push(Particle(
                position=Vec3.ZERO,
                velocity=Vec3(vx as f32, vy as f32, vz as f32),
                lifetime=self.rng.range(1.0, 3.0)))

    fn update(self, dt: float):
        for p in self.particles.iter_mut():
            p.velocity.y += (GRAVITY * dt) as f32
            p.position += p.velocity * dt as f32
            if p.position.y < 0.0:
                p.position.y = 0.0
                p.velocity.y = -p.velocity.y * 0.5
                self.bounces += 1
            p.age += dt
        self.particles.retain(fn(p) => p.age < p.lifetime)

    fn highest(self) -> float:
        best: f32 = 0.0
        for p in self.particles:
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
            t = frame as float * DT
            println(f"t={t:.1f}s alive={fountain.particles.len()} highest={fountain.highest():.2f}")
    println(f"bounces: {fountain.bounces}, alive at the end: {fountain.particles.len()}")

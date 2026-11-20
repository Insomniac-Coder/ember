#$ test: run-pass
#$ rules: TYP-46, ALC-1, ALC-2, ARN-2
#$ profiles: debug, development, release
#$ stdout: 2 16
## `[ARN-2]` — `Arena` implements `Allocator` (`[ALC-2]`): a value that needs drop is made in an
## arena as an `owns T using frame`, whose end runs its clean-up and gives its block back to the
## arena, which frees nothing one by one, so the value keeps no instance word (`[ALC-1]`): an `Enemy`
## is its `hp` and one address. IX.3's example.

struct Mesh:
    points: Array[float]

struct Enemy:
    hp: int
    mesh: owns Mesh using Arena

fn main():
    level = Arena.with_capacity(64 * 1024)
    boss = Enemy(hp=500, mesh=owns Mesh(points=[0.0, 1.0]) using level)
    println(boss.mesh.points.len(), size_of[Enemy]())

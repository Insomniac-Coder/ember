# Proposed syntax, readability attack. Each task changes its own player (the first attack's saved claims,
# its 3.5) and re-targets it. `p.target = ...` releases the old target, an object the parent owns.
# Scope rule 4 (answers report 4.3): "A task that drops handles it owns to parent objects writes 'the count
# of some C', so two such tasks conflict." The first attack's rule 4b lets a task retain a parent object
# atomically but "may not release" one. So this is refused; Python runs it.
class Enemy:
    x: float = 0.0
    hp: int = 10

class Player:
    x: float = 0.0
    target: Option[Enemy] = None

fn nearest(enemies: Array[Enemy], x: float) -> Enemy:
    best = enemies[0]
    for e in enemies:
        if abs(e.x - x) < abs(best.x - x):
            best = e
    return best

fn aim(players: Array[Player], enemies: Array[Enemy]):
    @parallel
    for p in players:
        p.target = nearest(enemies, p.x)      # releases the old target, retains the new one

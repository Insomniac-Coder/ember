# `ledger`: another package for the R17 tests (no test header: it is not a test).

pub class Player:
    pub name: String
    pub hp: int

pub class Game:
    pub players: Array[Player]
    pub log: Array[String]
    pub frame: int = 0

    # says what it reads: callers in other packages rely on `reads frame`
    pub fn note(msg: String) reads frame changes log:
        this.log.push(f"[{this.frame}] {msg}")

    # the same body with no `reads`: callers in other packages take it to read every field (R17)
    pub fn note_any(msg: String) changes log:
        this.log.push(f"[{this.frame}] {msg}")

pub struct Enemy:
    pub x: float
    pub hp: int

# only adds at the end of `enemies`: the "only adds" summary, recorded in the interface file
pub fn spawn_minion(ref mut enemies: Array[Enemy], x: float):
    enemies.push(Enemy(x=x, hp=10))

# removes: its summary records that it does more than add
pub fn kill_weak(ref mut enemies: Array[Enemy]):
    i = 0
    while i < enemies.len():
        if enemies[i].hp < 20:
            enemies.remove(i)
        else:
            i += 1

pub class Button:
    pub handlers: Array[fn() changes captured] = []

    pub fn on_click(owns f: fn() changes captured) changes handlers:
        this.handlers.push(f)

    pub fn click() changes handlers, captured:
        for ref mut h in this.handlers:
            h()

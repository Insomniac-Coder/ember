# A1 across packages: `note` is pub and touches only `log` and `frame`.
pub struct Player:
    pub name: String
    pub hp: int

pub struct Game:
    pub players: Array[Player]
    pub log: Array[String]
    pub frame: int = 0

    pub fn note(mut self, msg: String):
        self.log.push(f"[{self.frame}] {msg}")

# A6: structs holding views, and an EnTT-style handle holding `ref mut Registry`.
struct Lexer:
    src: str
    pos: int = 0

struct Parser:
    lexer: Lexer
    errors: Array[String]

    fn next_word(mut self) -> str:
        rest = self.lexer.src[self.lexer.pos..]
        end = rest.find(" ").unwrap_or(rest.len())
        self.lexer.pos += end + 1
        return rest[..end]

struct Registry:
    hp: Array[int]

struct Handle:
    reg: ref mut Registry
    entity: int

    fn damage(mut self, n: int):
        self.reg.hp[self.entity] -= n

fn main():
    text: String = "let x = 1"
    p = Parser(lexer=Lexer(src=text), errors=[])
    w = p.next_word()
    println(w)
    reg = Registry(hp=[10, 10])
    h = Handle(reg=ref mut reg, entity=1)
    h.damage(3)
    println(reg.hp)

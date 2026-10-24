# Attack on A6: an upcast to an interface handle forgets that the object carries a limit.
# `Registry.items` is a class field, so it may hold only handles with no limit; `Array[Named]`
# does not say that a Parser in it views `text`. Rust's twin needs `dyn Named + 'static`: refused.
interface Named:
    fn describe(self) -> String

struct Lexer:
    src: str

class Parser:
    lexer: Lexer

extend Parser implements Named:
    fn describe(self) -> String:
        return self.lexer.src.to_string()

class Registry:
    items: Array[Named]

fn add_parser(reg: Registry):
    text: String = "let x = 1"
    reg.items.push(Parser(lexer=Lexer(src=text)))

fn main():
    reg = Registry(items=[])
    add_parser(reg)
    println(reg.items[0].describe())

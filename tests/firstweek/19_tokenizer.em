# Tokenize a small arithmetic language into numbers, names, operators and parentheses, pointing at bad characters.

enum Token:
    Number(value: float)
    Name(text: String)
    Op(symbol: String)
    LParen
    RParen
    Comma

struct LexError:
    position: int
    found: String

fn is_operator(c: str) -> bool:
    return c in ["+", "-", "*", "/", "^", "="]

fn tokenize(source: str) -> Result[Array[Token], LexError]:
    tokens: Array[Token] = []
    i = 0
    while i < source.len():
        c = source[i]
        if c == " ":
            i += 1
        elif c.isdigit() or c == ".":
            start = i
            while i < source.len() and (source[i].isdigit() or source[i] == "."):
                i += 1
            text = source[start:i]
            tokens.push(Token.Number(text.parse[float]()?))
        elif c.isalpha() or c == "_":
            start = i
            while i < source.len() and (source[i].isalnum() or source[i] == "_"):
                i += 1
            tokens.push(Token.Name(source[start:i]))
        elif is_operator(c):
            tokens.push(Token.Op(c))
            i += 1
        elif c == "(":
            tokens.push(Token.LParen)
            i += 1
        elif c == ")":
            tokens.push(Token.RParen)
            i += 1
        elif c == ",":
            tokens.push(Token.Comma)
            i += 1
        else:
            return Err(LexError(position=i, found=c))
    return Ok(tokens)

fn show(t: Token) -> String:
    return match t:
        Number(v) => f"NUM({v})"
        Name(s) => f"NAME({s})"
        Op(s) => f"OP({s})"
        LParen => "("
        RParen => ")"
        Comma => ","

fn main():
    inputs = [
        "x = 3 + 4.5 * (y - 2)",
        "area = pi * r ^ 2",
        "max(a, b) / 2",
        "total = price * 1.2 $ tax",
        "1.2.3 + 4",
    ]
    for line in inputs:
        println(f"input: {line}")
        match tokenize(line):
            Ok(tokens):
                println("  " + " ".join([show(t) for t in tokens]))
                names = 0
                for t in tokens:
                    if Name(_) = t:
                        names += 1
                println(f"  {tokens.len()} tokens, {names} names")
            Err(e):
                println(f"  error at column {e.position}: unexpected '{e.found}'")
                println("  " + line)
                println("  " + " " * e.position + "^")

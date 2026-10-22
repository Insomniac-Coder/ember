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
    chars = source.chars().collect[Array[char]]()
    i = 0
    while i < source.len():
        c = chars[i]
        if c == ' ':
            i += 1
        elif c.is_digit() or c == '.':
            start = i
            while i < source.len() and (chars[i].is_digit() or chars[i] == '.'):
                i += 1
            text = source[start..i]
            tokens.push(Token.Number(text.parse[float]().map_err(fn(_) => LexError(position=start, found=text.to_string()))?))
        elif c.is_alphabetic() or c == '_':
            start = i
            while i < source.len() and (chars[i].is_alphanumeric() or chars[i] == '_'):
                i += 1
            tokens.push(Token.Name((source[start..i]).to_string()))
        elif is_operator(c.to_string()):
            tokens.push(Token.Op(c.to_string()))
            i += 1
        elif c == '(':
            tokens.push(Token.LParen)
            i += 1
        elif c == ')':
            tokens.push(Token.RParen)
            i += 1
        elif c == ',':
            tokens.push(Token.Comma)
            i += 1
        else:
            return Err(LexError(position=i, found=c.to_string()))
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
                println(f"  {[show(t) for t in tokens].join(" ")}")
                names = 0
                for t in tokens:
                    if Name(_) = t:
                        names += 1
                println(f"  {tokens.len()} tokens, {names} names")
            Err(e):
                println(f"  error at column {e.position}: unexpected '{e.found}'")
                println(f"  {line}")
                println(f"  {" ".repeat(e.position)}^")

# A reverse Polish notation calculator with a value stack and errors for bad tokens, division by zero and underflow.

enum CalcError:
    StackUnderflow(op: String)
    UnknownToken(token: String)
    DivisionByZero
    LeftoverValues(count: int)

class Calculator:
    stack: Array[float] = []

    fn pop(op: StrView) -> Result[float, CalcError]:
        match this.stack.pop():
            Some(v) => return Ok(v)
            None => return Err(CalcError.StackUnderflow(op.to_string()))

    fn apply(op: StrView) -> Result[(), CalcError]:
        b = this.pop(op)?
        a = this.pop(op)?
        if op == "/" and b == 0.0:
            return Err(CalcError.DivisionByZero)
        result = match op:
            "+" => a + b
            "-" => a - b
            "*" => a * b
            "/" => a / b
            "^" => a ** b
            _ => return Err(CalcError.UnknownToken(op.to_string()))
        this.stack.push(result)
        return Ok(())

    fn evaluate(line: StrView) -> Result[float, CalcError]:
        this.stack.clear()
        for token in line.split_whitespace():
            if token in ["+", "-", "*", "/", "^"]:
                this.apply(token)?
            elif token == "dup":
                v = this.pop("dup")?
                this.stack.push(v)
                this.stack.push(v)
            elif token == "swap":
                b = this.pop("swap")?
                a = this.pop("swap")?
                this.stack.push(b)
                this.stack.push(a)
            else:
                match token.parse[float]():
                    Ok(v) => this.stack.push(v)
                    Err(_) => return Err(CalcError.UnknownToken(token.to_string()))
        if this.stack.len() != 1:
            return Err(CalcError.LeftoverValues(this.stack.len()))
        return Ok(this.stack[0])

fn describe(e: CalcError) -> String:
    return match e:
        StackUnderflow(op) => f"not enough values for '{op}'"
        UnknownToken(t) => f"unknown token '{t}'"
        DivisionByZero => "division by zero"
        LeftoverValues(n) => f"{n} values left on the stack"

fn main():
    calc = Calculator()
    programs = [
        "3 4 +",
        "5 1 2 + 4 * + 3 -",
        "2 3 ^ dup *",
        "1 0 /",
        "1 +",
        "4 2 swap -",
        "7 x *",
        "1 2 3",
        "10 4 / 0.5 *",
    ]
    ok = 0
    for program in programs:
        match calc.evaluate(program):
            Ok(v):
                ok += 1
                println(f"{program:<22} = {v}")
            Err(e):
                println(f"{program:<22} ! {describe(e)}")
    println(f"{ok} of {programs.len()} programs evaluated")

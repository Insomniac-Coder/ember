# A reverse Polish notation calculator with a value stack and errors for bad tokens, division by zero and underflow.

enum CalcError:
    StackUnderflow(op: String)
    UnknownToken(token: String)
    DivisionByZero
    LeftoverValues(count: int)

class Calculator:
    stack: Array[float] = []

    fn pop(self, op: str) -> Result[float, CalcError]:
        match self.stack.pop():
            Some(v) => return Ok(v)
            None => return Err(CalcError.StackUnderflow(op.to_string()))

    fn apply(self, op: str) -> Result[(), CalcError]:
        b = self.pop(op)?
        a = self.pop(op)?
        if op == "/" and b == 0.0:
            return Err(CalcError.DivisionByZero)
        result = 0.0
        if op == "+":
            result = a + b
        elif op == "-":
            result = a - b
        elif op == "*":
            result = a * b
        elif op == "/":
            result = a / b
        elif op == "^":
            result = a ** b
        else:
            return Err(CalcError.UnknownToken(op.to_string()))
        self.stack.push(result)
        return Ok(())

    fn evaluate(self, line: str) -> Result[float, CalcError]:
        self.stack.clear()
        for token in line.split_whitespace():
            if token in ["+", "-", "*", "/", "^"]:
                self.apply(token)?
            elif token == "dup":
                v = self.pop("dup")?
                self.stack.push(v)
                self.stack.push(v)
            elif token == "swap":
                b = self.pop("swap")?
                a = self.pop("swap")?
                self.stack.push(b)
                self.stack.push(a)
            else:
                match token.parse[float]():
                    Ok(v) => self.stack.push(v)
                    Err(_) => return Err(CalcError.UnknownToken(token.to_string()))
        if self.stack.len() != 1:
            return Err(CalcError.LeftoverValues(self.stack.len()))
        return Ok(self.stack[0])

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

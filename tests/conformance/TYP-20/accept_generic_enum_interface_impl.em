#$ test: run-pass
#$ rules: TYP-16, TYP-17, TYP-20, ENM-1, MONO-1
#$ stdout: 42

interface Render:
    fn render() -> i32

enum Message[T] implements Render:
    Value(value: T)

    fn render() -> i32:
        return 42

[T: needs Render]
fn call_render(value: T) -> i32:
    return value.render()

fn main():
    message = Message[i32].Value(0)
    println(call_render(message))

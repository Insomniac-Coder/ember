interface Source:
    type Item
    fn get(self) -> Result[int, int]

fn through[S: Source[Item = int]](s: S) -> Result[int, int]:
    return s.get()

fn main():
    pass

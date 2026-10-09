interface Source:
    type Item
    fn get(self) -> Result[Item, int]

fn through[S: Source](s: S) -> Result[S.Item, int]:
    return s.get()

fn main():
    pass

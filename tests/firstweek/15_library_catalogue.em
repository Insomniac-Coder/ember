# A library catalogue: add books, search by title or author, and lend and return copies with due dates and limits.

struct Book:
    isbn: String
    title: String
    author: String
    year: int
    copies: int = 1

struct Loan:
    isbn: String
    member: String
    due_day: int

enum LendError:
    NoSuchBook
    NoCopiesLeft(title: String)
    TooManyLoans(member: String, limit: int)

const LOAN_DAYS: int = 14
const MAX_LOANS: int = 2

class Library:
    books: Map[String, Book] = {}
    loans: Array[Loan] = []
    today: int = 0

    fn add(self, owned book: Book):
        if book.isbn in self.books:
            self.books[book.isbn].copies += book.copies
        else:
            self.books[book.isbn] = book

    fn search(self, text: str) -> Array[Book]:
        needle = text.lower()
        return [b for b in self.books.values() if needle in b.title.lower() or needle in b.author.lower()]

    fn available(self, isbn: str) -> int:
        lent = self.loans.iter().filter(fn(l) => l.isbn == isbn).count()
        return self.books[isbn].copies - lent

    fn lend(self, isbn: str, member: str) -> Result[Loan, LendError]:
        if isbn not in self.books:
            return Err(LendError.NoSuchBook)
        if self.available(isbn) == 0:
            return Err(LendError.NoCopiesLeft(self.books[isbn].title))
        held = [l for l in self.loans if l.member == member]
        if held.len() >= MAX_LOANS:
            return Err(LendError.TooManyLoans(member, MAX_LOANS))
        loan = Loan(isbn=isbn, member=member, due_day=self.today + LOAN_DAYS)
        self.loans.push(loan)
        return Ok(loan)

    fn give_back(self, isbn: str, member: str) -> Option[int]:
        for i, loan in self.loans.iter().enumerate():
            if loan.isbn == isbn and loan.member == member:
                late = self.today - loan.due_day
                self.loans.remove(i)
                return Some(max(late, 0))
        return None

fn main():
    lib = Library()
    lib.add(Book(isbn="111", title="The Silent Forest", author="M. Reyes", year=1998))
    lib.add(Book(isbn="222", title="Cooking with Fire", author="J. Okafor", year=2011, copies=2))
    lib.add(Book(isbn="333", title="Forest Birds", author="A. Lindqvist", year=2005))
    lib.add(Book(isbn="111", title="The Silent Forest", author="M. Reyes", year=1998))

    println("search 'forest':")
    for b in lib.search("forest"):
        println(f"  {b.title} by {b.author} ({b.year}), {lib.available(b.isbn)} available")

    requests = [("111", "kim"), ("111", "lee"), ("111", "max"), ("222", "kim"), ("333", "kim"), ("999", "lee")]
    for isbn, member in requests:
        match lib.lend(isbn, member):
            Ok(loan) => println(f"{member} borrows {isbn}, due on day {loan.due_day}")
            Err(NoSuchBook) => println(f"{member}: there is no book {isbn}")
            Err(NoCopiesLeft(title)) => println(f"{member}: all copies of '{title}' are out")
            Err(TooManyLoans(who, limit)) => println(f"{who} already has {limit} books")

    lib.today = 20
    match lib.give_back("111", "kim"):
        Some(0) => println("kim returned the book on time")
        Some(days) => println(f"kim is {days} days late")
        None => println("kim had no such loan")
    println(f"copies of 111 available now: {lib.available("111")}")

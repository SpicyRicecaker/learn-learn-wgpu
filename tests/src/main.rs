fn main() {
    let bookDescriptor = BookDescriptor { title: "123", content: "123", pages: 123 };

    // let book = Book{&bookDescriptor};
}

struct Book<'a> {
    bookDescriptor: &'a BookDescriptor
}

struct BookDescriptor<'a> {
    title: &'a str,
    content: &'a str,
    pages: u32,
}

enum NumberChoice {
    One(u32),
    Two(u32),
}

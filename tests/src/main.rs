fn main() {
    let number = 12345;

    match number {
        123123 => {
            println!("no")
        }
        2 => {
            println!("yess")
        }
        _ => (),
    }
}

struct book<'a> {
    title: &'a str,
    content: &'a str,
    pages: u32
}
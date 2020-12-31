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

    let num = NumberChoice::One(12345);

    match num {
        NumberChoice::One(32) => {}
        NumberChoice::Two(_) => {}
        _ => (),
    }
    
    let mut num = 2;
    {
        num = 123
    }
    println!("{}", num);
}

struct book<'a> {
    title: &'a str,
    content: &'a str,
    pages: u32,
}

enum NumberChoice {
    One(u32),
    Two(u32),
}

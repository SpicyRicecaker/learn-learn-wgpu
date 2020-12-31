use wgpu::BufferCopyViewBase;

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
    
    let num = Boption::Some(123);
}

struct book<'a> {
    title: &'a str,
    content: &'a str,
    pages: u32
}

enum Boption<T> {
    Some(T),
    None
}
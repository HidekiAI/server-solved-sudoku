mod cell;

struct Cell
{
    u8 Value;
    u8 Row;
    u8 Col;
}

impl Cell {
 fn Set(&self) {
   println!(“baf!”);
 }
}

fn greet() -> String {
    "Hello, world!".to_string()
}

#[cfg(test)] // Only compiles when running tests
mod tests {
    use super::greet; // Import root greet function

    #[test]
    fn test_greet() {
        assert_eq!("Hello, world!", greet());
    }
}
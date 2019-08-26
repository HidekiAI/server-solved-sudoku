mod cell;

struct Cell
{
    u8 Value;
    u8 Row;
    u8 Col;
}

impl Cell {
 fn Set(&self) {
     Row = self.Row;
     Col = self.Col;
     match self.Value ->
        | > 9 -> fail;
        | v -> Value() = self.Value;
 }
}

fn Row() -> u8 {
    self.Row;
}

fn Col() -> u8 {
    self.Col;
}

fn Value() -> u8 {
    self.Value;
}

#[cfg(test)] // Only compiles when running tests
mod tests {
    use super::greet; // Import root greet function
    x

    #[test]
    fn test_greet() {
        assert_eq!("Hello, world!", greet());
    }
}
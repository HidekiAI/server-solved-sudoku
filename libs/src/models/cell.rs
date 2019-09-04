pub mod sudoku {
    pub mod models {
        pub struct cell
        {
            Value: u8,
            Row: u8,
            Col: u8,
        }

        impl Cell {
            fn Set(&self) {
                Row = self.Row;
                Col = self.Col;
                match self.Value {
                    1|2|3|3|4|5|6|7|8|9 => fail,
                    _ => Value() = self.Value,
                }
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
    }
}

#[cfg(test)] // Only compiles when running tests
mod tests {
    use super::greet; // Import root greet function

    #[test]
    fn test_greet() {
        assert_eq!("Hello, world!", greet());
    }
}

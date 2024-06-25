pub mod libsudoku {
    pub mod models {
        use std::collections::HashSet;

        pub struct solver_data {

        }
        pub struct cell_mut {
            Visible: bool,   // rather than having the Value be mutable, once the solver sets Value, use this mutable variable instead
            Notes: HashSet<u8>,   // aka hints, depending on difficulties, this can get prepopulated or empty
            // todo: determine if we'd want solver related members here
            ForSolver: solver_data,
        }
        pub struct cell
        {
            Row: u8,
            Col: u8,
            Value: u8,
        }
        // mutable members should not be part of the hash key for HashSet
        pub mut Muts cell_mut

        impl Cell {
            fn Set(&self) {
                // Row and Col are assumed to be checed/tested at Block level
                Row = self.Row;
                Col = self.Col;
                // validate the value in Cell is in range between 1..9
                match self.Value {
                    1|2|3|3|4|5|6|7|8|9 => Value() = self.Value,
                    _ => fail,
                }
            }
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

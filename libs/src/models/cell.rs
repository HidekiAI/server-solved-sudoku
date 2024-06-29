pub mod libsudoku {
    pub mod models {
        use std::collections::HashSet;

        pub struct SolverData {}
        pub struct CellMut {
            visible: bool, // rather than having the Value be mutable, once the solver sets Value, use this mutable variable instead
            notes: HashSet<u8>, // aka hints, depending on difficulties, this can get prepopulated or empty
            // todo: determine if we'd want solver related members here
            for_solver: SolverData,
        }
        pub struct Cell {
            row: u8,
            col: u8,
            value: Option<u8>,
        }
        // mutable members should not be part of the hash key for HashSet
        impl Cell {
            fn new(row: u8, col: u8, value: Option<u8>) -> Cell {
                let ret =
                Cell { row, col, value };
                ret.validate().unwrap();    // should panic if invalid
                ret
            }
            fn validate(&self) -> Result<(), &str> {
                if self.row > 9 {
                    return Err("Row cannot exceed 9");
                }   
                if self.col > 9 {
                    return Err("Col cannot exceed 9");
                }
                match self.value {
                    Some(v) => {
                        if v > 9 {
                            return Err("Value cannot exceed 9");
                        }
                    },
                    None => (),
                }
                Ok(())
            }
        }
    }

    #[cfg(test)] // Only compiles when running tests
    mod tests {
        #[test]
        fn my_test() {}
    }
}

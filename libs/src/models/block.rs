 // use Set to assure unique cells per block

pub mod libsudoku {
    pub mod models {
        use std::collections::HashSet;

        use crate::models::cell::{libsudoku::models::Cell};

        pub struct Block {
            cells: HashSet<HashSet<Cell>>, // each row (or column) must be unique
            width: u8,                     // usually 3 x 3 cells to a block
            height: u8,
        }
        impl Block {
            fn new(cells: HashSet<HashSet<Cell>>, width: u8, height: u8) -> Block {
                let ret = Block {
                    cells,
                    width,
                    height,
                };
                ret.validate().unwrap(); // should panic if invalid
                ret
            }
            fn validate(&self) -> Result<(), &str> {
                if self.width > 3 {
                    return Err("Width cannot exceed 3");
                }
                if self.height > 3 {
                    return Err("Height cannot exceed 3");
                }
                Ok(())
            }
        }
    }
}

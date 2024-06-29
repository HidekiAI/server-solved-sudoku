use std::collections::HashSet;

pub mod libsudoku {
    pub mod models {
        use std::collections::HashSet;

        use crate::models::block::{self, libsudoku::models::Block};
        use crate::models::cell::{self, libsudoku::models::Cell};

        pub struct Board {
            blocks: HashSet<Block>, // Q: can there be chances where two (or more) blocks are identical?
        }
    }
}

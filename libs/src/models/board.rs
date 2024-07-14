
pub mod libsudoku {
    pub mod models {
        use std::collections::HashSet;

        use crate::models::block::{libsudoku::models::Block};
        

        pub struct Board {
            blocks: HashSet<Block>, // Q: can there be chances where two (or more) blocks are identical?
        }
    }
}

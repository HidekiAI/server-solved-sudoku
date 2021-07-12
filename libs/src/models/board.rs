use std::collections::HashSet;

mod models {
    pub struct board {
        Blocks: HashSet<block>, // Q: can there be chances where two (or more) blocks are identical?
    }
}

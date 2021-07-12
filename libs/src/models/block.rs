use std::collections::HashSet;  // use Set to assure unique cells per block

mod models {
    pub struct block {
        Cells: HashSet<HashSet<cell>>   // each row (or column) must be unique
        Width: u8   // usually 3 x 3 cells to a block
        Height: u8
    }
}

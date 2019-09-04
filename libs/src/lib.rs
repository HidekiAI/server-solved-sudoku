use std::io::ErrorKind;

mod sudoku {
    fn check_cell (cell_value: models::cell) -> Result<bool, Err> {
        if cell_value.Value > 9 {
            Err("Call.Value cannot exceed value of 9 on a 3x3");
        }
        true;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

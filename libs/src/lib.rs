pub mod models;
pub mod generators;
pub mod solvers;

pub mod libsudoku {
    #[path="models/cell.rs"]

    fn check_cell(cell_value: cell) -> Result<bool, string> {
        if cell_value.Value > 9 {
            Err("Call.Value cannot exceed value of 9 on a 3x3");
        }
        Ok(true);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}


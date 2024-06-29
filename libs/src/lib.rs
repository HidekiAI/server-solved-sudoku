pub mod generators;
pub mod models;
pub mod solvers;

pub mod libsudoku {
    use crate::models::cell::{self, libsudoku::models::Cell};

}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

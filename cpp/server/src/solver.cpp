#include "solver.hpp"


namespace Sudoku
{
    Solver::Solver()
    {
        auto myCell = DataModels::Cell<int64_t>(std::nullopt, std::nullopt, std::vector<int64_t>(1));
    }
}



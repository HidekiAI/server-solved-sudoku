#include <iostream>
#include <thread>
#include <vector>
#include "../../protobuf/generated/route.pb.h"

namespace Sudoku
{
    namespace DataModels
    {
        template <typename T>
        class Cell final
        {
            public:
            Cell(std::optional<uint32_t> width, std::optional<uint32_t> height, const std::vector<T>& data)
            {
                possible_width = width;
                possible_height = height;
                cell_data = std::move(data);
            }
            Cell(const Router::Cell& cell)
            {
                auto possible_width = cell.width_count();
                auto possible_height = cell.height_count();
                for (const auto& d : cell.data())
                {
                    cell_data.add(d);
                };
            }

            private:
            std::optional<uint32_t> possible_width;
            std::optional<uint32_t> possible_height;
            std::vector<T> cell_data;
        };

        template <typename T>
        class Row final
        {
            public:
            Row(const std::vector<Cell<T>>& cells)
            {
                cells = std::move(cells);
            }

            private:
            std::vector<Cell<T>> cells;
        };

        template <typename T>
        class Grid final
        {
            public:
            Grid(const std::vector<Row<T>>& rows)
            {
                rows = std::move(rows);
            }

            private:
            std::vector<Row<T>> rows;
        };

        template<typename T>
        class Game final
        {
            public:
            Game(const std::vector<Grid<T>>& grids)
            {
                grids = std::move(grids);
            }
            private:
            std::vector<Grid<T>> grids = grids;
        };
    }
}
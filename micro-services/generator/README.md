# Sudoku Game Generator

My original intention of a generators was to randomly populate a 9x9 rmatrix, with each 3x3 to have unique digits of 1..9, and at the same time, make sure that the horzontal and vertical does not have same duplicate digits, then randomly remove (erase) cells until there are only 8 clues left on the board, call it "medium" and be done with it.

Apparently, "...every Sudoku puzzle with 16 or fewer clues has multiple valid solutions."[^1][^2], though further readings says that even with 17 or more clues, there can be multiple solutions.  In any case, we'll leave that up to the resolver.  For generators, all we care about is that there is AT LEAST ONE solution.

[^1]: [Can Sudoku Have Multiple Solutions?](https://masteringsudoku.com/can-sudoku-have-multiple-solutions/)
[^2]: [Mathematicians Solve Minimum Sudoku Problem](https://www.technologyreview.com/2012/01/06/188520/mathematicians-solve-minimum-sudoku-problem/)

## Generator

If there is a database of existing sudoku puzzles all catagorized for easy, medium, and hard, that has at least one solution, I prefer just polling from that database and using that pregenerated data.  As a starter, just to keep this project simple as possible, and because Generator is a micro-service of its own, it's completely decoupled and can be revisted in the future for better puzzle generator.  But for now, all I'll provide is about 10 pre-solved 9x9 sudoku puzzles, and the generator will just randomly pick one of them and randomly remove cells until there are only 8 clues left on the board, call it "medium" and be done with it.

Note that most likely, these pre-solved puzzles will be generated from [qqwing](https://qqwing.com/download.html) since it's available for Debian as dpkg.

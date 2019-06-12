using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;

public sealed class Block
{
    public Block(HashSet<Cell> cells, UInt16 row, UInt16 col)
    {
        Cells = cells;
        Row = row;
        Column = col;
        if (Row > 2)
        {
            throw new ArgumentOutOfRangeException("row", $"Row: {row} out of range");
        }
        if (Column > 2)
        {
            throw new ArgumentOutOfRangeException("col", $"Column: {col} out of range");
        }
    }
    public readonly HashSet<Cell> Cells;
    public readonly UInt16 Row;
    public readonly UInt16 Column;
}

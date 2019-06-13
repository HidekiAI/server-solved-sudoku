using System.Collections.Generic;
using System;
using System.Collections;
using System.Linq;
using LaYumba.Functional;

public sealed class Cell : IComparable, IEqualityComparer
{
    public Cell(Option<UInt16> value, UInt16 row, UInt16 col)
    {
        Row = row;
        Column = col;
        Value = value;
        if (Value.IsSome() && Value.value > 9)
        {
            throw new ArgumentOutOfRangeException("value", $"Value: {value} is out of range");
        }
        if (Row > 2)
        {
            throw new ArgumentOutOfRangeException("row", $"Row: {row} out of range");
        }
        if (Column > 2)
        {
            throw new ArgumentOutOfRangeException("col", $"Column: {col} out of range");
        }
    }
    public readonly Option<UInt16> Value;
    public readonly UInt16 Row;
    public readonly UInt16 Column;

    public bool PositionEquals(Cell other) =>
        Column == other.Column && Row == other.Row;

    public int CompareTo(object obj) =>
        obj == null
            ? int.MinValue
            : obj.GetType() != typeof(Cell)
                ? int.MinValue
                : Value.HasValue == false || (obj as Cell).Value.HasValue == false
                    ? int.MinValue
                    : Value.Value - (obj as Cell).Value;

    public new bool Equals(object x, object y)
    {
        if ((x == null) || (y == null))
        {
            return false;
        }
        if ((x.GetType() != typeof(Cell)) || (y.GetType() != typeof(Cell)))
        {
            return false;
        }
        var lhs = x as Cell;
        var rhs = y as Cell;
        return lhs.Value == rhs.Value;
    }

    public int GetHashCode(object obj) =>
        obj == null
            ? 0
            : typeof(Cell) != obj.GetType()
                ? 0
                : (obj as Cell).Value.GetHashCode();
}

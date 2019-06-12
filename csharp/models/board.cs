using System.Collections.Generic;
using System;
using System.Collections;
using System.Linq;

public sealed class Board
{
    public Board(Block[] blocks)
    {
        Blocks = blocks;
    }
    public readonly Block[] Blocks;
}

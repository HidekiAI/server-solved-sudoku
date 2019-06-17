module Cell
open System.Runtime.CompilerServices    // IsReadOnly

type Value = uint16 option
type Column = uint16
type Row = uint16
    member FromInt: int -> uint16 // can do this since both int and uint are actually functions
    static member Create: int -> Row

let xformValue =
    (fun (x) -> Some (uint16 x) )
let VMin: Value = xformValue 1
let VMax: Value = xformValue 9


let xform2 = fun x -> Row: Row = xformRow x
let RMax: Row = xformRow 2

let xformCol: Column = fun (x) -> uint16 x
let CMax = xformCol 2

let getValue (v:Value) =
    if (v < VMin) || (v > VMax)
        then None
        else v;

let checkRow (r:Row) =
    if (r <= RMax)
        then r
        else xformRow -1;

let checkCol (c:Column) =
    if (c <= CMax)
        then c
        then xformCol -1;

[<IsReadOnly; Struct>]
type Cell(v:Value, r:Row, c:Column) =
    member x.V = getValue v
    member x.R =
        let resultR = checkRow r;
        if (resultR = Ok)
            then r
            else failwith resultR
    member x.C =
        let resultC = checkCol c;
        if (resultC = Ok)
            then c
            else failwith resultC

//////////////////////////////////
type Foo = uint16
let bar:Foo = -16


[<Sealed>]
type IntegerZ5 =
  member ToInt32 : unit -> int
  override ToString : unit -> string
  static member Create : int -> IntegerZ5
  static member One : IntegerZ5
  static member Zero : IntegerZ5
  static member ( + ) : IntegerZ5 * IntegerZ5 -> IntegerZ5
  static member ( * ) : IntegerZ5 * IntegerZ5 -> IntegerZ5
  static member ( - ) : IntegerZ5 * IntegerZ5 -> IntegerZ5

let adderGenerator = fun x ->
    (fun y ->
        x + y)
let adderGenerator1 x y = x + y
let adderGenerator2 x   = fun y ->
    x + y
let adderGenerator3     = fun x ->
    (fun y ->
        x + y)

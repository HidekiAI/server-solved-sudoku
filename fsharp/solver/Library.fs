namespace solver

module BinaryTree

open TypeShape
open TypeShape.Core.Core
open System

type MyBinaryTreeNode<'T> =
    {
        Value : 'T
        Left : MyBinaryTreeNode<'T>
        Right: MyBinaryTreeNode<'T>
    }
type MyBinaryTree<'T> =
    {
        Root : MyBinaryTreeNode<'T>
        Height: int // 2^n is the width
    }


let tryParseDateTime (s: string) =
    match System.DateTime.TryParse(s) with
    | (true, dt) -> ValueSome dt
    | (false, _) -> ValueNone

let possibleDateString1 = "1990-12-25"
let possibleDateString2 = "This is not a date"

let result1 = tryParseDateTime possibleDateString1
let result2 = tryParseDateTime possibleDateString2

match (result1, result2) with
| ValueSome d1, ValueSome d2 -> printfn "Both are dates!"
| ValueSome d1, ValueNone -> printfn "Only the first is a date!"
| ValueNone, ValueSome d2 -> printfn "Only the second is a date!"
| ValueNone, ValueNone -> printfn "None of them are dates!"

// tryParse :: string -> 'T option
type tryParse<'T> = string -> 'T option

// tryParse :: string -> 'T option
let tryParse<'T> (S : string) =
    // switch typeof(T)
    match shapeof<'T> with
        // case Int32: // string -> ( Int32.TryParse S -> (bool, Int32) ) -> 
        | Shape.Int32 -> (bool, (System.Int32.TryParse S)) option
        // case Double: // string -> Some Double
        | Shape.Double -> (bool, (System.Double.TryParse S)) option
        // case Int64: // string -> Some Int64
        | Shape.Int64 -> (bool, (System.Int64.TryParse S)) option
        // default: // string -> None
        | _ -> None

let toArray<'T> (S : string) =
    S.Split [|' '|]
    |> Seq.map (fun s -> match tryParse s with
        | Some obj -> (s, Some obj)
        | _ -> (s, None))

let buildTree<'T> (a : array<'T>) =
    null

// > MyTests.main [|"1"; "2"; "3"|] ;;
// Hello World from F#!
// val it : int = 0



module PigLatin

module PigLatin =
    let toPigLatin (word: string) =
        let isVowel (c: char) =
            match c with
            | 'a' | 'e' | 'i' |'o' |'u' | 'A' | 'E' | 'I' | 'O' | 'U' -> true
            |_a1 -> false

        if isVowel word.[0] then
            word + "yay"
        else
            word.[1..] + string(word.[0]) + "ay"

module Say =
    let hello name =
        printfn "Hello %s" name



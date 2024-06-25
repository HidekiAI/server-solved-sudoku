# Sudoku Server-Client libraries

Firstly, I am a biased towards data-contract model, where I'd write libraries based on how the data is shaped, or how to reshape that data.  Hence, I tend to design/define data_models/data-structures first and provide API's for that data-model to reshape to another data-model or to take actions based on that data model[^1] (compared to what?  Well, those functional-programmers loves type-inferences (hence "functional") :confused: [^2]).  So in most cases, technical documentations and implementations begins by discussing the shape of the data-model (mainly, using Protobuf for this project[^3]).

All in all, preferably, libraries should be small, almost to a point where if possible, eliminate it kind of approach, and avoid code-sharing and use and/or pass lambdas/functions/delegates (locally declare lambdas within the function and only visible to that function, and/or pass lambdas to another function).  But unfortunately, there are some some shared libraries, in particular all the data-models generated from Protobuf targetting Rust (via gRPC).

Other than that, other shared logics are commonly [traits](https://doc.rust-lang.org/reference/items/traits.html) `#[derive()]` with base `impl`, etc.  As much as possible, keep code/logic/methods local to each module, and if you discover that it needs to be used for other modules, rather than copy-and-paste, make it into a `trait` and move it to the library side.  The exception is lambdas.  If you catch yourself copying lambdas from elsewhere and pasting it, so be it, don't try to be clever and make a common/shared lambdas.  

Though it now becomes more philosophical (code religion?) than actual library design, IMHO lambdas should be small, short-and-sweet, which should be easy to understand, maintain, and spot bugs.  If the lambda is taking up more than 1/2 (maybe even 1/3) of the screen tall, thae lambda should be refactored and broken down and separated into 2 (or more) lambdas in which lambdas to call lambda.  Nested lambdas are (sometimes) elegant to see, I like it a lot, but it's super hard for people other than the person who write it to read!  Especially because people tend to make variables short and meaningless when they start nesting lambdas!  While I'm at it, though I really think recurssions are elegant, unless that lambda is very short/small, don't make it a recursive lambda (especially functional programmers tend to love recurssions).  I think that's one caveat about functional programmers, IMHO it's not a team language, only one person, the person who wrote it, comprehends the internal implementations, and will argue that only thing that matters is the signature (input and output).  In any case, I've once researched to see how many large-scale projects were written in functional programming language, and how many programmers were concurrently working on it, and I've found very little.  Functional programming languages are great, but probably not ideal for large-scale projects that involved/involves multiple programmers in which programmers who did not write the code has to debug INTO somebody elses functions.  Also, functional programmers rarely tend to write unit-tests, mainly because the signature itself defines its purposes or something, I don't know...  I give up...   Maybe in the future, when a functional programming language is invented that performs as fast as C/C++ or Rust on a medium and large-scale project, I'll revisit this topic... :raising_hand:  Incidentally, is that debate of porting Quake to Managed C++ caused 15% slowdown, still a thing?  Are people still debating over performance of C/C++ games versus C# games?  Are people still arguing that 32-bit code is faster than 64-bit? :hear_no_evil:

[^1]: I'm sure some will argue [Programs = Algorithms + Data Structures](https://en.wikipedia.org/wiki/Algorithms_%2B_Data_Structures_%3D_Programs), but to me, I've come to think more that data is more concrete and algorithms are more fluid and adaptive to shape of data.  Commonly (or in the past), we have to write separate tools to reshape the data to fit the algorithm and make the program work.  And at times, we have to bend-over backwards on reshaping that data, and in some cases, fill the empty or unknown slot with default values (i.e. even `Option:None` is considered default values).  It tends to generate more buggy algorithms and/or programs IMHO.  Transforming and reshaping based on versioned data also causes nightmare, why not treat it as one transaction to another (i.e. suppose a country is changing its currency, which is more difficult in the long run: (a) taking both old and new currency until all old currency are out of circulations at stores and banks, or (b) have each persons go to the bank and exchange to new currency, and then use the new currency at the stores (stores will not accept old currency)?).
[^2]: For the record, I love F# (I did not like Haskel), and I also love some of the functional feels and aspects of Rust which I cannot live without!
[^3]: I like Protobuf (reduces ambiguity, expresses data language-agnostically) in a documentation over JSON, mainly because JSON will not be able to express quirky part of data-models such as whether the field is required or not (it requires 2 data, one with the field, one with absent, in which, in JSON, absent is treated as NULL/None - but if it is missing, how sure am I that it's optional/not-required or is a bug/typo?), or whether the field is a list or dictionary if it is empty `[]` (can you tell me if that is array or list of KVP?), or whether a string .  I prefer XML Schemas as it can define type-strictness, or actual Rust (or modern C++ with [optional](https://en.cppreference.com/w/cpp/utility/optional)) code `struct` for data-models (if C# had `Option<T>` as built-in type, perhaps even C#).  But in general rather than being picky about one language to another, Protobuf3 has [Optional](https://github.com/protocolbuffers/protobuf/releases/tag/v3.15.0) (on 3.15+) so it's just simpler to just talk in Protobuf and remain language-agnostic.

## Protobuf and gRPC

Firstly, because I'm 100% Rust on this project, I'm using [tonic](https://github.com/hyperium/tonic) for gRPC tied to [Protobuf](https://protobuf.dev/) via [tokio prost](https://github.com/tokio-rs/prost).  I'm still grasping the tool-chain, but one thing is for sure, I'm not calling [protoc](https://protobuf.dev/reference/cpp/cpp-generated/) (kind of jealous that even [Go](https://protobuf.dev/getting-started/gotutorial/) is supported by `protoc` and Rust isn't).  I'm not sure if I'm using using [prost-build](https://github.com/tokio-rs/prost/tree/master/prost-build) to generate Rust code from Protobuf via `tonic` but their documentations says it's relying on "tokio stack", but I'm sure it's somehow intertwined.  I'll just trust the build-chain, as long as the [build-dependencies](https://github.com/hyperium/tonic/blob/master/tonic-build/README.md) in my [Cargo.toml](./Cargo.toml) is correct:

```yaml
[build-dependencies]
tonic-build = "0.11"

[dependencies]
tonic = "0.11"
prost = "0.12"
tokio = { version = "1.0", features = ["full", "macros", "rt-multi-thread"] }
```

And setup my [build.rs](./build.rs) to generate Rust code from Protobuf:

```rust
fn main() {
    tonic_build::configure()
       .build_server(true)
       .build_client(true)
       .out_dir("src/")
       .compile(
            &["protobuf/route.proto"],
            &["protobuf"],
        )
       .unwrap();
}
```

In a way, this is much more easier than C++...

### I/O versus Stack

As much as I dislike redundancies, there is a separation between Protobuf data-model and Rust `struct`, but we'll make sure that SerDe (serialize/deserialize) will transparently transform from Protobuf data to Rust `struct` and vice-versa.

At the `serde` level, mainly at `into` traits, we'll make sure that if/when new elements are added or removed, it'll fail to compile.

It is tempting to bypass all this and pass the raw Protobuf data (in the past, I confess I've done it and I am guilty of my laziness, but I'd argue that I was using C++ `placement-new` and implementing fast message-router and it was to avoid resource allocations and deep-copying data and pointing the memory as typeless/generic `void *`, so I claim not-guilty of laziness and only violated possible memory-leak, potential double-free, can have `NULL`-pointer, loosely-typed, bug-proned code-smell misdemeanor which all C++ programmers are immune to), but to assure separations of I/O-based data and stack-based data, there will be redundancies of defining the data-model twice, once in Protobuf and once in Rust.  This way, when unit-testing and mocking/faking/stubbing, it'll be decoupled and agnostic of the I/O-based data-model.  Not only that, but if Protobuf is no longer desired, there will be no need to touch any layer other than RPC layer.

## Servers

There are libraries that are shared between servers-to-servers (S2S).

## Client simulator vs Unit-test

The libraries that are written between client-to-server (C2S) is probably not as common so there are probably only small amount (probably none) of libraries for it.  But libraries that are shared between client simultors, different kinds of clients, or even unit-test mocking clilents, are probably more common (shared).

In preference and over experiences, unit-test is preferred over client-simulators basically because of maintenance nightmares.  I also believe that if you can write simple mock/fake/stub[^4] on any persisted data based unit-test is more useful because it proves decoupled design as well as deterministic and idempotent behavior of same input should always result to same output.

If client simulators are needed, it should be (though it's a maintenance nightmare) written as a client, not as part of the library module/crate.  It may be useful for long-running tests (i.e. segmentations, implementation validations over data that assumes certain value based on mean and averages, etc.), but for this project, only long/continuous running client is for training.

If client simulator is argued to be needed for end-to-end application testing, I'd argue that backend is designed to not be fault-tolerant and the cause/problem should be identified and fixed.  This is for multiple reasons, but mainly to avoid long period of debugging once the services are written.  More moving parts dependencies, longer it takes to debug, and in some cases, other micro-service will time out if you spend too long at a breakpoint inspecting the stack and local variables.  It should be redesigned so that all you need is a unit-tests for each micro-services to verify that it works as expected, and trust it.  I totally comprehend that this is ideal and not always possible, but keeping this approach in mind will help a long on horizontally scaling the services.

OK, I'm tired of arguing why I dislike simulators...

[^4]: In Rust, there are many ways to do this such as [deterministic unions](https://doc.rust-lang.org/reference/items/unions.html#pattern-matching-on-unions) and [traits](https://doc.rust-lang.org/reference/items/traits.html).
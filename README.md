# solana skeleton
## idiomatic vanilla solana program template

simplest typical solana program is four files:
* `entrypoint.rs`: entrypoint for when the program is compiled to bpf via `cargo build-sbf`. this is ifdeffed out when building with `cargo build` so the types and functions can be imported like a normal library
* `lib.rs`: normal lib file that reexports everything. also reexports `solana_program` by convention so people can version match. program id is declared here with a macro which creates an `id()` function that returns `Pubkey`
* `instruction.rs`: contains an enum for all your instructions for your program, which usually derives borsh serialize/deserialize. sometimes legacy programs use bincode. describe each enum variant with a doc comment with a zero-indexed list of account descriptions and a tag `[]`, `[w]`, `[s]`, or `[s, w]` showing which are writeable or signer. enum variants can have whatever data you need in the processor\
`instruction.rs` also should have helper functions that return `Instruction` or `Vec<Instruction>`
* `processor.rs`: contains a dummy struct called `Processor` with `process_*()` functions for all your instructions. usually these make an iterator from `&[AccountInfo]`, pop off all the accounts at the start, then do as much owner/signer/address/whatever validation as possible up front before any processing. its also common to do validation in helper functions you use to parse account data. the idea here is you want it to be impossible to ever accidentally use an account you havent validated, this feels about as safe as anchor attributes once you get used to it. theres a simple example of how to do this in spl/single-pool and a beautiful and sexy example in spl/governance\
also give `Processor` a function called `process()` of type `&Pubkey -> &[AccountInfo] -> &[u8] -> ProgramResult`. the reason is so you can give it to `ProgramTest` in your tests if you want. its the same type as the entrypoint

odds and ends can go in `lib.rs`, `processor.rs`, or whatever new files you want. its normal to make `state.rs` for onchain data and `error.rs` for custom errors if you want those things. onchain data should be a single enum of structs so you get discriminators for free. its a smart idea to use an enum even if you only have one kind of account in case you make more later. for complicated programs you might wanna split it up into crates so people can just import what they need to parse or cpi without pulling in everything

for simple integration tests you can do something like this:

```rust
let mut program_test = ProgramTest::default();
program_test.prefer_bpf(false);

program_test.add_program("skeleton", id(), processor!(Processor::process));

program_test.start_with_context().await
```

this returns `ProgramTestContext` which gives you `BanksClient`, payer, and latest blockhash. basically its the state transition stuff from the validator without all the validation. its fast you can use it a lot in parallel. the code block uses the processor function directly without relying on bpf but if you dont do `prefer_bpf(false)` itll look in `target/deploy/` for the binary. you should use bpf if you do syscalls

for big complicated integration tests look in `tests/tests.rs`. this stands up what is essentially a full `solana-test-validator` actually listening on `localhost:8899` doing all the real validator things. its especially good for testing cli apps because you can test by just using the cli normally. this is slow tho so test in serial

pls dont pr this repo i just made it in an hour because i wanted to test an exploit i thought of (it didnt work)

Thanks for contributing to `Plotters`! 

Here's the useful information about contributing to Plotters.

# License

`Plotters` is under MIT license currently. 
You may intesrested in reading [the full text of license](https://github.com/38/plotters/blob/master/LICENSE).
If you have any questions or concerns please contact us at <haohou302@gmail.com>.

# Contributing Code

You are warmly welcomed to contribute code and make Plotters better. Here's a few things that may be helpful to you.

## How to make sure my code works ? 

You may realize that `Plotters` doesn't have a high testing coverage, but we are trying hard to improve. 
It would be nice if you add more test cases for newly added code, or contribute new test cases directly. 

Before you finalize your PR, please check the following thing:

- Please use `cargo test` to make sure all test case passes. If any case fails, we need to dig into that.

- Please run the benchmark with `cargo bench` to check if the performance changed compare to the master branch. 

- Please run the following command to check if the example output changes:

  ```bash
  cargo test --doc
  cargo build --release --examples
  for i in examples/*.rs
  do
  ./target/release/examples/$(basename $i .rs)
  done
  cd plotters-doc-data
  git status
  ```
  And there shouldn't be no change if you are not modifying the layout code.
  
- Please make sure the WASM target works as well. The easiest way to do that is try to run our WASM demo under [examples/wasm-demo](https://github.com/38/plotters/blob/master/examples/wasm-demo) directory and follow the instruction in the `README.md` file under that directory.

## Is my code meets the styling guideline ?

- In general, the only guide line is we need to make sure `cargo fmt` doesn't change anything. 
So it's recommended use `cargo fmt` to fix the code styling issues before you wrap up the work. (Such as start a pull reuqest)
- For naming, acronyms or initials aren't normally used in the code base. Descriptive identifier is highly recommended.
- Documentation is highly recommended. (But there are still a lot of undocumented code unfortunately). 
- For API documentation, we normally follows doxygen's style, which looks like
```rust
/// Some description to this API
/// - `param_1`: What param_1 do
/// - `param_2`: What param_2 do
/// - **returns**: The return value description
fn foo(param_1: u32, param_2: u32) -> u32{ 0 }d 
```

## Top Level Documentation and Readme

Please notice we put almost same content for top level rustdoc and `README.md`. Thus the both part are gennerated by script.
If you need to modify the readme and documentation, please change the template at [doc-template/readme.template.md](https://github.com/38/plotters/blob/master/doc-template/readme.template.md) and 
use the following command to synchronize the doc to both `src/lib.rs` and `README.md`.

```bash
bash doc-template/update-readme.sh
```


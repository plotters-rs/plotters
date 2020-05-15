# plotters examples

To run any example, from within the repo, run `cargo run --example <example_name>` where `<example name>` is the name of the file without the `.rs` extension.

All the examples assumes the directory [plotters-doc-data](https://github.com/38/plotters-doc-data) exists, otherwise those example crashs.

The output of these example files are used to generate the [plotters-doc-data](https://github.com/38/plotters-doc-data) repo that populates the sample images in the main README.
We also relies on the output of examples to detect protential layout changes.
For that reason, **they must be run with `cargo` from within the repo, or you must change the output filename in the example code to a directory that exists.**

The examples that have their own directories and `Cargo.toml` files work differently. They are run the same way you would a standalone project.

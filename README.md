mini-poml-rs
===
A partial implementation of [POML](https://microsoft.github.io/poml/) purely in Rust.

This project is still in early development. Use at your own risk! The API may change significantly in the near term.

## Features 
### Supported features
* Variables
* `if` / `for` attribute
* `<let>` for assigning values to variables 
* `<include>` to include other files
* Render as Markdown
* `<code>` block
* Expression evaluation
    * Array item and object field access
    * `+` / `-` / `*` / `/` arithmetic operators
    * `!` / `&&` / `||` logical operators

### Features in work
* Expression evaluation

## Run Example
Examples of supported POML files can be found in [supported_poml_docs/](supported_poml_docs/). An example program is 
provided in [examples/](examples/). You can run it with cargo:

```
$ cargo run --example poml_render \
    supported_poml_docs/2_for_loop_on_context/main.poml \
    supported_poml_docs/2_for_loop_on_context/context.json
```


## Copyright
Copyright (c) 2025, mini-poml-rs [authors](AUTHORS). All rights reserved. 

Released under Mozilla Public License 2.0. See [LICENSE](LICENSE).


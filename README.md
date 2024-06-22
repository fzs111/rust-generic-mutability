# rust-generic-mutability

This crate enables the creation of functions, methods, traits or even structs generic over mutability.

The main use case is allowing crates to write pairs of getter functions of the form
```rust
fn get<'a>(&'a T, ...) -> &'a U
fn get_mut<'a>(&'a mut T, ...) -> &'a mut U
```
as a single function
```rust
fn get_gen<'a, M: Mutability>(GenRef<'a, M, T>, ...) -> GenRef<'a, M, U>
```

This project is currently in an **experimental state**. Breaking changes are expected before reaching `1.0.0`. You can start experimenting with it in your own projects. Any feedback is welcome!

## Contributing

If you can, please help with any of the following:

- Review
    - [ ] `unsafe`
        
        This crate relies on `unsafe` to work.

    - [ ] documentation

        If there is anything that is unclear, you can ask me or improve it yourself.

    - [ ] API design

        Feel free to bikeshed some things and point out problems before this goes "production ready"!

- [ ] Tests

    This project doesn't have a lot of tests. If you can write some, I'd greatly appreciate it!

- [ ] `std` interface

    We'll need to create extension traits and functions that make `std` accessible to generic mutability.
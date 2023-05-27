# Experimental generic mutability for Rust on the type level

This project is in a very experimental state, please don't use it in your projects just yet!

`GenRef<'a, M, T>` is the main type of this crate. In many ways, it acts like a `&mut &` reference (it isn't `Copy` and is invariant over `T`), but code that knows the mutability can downcast it to a `&` or `&mut` reference using the `as_immut` and `as_mut` methods.

The backbone of the functionality is provided by the `Mutability` trait (and its implementations for the `Mutable` and `Immutable` empty types). It defines behavior for each type of reference in form of unsafe, pointer-to-pointer functions.

Generic mutability can also be applied to traits, functions and structs. To use it on a struct, add a field of type `PhantomData<*const M> where M: Mutability`.

Currently there's no support (via syntax or macros) for mapping field/method access without code duplication, unless the method itself uses generic mutability.

If you have a usecase for this, are interested in this feature or have some ideas, check out [this](https://internals.rust-lang.org/t/pre-rfc-unify-references-and-make-them-generic-over-mutability/18846?u=fzs) discussion and feel free to open an issue on this repo.

/// This macro simplifies the implementation of generically mutable APIs, where the mutable and the shared code paths are (mostly) identical.
/// It has the following syntax:
///
/// ```rust, ignore
/// gen_mut!{ $M => {
///     /* code */
/// }}
/// ```
/// where `$M` is the name of the generic parameter you want to unwrap.
/// The code inside has access to three macros, which allow it to emit different code depending on the value of `M`:
///
/// - `from_gen!($genref)` / `from_gen!()`
///
///     Calls `GenRef::gen_into_shared` and `GenRef::gen_into_mut` on the `$genref` passed as an argument.
///     The type of return value is different in the shared vs the mutable case, so it is not possible to move the return value outside of the macro call (attempting to do so would run into a type checker error on trying to assign `&mut T` to `&T` or vice versa).
///     The return value can be converted back into a `GenRef` using the `into_gen!` macro.
///     If no arguments are passed, it returns a closure `Fn(GenRef<'_, M, T>) -> &T` / `Fn(GenRef<'_, M, T>) -> &mut T`.
///
/// - `into_mut!($reference)` / `into_mut!(&gen $place)` / `into_mut!()`
///
///     Calls `GenRef::gen_from_shared` and `GenRef::gen_from_mut` on the reference passed as an argument, and returns the resulting `GenRef`.
///     The type of the input is different in the shared vs the mutable case, so it is not possible to call this with a reference that was not created via `from_gen!` or `switch_mut_shared!`.
///     To allow accessing fields, you can use the `into_mut!(&gen $place)` syntax, which references the `$place` expression with the appropriate kind of reference.
///     If no arguments are passed, it returns a closure `Fn(&T) -> GenRef<'_, M, T>` / `Fn(&mut T) -> GenRef<'_, M, T>`.
///
/// - `switch_shared_mut!($shared_expr, $mutable_expr)` / `switch_shared_mut!({ $shared_tts } { $mutable_tts })`
///
///     Expands to `shared_expr` in the shared case and `mutable_expr` in the mutable case.
///     The `switch_shared_mut!({ $shared_tts } { $mutable_tts })` syntax allows you to expand to arbitrary token trees, not just expressions.
///     This requires you to wrap them in brackets, which will not appear in the expansion.
///     Also note that in this syntax there is no comma separating the two cases.

#[macro_export]
macro_rules! gen_mut {
    ($m:ty => $code:expr) => {
        match <$m as $crate::Mutability>::mutability() {
            $crate::MutabilityEnum::Shared(proof) => {
                macro_rules! into_gen {
                    () => {
                        |genref| $crate::GenRef::gen_from_shared(genref, proof)
                    };
                    (&gen $genref:expr) => {
                        $crate::GenRef::gen_from_shared(&$genref, proof)
                    };
                    ($genref:expr) => {
                        $crate::GenRef::gen_from_shared($genref, proof)
                    };
                }
                macro_rules! from_gen {
                    () => {
                        |genref| $crate::GenRef::gen_into_shared(genref, proof)
                    };
                    ($reference:expr) => {
                        $crate::GenRef::gen_into_shared($reference, proof)
                    };
                }
                #[allow(unused_macros)]
                macro_rules! switch_shared_mut {
                    ($shared:tt $mutable:tt) => {
                        // For syntactic reasons, it is impossible to define a macro with a repeating capture group inside another macro.
                        // (The definition would be interpreted as a repeating expansion group of the outer macro.)
                        // So, processing is outsourced to a macro defined elsewhere.
                        $crate::__unwrap($shared);
                    };
                    ($shared:expr, $mutable:expr) => {
                        $shared
                    };
                }
                $code
            }
            $crate::MutabilityEnum::Mutable(proof) => {
                macro_rules! into_gen {
                    () => {
                        |genref| $crate::GenRef::gen_from_mut(genref, proof)
                    };
                    (&gen $genref:expr) => {
                        $crate::GenRef::gen_from_mut(&mut $genref, proof)
                    };
                    ($genref:expr) => {
                        $crate::GenRef::gen_from_mut($genref, proof)
                    };
                }
                macro_rules! from_gen {
                    () => {
                        |genref| $crate::GenRef::gen_into_mut(genref, proof)
                    };
                    ($reference:expr) => {
                        $crate::GenRef::gen_into_mut($reference, proof)
                    };
                }
                #[allow(unused_macros)]
                macro_rules! switch_shared_mut {
                    ($shared:tt $mutable:tt) => {
                        // For syntactic reasons, it is impossible to define a macro with a repeating capture group inside another macro.
                        // (The definition would be interpreted as a repeating expansion group of the outer macro.)
                        // So, processing is outsourced to a macro defined elsewhere.
                        $crate::__unwrap!($mutable)
                    };
                    ($shared:expr, $mutable:expr) => {
                        $mutable
                    };
                }
                $code
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __unwrap{
    ( { $($items:tt)* } ) => {
        $($items)*
    }
}

/// Maps a `GenRef` over field access and indexing.
///
/// Returns a `GenRef` to the field. Accessing nested fields is supported.
///
/// The receiver (the expression returning `GenRef`) must be a single token (an identifier) or it must be wrapped in parentheses.
///
/// Examples:
///
/// ```rust, ignore
/// field!(&gen genref.field)
/// field!(&gen genref.field1.2.field3[4])
/// field!(&gen (obtain_genref()).field)
/// field!(&gen (container.genref).field)
/// ```
#[macro_export]
macro_rules! field {
    (&gen $genref:tt $($field:tt)+) => {
        $crate::GenRef::map($genref, |r| &mut r $($field)+, |r| & r $($field)+)
    };
}

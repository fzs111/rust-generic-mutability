/// Convenience macro for mapping or splitting a `GenRef`, where both the mutable and immutable mapping functions are exactly the same.
/// 
/// Practically, this mostly means field access, indexing (`Index[Mut]`) and dereferencing (`Deref[Mut]`).
/// 
/// Syntax:
/// 
/// ```ignore
/// // Mapping:
/// let mapped = gen_ref!(gen_ref_variable [move moved_variable1, ...] -> &gen gen_ref_variable.field[index])
/// 
/// // Splitting:
/// let (split1, split2) = gen_ref!(gen_ref_variable [move moved_variable1, ...] -> (&gen gen_ref_variable.field1, &gen gen_ref_variable.field2))
/// ```
/// 
/// Examples:
/// 
/// Implementing generic indexing for slices:
/// ```
/// # use generic_mutability::gen_ref;
/// # use generic_mutability::Mutability;
/// # use generic_mutability::GenRef;
/// fn gen_index<M: Mutability, T>(slice: GenRef<'_, M, [T]>, idx: usize) -> GenRef<'_, M, T> {
///     gen_ref!(slice -> &gen slice[idx])
/// 
///     /* 
///     Becomes:
/// 
///     GenRef::map(
///         slice, 
///         |slice| &mut slice[idx], 
///         |slice| &    slice[idx]
///     )
///     */
/// }
/// ```
/// 
/// Implementing generic indexing for all `IndexMut` types:
/// ```
/// # use generic_mutability::gen_ref;
/// # use generic_mutability::Mutability;
/// # use generic_mutability::GenRef;
/// # use core::ops::IndexMut;
/// fn very_gen_index<M: Mutability, I, T: IndexMut<I>>(indexable: GenRef<'_, M, T>, index: I) -> GenRef<'_, M, T::Output> {
/// 
///     // `I` is not necessarily `Copy`, so we have to explicitly `move` it:
///     gen_ref!(indexable move index -> &gen indexable[index])
/// 
///     /*
///     Becomes:
/// 
///     GenRef::map_with_move(
///         indexable, 
///         index, 
///         |indexable, index| &mut indexable[index], 
///         |indexable, index| &    indexable[index]
///     )
///     */
/// }
/// ```
/// Or doing borrow splitting: 
/// ```
/// # use generic_mutability::gen_ref;
/// # use generic_mutability::Mutability;
/// # use generic_mutability::GenRef;
/// struct Point {
///     x: i32,
///     y: i32,
/// }
/// 
/// fn split_point<M: Mutability>(point: GenRef<'_, M, Point>) -> (GenRef<'_, M, i32>, GenRef<'_, M, i32>) {
///     gen_ref!(point -> (&gen point.x, &gen point.y))
/// }
/// ```
#[macro_export]
macro_rules! gen_ref {
    ($gen_ref:ident -> (&gen $place_a:expr, &gen $place_b:expr)) => {
        GenRef::split($gen_ref, |$gen_ref| (&mut $place_a, &mut $place_b), |$gen_ref| (& $place_a, & $place_b))
    };

    ($gen_ref:ident move $($moved:ident),+ -> (&gen $place_a:expr, &gen $place_b:expr)) => {
        GenRef::split_with_move($gen_ref, ($($moved),+,), |$gen_ref, ($($moved),+,)| (&mut $place_a, &mut $place_b), |$gen_ref, ($($moved),+,)| (& $place_a, & $place_b))
    };

    ($gen_ref:ident -> &gen $place:expr) => {
        GenRef::map($gen_ref, |$gen_ref| &mut $place, |$gen_ref| & $place)
    };

    // ? Are there any use cases for this branch?
    ($gen_ref:ident move $($moved:ident),+ -> &gen $place:expr) => {
        GenRef::map_with_move($gen_ref, ($($moved),+,), |$gen_ref, ($($moved),+,)| &mut $place, |$gen_ref, ($($moved),+,)| & $place)
    };
}
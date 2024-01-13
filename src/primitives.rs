use core::ptr::NonNull;

struct HList<Head, Tail>(Head, Tail);

macro_rules! tuple_ref_into_nonnull {
    ($trait:ident, $mut_or_not:ident) => {
        pub unsafe trait $trait<'a>{
            type Output;
            fn into_nonnull(ref_tuple: Self) -> Self::Output;
        }

        unsafe impl<'a> $trait<'a> for () {
            type Output = ();
            fn into_nonnull((): Self) -> () {
                ()
            }
        }

        unsafe impl<'a, Head: ?Sized, Tail: $trait<'a>> $trait<'a> for HList<$mut_or_not<'a, Head>, Tail> {
            type Output = (NonNull<Head>, <Tail as $trait<'a>>::Output);
            fn into_nonnull(HList(head, tail): Self) -> Self::Output {
                (NonNull::from(head), $trait::into_nonnull(tail))
            }
        }

        unsafe impl<'a, T: ?Sized, U: ?Sized> $trait<'a> for ($mut_or_not<'a, T>, $mut_or_not<'a, U>) {
            type Output = (NonNull<T>, NonNull<U>);
            fn into_nonnull((t, u): Self) -> Self::Output {
                (NonNull::from(t), NonNull::from(u))
            }
        }
        unsafe impl<'a, T: ?Sized, U: ?Sized, V: ?Sized> $trait<'a> for ($mut_or_not<'a, T>, $mut_or_not<'a, U>, $mut_or_not<'a, V>)

        {
            type Output = (NonNull<T>, NonNull<U>, NonNull<V>);
            fn into_nonnull((t, u, v): Self) -> Self::Output {
                (NonNull::from(t), NonNull::from(u), NonNull::from(v))
            }
        }
    }
}

// These type aliases make the macro a bit more readable to implement
type MutRef<'a, T> = &'a mut T;
type ImmutRef<'a, T> = &'a T;

tuple_ref_into_nonnull!(TupleMutIntoNonNull, MutRef);
tuple_ref_into_nonnull!(TupleImmutIntoNonNull, ImmutRef);
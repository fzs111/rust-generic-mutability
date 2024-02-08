use core::ptr::NonNull;

use crate::{GenRef, Mutability};

pub enum OneOf<T, U>{
    First(T),
    Second(U)
}
pub struct Untouched<T>(pub T);

macro_rules! impl_ref_into_nonnull {
    ($trait:ident, $mut_or_not:ident) => {
        pub unsafe trait $trait<'a>{
            type Output;
            fn into_nonnull_structure(ref_structure: Self) -> Self::Output;
        }

        unsafe impl<'a, T: ?Sized> $trait<'a> for $mut_or_not<'a, T>
        {
            type Output = NonNull<T>;
            fn into_nonnull_structure(reference: Self) -> NonNull<T> {
                NonNull::from(reference)
            }
        }

        unsafe impl<'a> $trait<'a> for () {
            type Output = ();
            fn into_nonnull_structure((): Self) -> () {
                ()
            }
        }

        unsafe impl<'a, T, U> $trait<'a> for (T, U)
            where 
                T: $trait<'a>,
                U: $trait<'a>
        {
            type Output = (T::Output, U::Output);
            fn into_nonnull_structure((t, u): Self) -> (T::Output, U::Output) {
                ($trait::into_nonnull_structure(t), $trait::into_nonnull_structure(u))
            }
        }

        unsafe impl<'a, T, U> $trait<'a> for OneOf<T, U>
            where 
                T: $trait<'a>,
                U: $trait<'a>
        {
            type Output = OneOf<T::Output, U::Output>;
            fn into_nonnull_structure(one_of: Self) -> OneOf<T::Output, U::Output> {
                match one_of {
                    OneOf::First(t) => OneOf::First($trait::into_nonnull_structure(t)),
                    OneOf::Second(u) => OneOf::Second($trait::into_nonnull_structure(u))
                }
            }
        }

        unsafe impl<'a, T> $trait<'a> for Untouched<T>
        {
            type Output = Untouched<T>;
            fn into_nonnull_structure(t: Self) -> Untouched<T> {
                t
            }
        }

/*
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
        */
    }
}

// These type aliases make the macro a bit more readable to implement
type MutRef<'a, T> = &'a mut T;
type ImmutRef<'a, T> = &'a T;

impl_ref_into_nonnull!(StructureMutIntoNonNull, MutRef);
impl_ref_into_nonnull!(StructureImmutIntoNonNull, ImmutRef);

/*
//TODO: Add safety requirements
pub unsafe trait TupleNonNullIntoGenRef<'a, M: Mutability>{
    type Output;
    unsafe fn into_genref(ref_tuple: Self) -> Self::Output;
}

//TODO: Add safety comment
unsafe impl<'a, M: Mutability> TupleNonNullIntoGenRef<'a, M> for () {
    type Output = ();

    #[inline]
    // This can be marked safe with this RFC: https://github.com/rust-lang/rust/issues/87919
    unsafe fn into_genref((): Self) -> () {
        ()
    }
}

//TODO: Add safety comment
unsafe impl<'a, M: Mutability, Head: 'a + ?Sized, Tail: TupleNonNullIntoGenRef<'a, M>> TupleNonNullIntoGenRef<'a, M> for HList<NonNull<Head>, Tail> {

    type Output = HList<GenRef<'a, M, Head>, <Tail as TupleNonNullIntoGenRef<'a, M>>::Output>;

    #[inline]
    unsafe fn into_genref(HList(head, tail): Self) -> Self::Output {
        HList(
            GenRef::new(head),
            TupleNonNullIntoGenRef::into_genref(tail)
        )
    }
}

//TODO: Add safety comment
unsafe impl<'a, M: Mutability, T: 'a + ?Sized, U: 'a + ?Sized> TupleNonNullIntoGenRef<'a, M> for (NonNull<T>, NonNull<U>) {

    type Output = (GenRef<'a, M, T>, GenRef<'a, M, U>);

    #[inline]
    unsafe fn into_genref((t, u): Self) -> Self::Output {
        (GenRef::new(t), GenRef::new(u))
    }
}

//TODO: Add safety comment
unsafe impl<'a, M: Mutability, T: 'a + ?Sized, U: 'a + ?Sized, V: 'a + ?Sized> TupleNonNullIntoGenRef<'a, M> for (NonNull<T>, NonNull<U>, NonNull<V>) {

    type Output = (GenRef<'a, M, T>, GenRef<'a, M, U>, GenRef<'a, M, V>);

    #[inline]
    unsafe fn into_genref((t, u, v): Self) -> Self::Output {
        (GenRef::new(t), GenRef::new(u), GenRef::new(v))
    }
}
*/

pub unsafe trait StructureNonNullIntoGenRef<'a, M: Mutability>{
    type Output;
    unsafe fn into_genref_structure(nonnull_structure: Self) -> Self::Output;
}

unsafe impl<'a, M: Mutability, T: 'a + ?Sized> StructureNonNullIntoGenRef<'a, M> for NonNull<T>
{
    type Output = GenRef<'a, M, T>;
    unsafe fn into_genref_structure(nonnull: Self) -> GenRef<'a, M, T> {
        GenRef::new(nonnull)
    }
}

unsafe impl<'a, M: Mutability> StructureNonNullIntoGenRef<'a, M> for () {
    type Output = ();
    unsafe fn into_genref_structure((): Self) -> () {
        ()
    }
}

unsafe impl<'a, M: Mutability, T, U> StructureNonNullIntoGenRef<'a, M> for (T, U)
    where 
        T: StructureNonNullIntoGenRef<'a, M>,
        U: StructureNonNullIntoGenRef<'a, M>
{
    type Output = (T::Output, U::Output);
    unsafe fn into_genref_structure((t, u): Self) -> (T::Output, U::Output) {
        (StructureNonNullIntoGenRef::into_genref_structure(t), StructureNonNullIntoGenRef::into_genref_structure(u))
    }
}

unsafe impl<'a, M: Mutability, T, U> StructureNonNullIntoGenRef<'a, M> for OneOf<T, U>
    where 
        T: StructureNonNullIntoGenRef<'a, M>,
        U: StructureNonNullIntoGenRef<'a, M>
{
    type Output = OneOf<T::Output, U::Output>;
    unsafe fn into_genref_structure(one_of: Self) -> OneOf<T::Output, U::Output> {
        match one_of {
            OneOf::First(t) => OneOf::First(StructureNonNullIntoGenRef::into_genref_structure(t)),
            OneOf::Second(u) => OneOf::Second(StructureNonNullIntoGenRef::into_genref_structure(u))
        }
    }
}

unsafe impl<'a, M: Mutability, T> StructureNonNullIntoGenRef<'a, M> for Untouched<T>
{
    type Output = Untouched<T>;
    unsafe fn into_genref_structure(t: Self) -> Untouched<T> {
        t
    }
}
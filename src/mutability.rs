use core::ptr::NonNull;

use crate::primitives::{StructureImmutIntoNonNull, StructureMutIntoNonNull};

pub(crate) mod seal{
    pub trait MutabilitySealed {}
}

/// This is one of the types implementing the `Mutability` trait, representing the mutability of a shared reference (`&T`).
/// It is an uninhabited type, only used as typestate.
///
/// Note that mutation of interior mutability types (built around `UnsafeCell`) **is allowed** by `Immutable`, just as it is allowed for shared references.
///
/// For more information, visit the documentation on the `Mutability` trait.
pub enum Immutable{}

/// This is one of the types implementing the `Mutability` trait, representing the mutability of a unique reference (`&mut T`).
/// It is an uninhabited type, only used as typestate.
///
/// For more information, visit the documentation on the `Mutability` trait.
pub enum Mutable{}

impl seal::MutabilitySealed for Mutable{}

impl seal::MutabilitySealed for Immutable{}

/*
/// This is the core primitive of this crate. 
/// It exposes raw, pointer-to-pointer operations for both mutability values.
/// This trait is used as a bound for all items that are generic over mutability.
///
/// It is implemented for two types, `Mutable` and `Immutable`, and it is sealed, so no other types can implement it.
pub unsafe trait Mutability: seal::MutabilitySealed{

    /// Casts the pointer to a reference and calls either `fn_mut` or `fn_immut` depending on the mutability. 
    /// Returns the value returned by the called closure.
    ///
    /// Capturing the same values with both closures will not work: instead, you can use `moved` helper argument to move the values into the closure that is actually called.
    /// If you do not need to capture any values, you can pass `()` to ignore it. 
    ///
    /// # Safety
    ///
    /// `ptr` will be dereferenced and converted to a reference of the chosen mutability. As such:
    ///
    /// - The pointer must be properly aligned.
    /// - The pointer must point to an initialized instance of `T`.
    /// - The lifetime `'a` is arbitrarily chosen and doesn't reflect the actual lifetime of the data. Extra care must be taken to ensure that the correct lifetime is used.
    /// - Furthermore:
    ///     - If the mutability is `Immutable`:
    ///         - The pointer must be valid for reads for lifetime `'a`.
    ///         - The pointed-to value must not be written to by other pointers and no mutable references to it may exist during `'a`.
    ///     - If the mutability is `Mutable`:
    ///         - The pointer must be valid for reads and writes for lifetime `'a`.
    ///         - The pointed-to value must not be accessed (read or written) by other pointers, and no references to it may exist during `'a`.
    unsafe fn dispatch<'a, T, U, X, FM, FIM>(ptr: NonNull<T>, moved: X, fn_mut: FM, fn_immut: FIM) -> U
        where 
            T: 'a + ?Sized,
            FM:  FnOnce(&'a mut T, X) -> U,
            FIM: FnOnce(&'a T,     X) -> U;

    /// Casts the pointer to a reference of the given mutability, maps it into another one of the same mutability and returns it as a pointer.
    ///
    /// Capturing the same values with both closures will not work: instead, you can use `moved` helper argument to move the values into the closure that is actually called.
    /// If you do not need to capture any values, you can pass `()` to ignore it. 
    ///
    /// # Safety
    ///
    /// `ptr` will be dereferenced and converted to a reference of the chosen mutability. As such:
    ///
    /// - The pointer must be properly aligned.
    /// - The pointer must point to an initialized instance of `T`.
    /// - The lifetime `'a` is arbitrarily chosen and doesn't reflect the actual lifetime of the data. Extra care must be taken to ensure that the correct lifetime is used.
    /// - Furthermore:
    ///     - If the mutability is `Immutable`:
    ///         - The pointer must be valid for reads for lifetime `'a`.
    ///         - The pointed-to value must not be written to by other pointers and no mutable references to it may exist during `'a`.
    ///     - If the mutability is `Mutable`:
    ///         - The pointer must be valid for reads and writes for lifetime `'a`.
    ///         - The pointed-to value must not be accessed (read or written) by other pointers, and no references to it may exist during `'a`.
    /// 
    /// # Guarantees
    /// 
    /// The returned pointer also satisfies all requirements listed in the Safety section. `unsafe` code may rely on this guarantee.
    unsafe fn map<'a, T, U, X, FM, FIM>(ptr: NonNull<T>, moved: X, fn_mut: FM, fn_immut: FIM) -> NonNull<U>
        where 
            T: 'a + ?Sized,
            U: 'a + ?Sized,
            FM:  FnOnce(&'a mut T, X) -> &'a mut U,
            FIM: FnOnce(&'a T,     X) -> &'a U;

    /// Casts the pointer to a reference of the given mutability, and maps it to two derived references of the same mutability and returns them as a tuple of pointers.
    ///
    /// Capturing the same values with both closures will not work: instead, you can use `moved` helper argument to move the values into the closure that is actually called.
    /// If you do not need to capture any values, you can pass `()` to ignore it.
    ///
    /// Note: it is not yet decided whether this will be generalized to support n-way splitting.
    /// Three implementations already exist, see the `primitives`, `split-tuples-macros` and `split-cons` branches.
    /// If you have a use case that requires more than 2-way splitting, please tell me about it in an issue.
    ///
    /// # Safety
    ///
    /// `ptr` will be dereferenced and converted to a reference of the chosen mutability. As such:
    ///
    /// - The pointer must be properly aligned.
    /// - The pointer must point to an initialized instance of `T`.
    /// - The lifetime `'a` is arbitrarily chosen and doesn't reflect the actual lifetime of the data. Extra care must be taken to ensure that the correct lifetime is used.
    /// - Furthermore:
    ///     - If the mutability is `Immutable`:
    ///         - The pointer must be valid for reads for lifetime `'a`.
    ///         - The pointed-to value must not be written to by other pointers and no mutable references to it may exist during `'a`.
    ///     - If the mutability is `Mutable`:
    ///         - The pointer must be valid for reads and writes for lifetime `'a`.
    ///         - The pointed-to value must not be accessed (read or written) by other pointers, and no references to it may exist during `'a`.
    /// 
    /// # Guarantees
    /// 
    /// Both returned pointers also satisfy all requirements listed in the Safety section. `unsafe` code may rely on this guarantee.
    unsafe fn split<'a, T, U, V, X, FM, FIM>(ptr: NonNull<T>, moved: X, fn_mut: FM, fn_immut: FIM) -> (NonNull<U>, NonNull<V>)
        where 
            T: 'a + ?Sized,
            U: 'a + ?Sized,
            V: 'a + ?Sized,
            FM:  FnOnce(&'a mut T, X) -> (&'a mut U, &'a mut V),
            FIM: FnOnce(&'a T,     X) -> (&'a U, &'a V);

    /// Returns `true` if the mutability is `Mutable`, `false` otherwise.
    ///
    /// This can be used to observe the mutability of the value at runtime.
    /// Consider using `dispatch` or `map` instead for compile-time resolution.
    ///
    /// # Guarantees
    /// 
    /// This method has to be implemented correctly and `unsafe` code can rely on this guarantee.
    fn is_mutable() -> bool;
}

unsafe impl Mutability for Mutable{

    #[inline]
    unsafe fn dispatch<'a, T, U, X, FM, FIM>(mut ptr: NonNull<T>, moved: X, fn_mut: FM, _fn_immut: FIM) -> U
        where 
            T: 'a + ?Sized,
            FM:  FnOnce(&'a mut T, X) -> U,
            FIM: FnOnce(&'a T,     X) -> U, 
    {
        let reference = unsafe {
            // SAFETY: the caller must uphold the safety contract
            ptr.as_mut()
        };

        fn_mut(reference, moved)
    }

    #[inline]
    unsafe fn map<'a, T, U, X, FM, FIM>(mut ptr: NonNull<T>, moved: X, fn_mut: FM, _fn_immut: FIM) -> NonNull<U>
        where 
            T: 'a + ?Sized,
            U: 'a + ?Sized,
            FM:  FnOnce(&'a mut T, X) -> &'a mut U,
            FIM: FnOnce(&'a T,     X) -> &'a U,
    {
        let reference = unsafe{
            // SAFETY: the caller must uphold the safety contract
            ptr.as_mut()
        };
        fn_mut(reference, moved).into()
    }

    #[inline]
    unsafe fn split<'a, T, U, V, X, FM, FIM>(mut ptr: NonNull<T>, moved: X, fn_mut: FM, _fn_immut: FIM) -> (NonNull<U>, NonNull<V>)
        where 
            T: 'a + ?Sized,
            U: 'a + ?Sized,
            V: 'a + ?Sized,
            FM:  FnOnce(&'a mut T, X) -> (&'a mut U, &'a mut V),
            FIM: FnOnce(&'a T,     X) -> (&'a U, &'a V) 
    {
        let reference = unsafe{
            // SAFETY: the caller must uphold the safety contract
            ptr.as_mut()
        };
        let (a, b) = fn_mut(reference, moved);
        (a.into(), b.into())
    }

    #[inline]
    fn is_mutable() -> bool {
        true
    }
}

unsafe impl Mutability for Immutable{

    #[inline]
    unsafe fn dispatch<'a, T, U, X, FM, FIM>(ptr: NonNull<T>, moved: X, _fn_mut: FM, fn_immut: FIM) -> U
        where 
            T: 'a + ?Sized,
            FM:  FnOnce(&'a mut T, X) -> U,
            FIM: FnOnce(&'a T,     X) -> U,
    {
        let reference = unsafe{
            // SAFETY: the caller must uphold the safety contract
            ptr.as_ref()
        };
        fn_immut(reference, moved)
    }

    #[inline]
    unsafe fn map<'a, T, U, X, FM, FIM>(ptr: NonNull<T>, moved: X, _fn_mut: FM, fn_immut: FIM) -> NonNull<U>
        where 
            T: 'a + ?Sized,
            U: 'a + ?Sized,
            FM:  FnOnce(&'a mut T, X) -> &'a mut U,
            FIM: FnOnce(&'a T,     X) -> &'a U,
    {
        let reference = unsafe{
            // SAFETY: the caller must uphold the safety contract
            ptr.as_ref()
        };
        fn_immut(reference, moved).into()
    }

    #[inline]
    unsafe fn split<'a, T, U, V, X, FM, FIM>(ptr: NonNull<T>, moved: X, _fn_mut: FM, fn_immut: FIM) -> (NonNull<U>, NonNull<V>)
        where 
            T: 'a + ?Sized,
            U: 'a + ?Sized,
            V: 'a + ?Sized,
            FM:  FnOnce(&'a mut T, X) -> (&'a mut U, &'a mut V),
            FIM: FnOnce(&'a T,     X) -> (&'a U, &'a V)
    {
        let reference = unsafe{
            // SAFETY: the caller must uphold the safety contract
            ptr.as_ref()
        };
        let (a, b) = fn_immut(reference, moved);
        (a.into(), b.into())
    }

    #[inline]
    fn is_mutable() -> bool {
        false
    }
}
*/

/// This is the core primitive of this crate.
/// It exposes raw, pointer-to-pointer operations for both mutability values.
/// This trait is used as a bound for all items that are generic over mutability.
///
/// It is implemented for two types, `Mutable` and `Immutable`, and it is sealed, so no other types can implement it.
pub unsafe trait Mutability: seal::MutabilitySealed {
    unsafe fn map_to_structure<'a, T, U, UM, UIM, X>(
        ptr: NonNull<T>,
        moved: X,
        fn_mut:   impl FnOnce(&'a mut T, X) -> UM,
        fn_immut: impl FnOnce(&'a     T, X) -> UIM
    ) -> U
        where
            T: 'a + ?Sized,
            UM: StructureMutIntoNonNull<'a, Output = U>,
            UIM: StructureImmutIntoNonNull<'a, Output = U>;

    /// Is `true` if the mutability is `Mutable` and `false` if the mutability is `Immutable`.
    ///
    /// This can be used to observe the mutability of the value at runtime.
    /// Consider using `dispatch` or `map` instead for compile-time resolution.
    ///
    /// This constant has to be implemented correctly and `unsafe` code can rely on this guarantee.
    const IS_MUTABLE: bool;
}

unsafe impl Mutability for Mutable {

    unsafe fn map_to_structure<'a, T, U, UM, UIM, X>(
        ptr: NonNull<T>,
        moved: X,
        fn_mut:    impl FnOnce(&'a mut T, X) -> UM,
        _fn_immut: impl FnOnce(&'a     T, X) -> UIM
    ) -> U
        where
            T: 'a + ?Sized,
            UM: StructureMutIntoNonNull<'a, Output = U>,
            UIM: StructureImmutIntoNonNull<'a, Output = U>
    {
        let reference = unsafe{
            //TODO: Add safety comment
            &mut *ptr.as_ptr()
        };
        StructureMutIntoNonNull::into_nonnull_structure(fn_mut(reference, moved))
    }

    const IS_MUTABLE: bool = true;
}
unsafe impl Mutability for Immutable {
    unsafe fn map_to_structure<'a, T, U, UM, UIM, X>(
        ptr: NonNull<T>,
        moved: X,
        _fn_mut:  impl FnOnce(&'a mut T, X) -> UM,
        fn_immut: impl FnOnce(&'a     T, X) -> UIM
    ) -> U
        where
            T: 'a + ?Sized,
            UM: StructureMutIntoNonNull<'a, Output = U>,
            UIM: StructureImmutIntoNonNull<'a, Output = U>
    {
        let reference = unsafe{
            //TODO: Add safety comment
            & *ptr.as_ptr()
        };
        StructureImmutIntoNonNull::into_nonnull_structure(fn_immut(reference, moved))
    }

    const IS_MUTABLE: bool = false;
}
use core::borrow::{Borrow, BorrowMut};
use core::fmt;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

use crate::mutability::{IsMutable, IsShared, Mutability, Mutable, Shared};

mod impl_traits;

/// This is the main type of this crate.
/// It is the generic-mutability equivalent of safe reference types (`&` and `&mut`).
///
/// # Usage for end users
///
/// *You need to interface with an API that is generic over mutability, but you do not use generic mutability to call it.*
///
/// You will need to create a `GenRef<'_, Mutable, T>` or `GenRef<'_, Shared, T>` via the `From<&mut T>` or `From<&T>` implementations, respectively.
/// Then pass the `GenRef` to the function(s) you would like to call.
/// At the end, you can use the resulting `GenRef` mostly the same way you would use a reference, but you can also unwrap it into a normal reference using the `into_mut` or `into_shared` functions.
///
/// For example:
///
/// ```
/// # use core::ops::{Index, IndexMut};
/// # use generic_mutability::{gen_mut, GenRef, Mutability, Shared};
/// # fn gen_index<M: Mutability, I, C>(gen_collection: GenRef<'_, M, C>, index: I) -> GenRef<'_, M, C::Output>
/// # where
/// #     C: Index<I> + IndexMut<I>,
/// # {
/// #     gen_mut! {M => {
/// #         into_gen!(switch_shared_mut![C::index, C::index_mut](from_gen!(gen_collection), index))
/// #     }}
/// # }
/// let v = vec![1, 2, 3];
///
/// let gen_r: GenRef<'_, Shared, i32> = gen_index(GenRef::from(&v), 1);
///
/// let r: &i32 = GenRef::into_shared(gen_r);
///
/// assert_eq!(gen_r, &2);
/// assert_eq!(r, &2);
/// ```
///
/// # Usage for libraries
///
/// *You are implementing an API that is supposed to be generic over mutability.*
///
/// You should start by introducing a *mutability parameter*.
/// It is a generic argument (usually named `M`) with a `M: Mutability` bound on it .
///
/// Then you can take a `GenRef` as an argument.
///
/// `GenRef` always provides immutable access to the pointed-to value through the `Deref` trait.
///
/// Then, to map the generic reference into one of another type, you can do one of these:
///
/// - If the API you are calling has generic mutability accessors, you can pass the `GenRef` directly to them.
///     Unlike normal references, which are automatically reborrowed, you may need to use `GenRef::reborrow` to perform a reborrow manually.
///     You can also call `map_deref` to perform a dereference.
///
/// - If the API you're calling *does not* have generic mutability, you can use one of the following ways to unwrap and reconstruct the `GenRef`:
///     - `map`
///     - `field!` macro for accessing fields
///     - `gen_mut!` macro
///     - branch to cases on `Mutability::mutability()` and use `gen_{into,from}_{mut,shared}` with the proof provided by the return value of `Mutability::mutability()`.
///         See the Examples section on how to do this.
///
/// # Examples
///
/// When you need to map a generic `GenRef` through an API that doesn't support it, you can use the following pattern:
///
/// ```
/// # use generic_mutability::{ GenRef, Mutability, MutabilityEnum::* };
/// fn gen_index<M: Mutability, T>(gen_slice: GenRef<'_, M, [T]>, index: usize) -> GenRef<'_, M, T> {
///     match M::mutability() {
///         Mutable(proof) => {
///             let mut_slice: &mut [T] = GenRef::gen_into_mut(gen_slice, proof);
///             let mut_elem: &mut T = &mut mut_slice[index];
///             GenRef::gen_from_mut(mut_elem, proof)
///         },
///         Shared(proof) => {
///             let ref_slice: &[T] = GenRef::gen_into_shared(gen_slice, proof);
///             let ref_elem: &T = &ref_slice[index];
///             GenRef::gen_from_shared(ref_elem, proof)
///         }
///     }
/// }
/// ```
///
/// In most cases, the mutable and shared cases share most of the code. When that is the case, it is often possible to use the `gen_mut!` macro, which expands to the same as above:
///
/// ```
/// # use generic_mutability::{ GenRef, Mutability, gen_mut };
/// fn gen_index<M: Mutability, T>(gen_slice: GenRef<'_, M, [T]>, index: usize) -> GenRef<'_, M, T> {
///     gen_mut!(M => {
///         let ref_slice = from_gen!(gen_slice);
///         into_gen!(&gen ref_slice[index])
///     })
/// }
/// ```
///
/// When performing a one-to-one mapping with two separate functions (often when calling into an API), the `GenRef::map` can come handy:
///
/// ```
/// # use generic_mutability::{ GenRef, Mutability };
/// fn gen_index<M: Mutability, T>(gen_slice: GenRef<'_, M, [T]>, index: usize) -> GenRef<'_, M, T> {
///     GenRef::map(gen_slice, |r| &r[index], |r| &mut r[index])
/// }
/// ```
///
/// Finally, when accessing fields or performing indexing, the `field!` macro can be helpful as well:
///
/// ```
/// # use generic_mutability::{ GenRef, Mutability, field };
/// fn gen_index<M: Mutability, T>(gen_slice: GenRef<'_, M, [T]>, index: usize) -> GenRef<'_, M, T> {
///     field!(&gen gen_slice[index])
/// }
/// ```
#[repr(transparent)]
pub struct GenRef<'s, M: Mutability, T: ?Sized> {
    // This could contain an `ErasedMutRef` instead of `_lifetime` and `ptr`,
    // but that way it could not implement `Copy`
    _lifetime: PhantomData<&'s mut T>,
    _mutability: PhantomData<*const M>,
    ptr: NonNull<T>,
}

macro_rules! docs_for {
    (as_ptr) => {
         "Casts the reference into a `NonNull` pointer.

# Safety

The `GenRef` must not be used while the pointer is active. 
The exact semantics of this depend on the memory model adopted by Rust.

# Guarantees

The returned pointer is guaranteed to be valid for reads for `'s`, and also for writes if `M` is `Mutable`."
    };
    (gen_into_shared_downgrading) => {
         "Converts a generic `GenRef<'_, M, T>` into `&T`, downgrading the reference if `M` is `Mutable`.

If `M` is `Shared` it behaves exactly the same way as `gen_into_shared` without requiring a proof for sharedness.
In this case, the difference between the two is purely semantic: if you have proof that `M` is `Shared`, you should use `gen_into_shared`."
    };
    (gen_into_mut) => {
         "Converts a generic `GenRef<'_, M, T>` into `&mut T`. 
This is available in a generic context.

Once the transformations are done, the result can be converted back into a `GenRef` using the `gen_from_mut` function.

The conversion requires that `M` is `Mutable`, this must be proven by passing an `IsMutable<M>` value.
That can be obtained by `match`ing on `M::mutability()`."
    };
    (gen_into_shared) => {
         "Converts a generic `GenRef<'_, M, T>` into `&T`. 
This is available in a generic context.

Once the transformations are done, the result can be converted back into a `GenRef` using the `gen_from_shared` function.

The conversion requires that `M` is `Shared`, this must be proven by passing an `IsShared<M>` value.
That can be obtained by `match`ing on `M::mutability()`.

If you want to force the conversion even if `M` is `Mutable`, you can use the `gen_into_shared_downgrading` function."
    };
    (reborrow) => {
         "Generically reborrows a `GenRef`. 
That is, it creates a shorter-lived owned `GenRef` from a `&mut GenRef`.
This is available in a generic context.

This requires the variable to be marked `mut`, even if `M` is `Shared` and thus no mutation takes place."
    };
    (map) => {
         "Maps a generic `GenRef` into another one using either `f_mut` or `f_shared`. 
This is available in a generic context.

Using this function is usually sufficient.
For mapping over field access, you can use the `field!` macro instead.
If you need more flexibility, you can use the `gen_mut!` macro or `match`ing over `M::mutability()`."
    };
    (map_deref) => {
         "Generically dereferences the value contained in the `GenRef`.
This is available in a generic context."
    };
}

impl<'s, M: Mutability, T: ?Sized> GenRef<'s, M, T> {
    /// Creates a `GenRef<'s, M, T>` from a pointer with the chosen mutability `M` and lifetime `'s`, without checking.
    ///
    /// To create a non-generic `GenRef` from a reference, use the `From` implementation.
    ///
    /// To convert a reference into a `GenRef` in a generic context, use the `gen_from_shared` and `gen_from_mut` methods with a proof or the `gen_from_mut_downgrading` method instead.
    ///
    /// # Safety
    ///
    /// `GenRef` is a safe reference type. Using this method is equivalent to dereferencing the pointer and creating a reference from it. As such:
    ///
    /// - The pointer must be properly aligned.
    /// - The pointer must point to an initialized instance of `T`.
    /// - The lifetime `'s` and mutability `M` are arbitrarily chosen and do not necessarily reflect the actual lifetime and mutability of the data.
    ///     Extra care must be taken to ensure that the correct lifetime and mutability parameters are used.
    /// - Furthermore:
    ///     - If the mutability is `Immutable`:
    ///         - The pointer must be valid for reads for lifetime `'s`.
    ///         - The pointed-to value must not be written to by other pointers and no mutable references to it may exist during `'s`.
    ///     - If the mutability is `Mutable`:
    ///         - The pointer must be valid for reads and writes for lifetime `'s`.
    ///         - The pointed-to value must not be accessed (read or written) by other pointers, and no other references to it may exist during `'s`.
    pub unsafe fn from_ptr_unchecked(ptr: NonNull<T>) -> Self {
        Self {
            _lifetime: PhantomData,
            _mutability: PhantomData,
            ptr,
        }
    }

    #[doc = docs_for!(as_ptr)]
    pub fn as_ptr(genref: &Self) -> NonNull<T> {
        genref.ptr
    }

    /// Converts a `&mut T` into a generic `GenRef<'_, M, T>`, downgrading the reference if `M` is `Shared`.
    ///
    /// If `M` is `Mutable` it behaves exactly the same way as `gen_from_mut` without requiring a proof for mutability.
    /// In this case, the difference between the two is purely semantic: if you have proof that `M` is `Mutable`, you should use `gen_from_mut`.
    pub fn gen_from_mut_downgrading(reference: &'s mut T) -> Self {
        let ptr = NonNull::from(reference);

        // SAFETY: `ptr` is derived from a mutable reference, so points to a valid `T` and is valid and unaliased for `'s`.
        // The correct lifetime is enforced by the function signature.
        unsafe { Self::from_ptr_unchecked(ptr) }
    }

    #[doc = docs_for!(gen_into_shared_downgrading)]
    pub fn gen_into_shared_downgrading(genref: Self) -> &'s T {
        let ptr = GenRef::as_ptr(&genref);

        // SAFETY: `GenRef::as_ptr` guarantees that `ptr` points to a valid `T` and is valid for reads for `'s`.
        unsafe { ptr.as_ref() }
    }

    #[doc = docs_for!(gen_into_mut)]
    pub fn gen_into_mut(genref: Self, _proof: IsMutable<M>) -> &'s mut T {
        let mut ptr = GenRef::as_ptr(&genref);

        // SAFETY: For a value of `IsMutable<M>` to exist, `M` must be `Mutable`.
        // `GenRef::as_ptr` guarantees that `ptr` points to a valid `T` and (given that `M` is `Mutable`) is valid for reads and writes for `'s`.
        // This function takes ownership of `GenRef`, so `ptr` can not be aliased.
        unsafe { ptr.as_mut() }
    }
    /// Converts a `&mut T` into a generic `GenRef<'_, M, T>`.
    /// This is available in a generic context.
    ///
    /// The conversion requires that `M` is `Mutable`, this must be proven by passing an `IsMutable<M>` value.
    /// That can be obtained by `match`ing on `M::mutability()`.
    ///
    /// If you want to force the conversion even if `M` is `Shared`, you can use the `gen_from_mut_downgrading` function.
    pub fn gen_from_mut(reference: &'s mut T, _proof: IsMutable<M>) -> Self {
        // `gen_from_mut_downgrading` is semantically different, but when called with `M = Mutable` they perform the same operation.
        GenRef::gen_from_mut_downgrading(reference)
    }

    #[doc = docs_for!(gen_into_shared)]
    pub fn gen_into_shared(genref: Self, _proof: IsShared<M>) -> &'s T {
        // `gen_into_shared_downgrading` is semantically different, but when called with `M = Shared` they perform the same operation.
        GenRef::gen_into_shared_downgrading(genref)
    }

    /// Converts a `&T` into a generic `GenRef<'_, M, T>`.
    /// This is available in a generic context.
    ///
    /// The conversion requires that `M` is `Shared`, this must be proven by passing an `IsShared<M>` value.
    /// That can be obtained by `match`ing on `M::mutability()`.
    pub fn gen_from_shared(reference: &'s T, _proof: IsShared<M>) -> Self {
        let ptr = NonNull::from(reference);

        // SAFETY: For a value of `IsShared<M>` to exist, `M` must be `Shared`. `ptr` is derived from a shared reference, so it is valid for reads for `'s`.
        // The correct lifetime is enforced by the function signature.
        unsafe { Self::from_ptr_unchecked(ptr) }
    }

    #[doc = docs_for!(reborrow)]
    pub fn reborrow(genref: &mut Self) -> GenRef<'_, M, T> {
        // SAFETY: `GenRef::as_ptr` guarantees that `ptr` points to a valid `T` and is valid for reads for `'_`. If `M` is `Mutable`, it also guarantees validity for writes.
        // The `GenRef` received as argument is not used after calling `as_ptr` for the lifetime `'_`, and the pointer returned by `as_ptr` can not be aliased, both guaranteed by the exclusive reference received as argument.
        // The correct lifetime and mutability parameters are enforced by the function signature.
        unsafe { GenRef::from_ptr_unchecked(GenRef::as_ptr(genref)) }
    }

    #[doc = docs_for!(map)]
    pub fn map<U: ?Sized>(
        genref: Self,
        f_shared: impl FnOnce(&T) -> &U,
        f_mut: impl FnOnce(&mut T) -> &mut U,
    ) -> GenRef<'s, M, U> {
        use crate::MutabilityEnum::*;

        match M::mutability() {
            Mutable(proof) => {
                GenRef::gen_from_mut(f_mut(GenRef::gen_into_mut(genref, proof)), proof)
            }
            Shared(proof) => {
                GenRef::gen_from_shared(f_shared(GenRef::gen_into_shared(genref, proof)), proof)
            }
        }
    }

    #[doc = docs_for!(map_deref)]
    pub fn map_deref(genref: Self) -> GenRef<'s, M, T::Target>
    where
        T: Deref + DerefMut,
    {
        GenRef::map(genref, Deref::deref, DerefMut::deref_mut)
    }
}

mod seal {
    use crate::{GenRef, Mutability};

    pub trait Sealed {}
    impl<M: Mutability, T: ?Sized> Sealed for GenRef<'_, M, T> {}
}

/// This trait allows you to call associated functions of `GenRef` with method syntax.
///
/// These functions are not methods on `GenRef` to avoid confusion of own methods with methods of `T` (`GenRef` implements `Deref`).
/// This trait lets you bypass this restriction in cases where it is not confusing. Use with care.
///
/// This trait is only implemented for `GenRef<'_, M, T>` and is sealed so no other types can implement it.
///
/// In theory, it is also possible to receive `impl GenRefMethods<'_, M, T>` instead of `GenRef<'_, M, T>` as an argument, which disables the `Deref` impl, although it is more confusing from the caller side.
pub trait GenRefMethods<'s, M: Mutability, T: ?Sized>: seal::Sealed {
    /// This is a method variant of the equivalent associated function on `GenRef`.
    #[doc = docs_for!(as_ptr)]
    fn as_ptr(&self) -> NonNull<T>;

    /// This is a method variant of the equivalent associated function on `GenRef`.
    #[doc = docs_for!(gen_into_shared_downgrading)]
    fn gen_into_shared_downgrading(self) -> &'s T;

    /// This is a method variant of the equivalent associated function on `GenRef`.
    #[doc = docs_for!(gen_into_mut)]
    fn gen_into_mut(self, _proof: IsMutable<M>) -> &'s mut T;

    /// This is a method variant of the equivalent associated function on `GenRef`.
    #[doc = docs_for!(gen_into_shared)]
    fn gen_into_shared(self, _proof: IsShared<M>) -> &'s T;
    /// This is a method variant of the equivalent associated function on `GenRef`.
    #[doc = docs_for!(reborrow)]
    fn reborrow(&mut self) -> GenRef<'_, M, T>;

    /// This is a method variant of the equivalent associated function on `GenRef`.
    #[doc = docs_for!(map)]
    fn map<U: ?Sized>(
        self,
        f_mut: impl FnOnce(&mut T) -> &mut U,
        f_shared: impl FnOnce(&T) -> &U,
    ) -> GenRef<'s, M, U>;

    /// This is a method variant of the equivalent associated function on `GenRef`.
    #[doc = docs_for!(map_deref)]
    fn map_deref(self) -> GenRef<'s, M, T::Target>
    where
        T: Deref + DerefMut;

    /// Dereferences the `GenRef`. Same as `Deref::deref(self)`.
    /// This method allows you to call methods on the referenced value explicitly.
    fn deref(&self) -> &T;
}
impl<'s, M: Mutability, T: ?Sized> GenRefMethods<'s, M, T> for GenRef<'s, M, T> {
    fn as_ptr(&self) -> NonNull<T> {
        GenRef::as_ptr(self)
    }

    fn gen_into_shared_downgrading(self) -> &'s T {
        GenRef::gen_into_shared_downgrading(self)
    }
    fn gen_into_mut(self, proof: IsMutable<M>) -> &'s mut T {
        GenRef::gen_into_mut(self, proof)
    }

    fn gen_into_shared(self, proof: IsShared<M>) -> &'s T {
        GenRef::gen_into_shared(self, proof)
    }

    fn reborrow(&mut self) -> GenRef<'_, M, T> {
        GenRef::reborrow(self)
    }

    fn map<U: ?Sized>(
        self,
        f_mut: impl FnOnce(&mut T) -> &mut U,
        f_shared: impl FnOnce(&T) -> &U,
    ) -> GenRef<'s, M, U> {
        GenRef::map(self, f_shared, f_mut)
    }

    fn map_deref(self) -> GenRef<'s, M, T::Target>
    where
        T: Deref + DerefMut,
    {
        GenRef::map_deref(self)
    }

    fn deref(&self) -> &T {
        self
    }
}

impl<'s, T: ?Sized> GenRef<'s, Shared, T> {
    /// Converts a `GenRef<'_, Shared, T>` into `&T` in a non-generic context.
    ///
    /// This is used to unwrap the reference in end-user code.
    ///
    /// To perform the same operation in a generic context, use `gen_into_shared` or `gen_into_shared_downgrading`.
    pub fn into_shared(genref: Self) -> &'s T {
        Self::gen_into_shared(genref, Shared::mutability())
    }
}
impl<'s, T: ?Sized> GenRef<'s, Mutable, T> {
    /// Converts a `GenRef<'_, Mutable T>` into `&mut T` in a non-generic context.
    ///
    /// This is used to unwrap the reference in end-user code.
    ///
    /// To perform the same operation in a generic context, use `gen_into_mut`.
    pub fn into_mut(genref: Self) -> &'s mut T {
        Self::gen_into_mut(genref, Mutable::mutability())
    }
}

/// Creates a non-generic `GenRef<'_, Shared, T>` from a `&T`.
///
/// This is used to create a `GenRef` in end-user code.
///
/// To create a generic `GenRef<'_, M, T>`, use `gen_from_shared`.
impl<'s, T: ?Sized> From<&'s T> for GenRef<'s, Shared, T> {
    fn from(reference: &'s T) -> Self {
        GenRef::gen_from_shared(reference, Shared::mutability())
    }
}
/// Creates a non-generic `GenRef<'_, Mutable, T>` from a `&mut T`.
///
/// This is used to create a `GenRef` in end-user code.
///
/// To create a generic `GenRef<'_, M, T>`, use `gen_from_mut` or `gen_from_mut_downgrading`.
impl<'s, T: ?Sized> From<&'s mut T> for GenRef<'s, Mutable, T> {
    fn from(reference: &'s mut T) -> Self {
        GenRef::gen_from_mut(reference, Mutable::mutability())
    }
}

/// This is available in a generic context.
impl<M: Mutability, T: ?Sized> Deref for GenRef<'_, M, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        let ptr = GenRef::as_ptr(self);

        // SAFETY: `GenRef::as_ptr` guarantees that `ptr` points to a valid `T` and is valid for reads for `'s`.
        // The correct lifetime is enforced by the function signature.
        unsafe { ptr.as_ref() }
    }
}

/// This is only implemented for `GenRef<'_, Mutable, T>`, and is not available in a generic context.
///
/// To get mutable access to the value, call `reborrow` followed by `gen_into_mut` (this requires proving that `M` is mutable).
impl<T: ?Sized> DerefMut for GenRef<'_, Mutable, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        GenRef::into_mut(GenRef::reborrow(self))
    }
}

/// This is only implemented for `GenRef<'_, Shared, T>`, and is not available in a generic context.
impl<T: ?Sized> Clone for GenRef<'_, Shared, T> {
    /// Copies the reference. This does not call `clone` on the pointed-to value.
    fn clone(&self) -> Self {
        *self
    }
}
/// This is only implemented for `GenRef<'_, Shared, T>`, and is not available in a generic context.
impl<T: ?Sized> Copy for GenRef<'_, Shared, T> {}

impl<M: Mutability, T: ?Sized> Borrow<T> for GenRef<'_, M, T> {
    fn borrow(&self) -> &T {
        self
    }
}
/// This is only implemented for `GenRef<'_, Mutable, T>`, and is not available in a generic context.
impl<T: ?Sized> BorrowMut<T> for GenRef<'_, Mutable, T> {
    fn borrow_mut(&mut self) -> &mut T {
        self
    }
}

impl<M: Mutability, T: ?Sized> fmt::Pointer for GenRef<'_, M, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.ptr.fmt(f)
    }
}

/// This implementation requires `T: Sync` even when `M` is `Mutable`.
/// With specialisation, this requirement could be lifted.
// SAFETY: `GenRef` behaves like a reference, and both `&T` and `&mut T` implement `Send` if `T` is `Send` and `Sync`
unsafe impl<M: Mutability, T: ?Sized> Send for GenRef<'_, M, T> where T: Send + Sync {}
// SAFETY: `GenRef` behaves like a reference, and both `&T` and `&mut T` implement `Sync` if `T` is `Sync`
unsafe impl<M: Mutability, T: ?Sized> Sync for GenRef<'_, M, T> where T: Sync {}

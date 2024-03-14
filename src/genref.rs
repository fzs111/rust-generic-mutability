use core::borrow::{Borrow, BorrowMut};
use core::cmp::Ordering;
use core::fmt;
use core::hash::Hash;
use core::iter::FusedIterator;
use core::ops::{Deref, DerefMut};
use core::marker::PhantomData;
use core::ptr::NonNull;

#[cfg(any(feature = "std", doc))]
extern crate std;

use crate::erased_mutability_ref::ErasedMutabilityRef;
use crate::mutability::{Mutability, Mutable, Shared, IsMutable, IsShared};

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
/// # use generic_mutability::{ GenRef, Mutability, Shared };
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
/// `GenRef` always provides immutable access to te pointed-to value through the `Deref` trait.
/// 
/// Then, to map the generic reference into one of another type, you can do one of these:
/// 
/// - If the API you are calling has generic mutability accessors, you can pass the `GenRef` directly to them.
///     Unlike normal references, which are automatically reborrowed, you may need to use `GenRef::reborrow` to perform a reborrow manually. 
///     You can also call `as_deref` to perform a dereference.
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
#[repr(transparent)]
pub struct GenRef<'s, M: Mutability, T: ?Sized> {
    // This could contain an `ErasedMutRef` instead of `_lifetime` and `ptr`, 
    // but that way it could not implement `Copy`
    _lifetime: PhantomData<&'s mut T>,
    _mutability: PhantomData<*const M>,
    ptr: NonNull<T>
}

impl<'s, M: Mutability, T: ?Sized> GenRef<'s, M, T> {
    pub unsafe fn from_erased_unchecked(erased: ErasedMutabilityRef<'s, T>) -> Self {
        Self{
            _lifetime: PhantomData,
            _mutability: PhantomData,
            ptr: erased.as_ptr()
        }
    }
    pub fn into_erased(genref: Self) -> ErasedMutabilityRef<'s, T> {
        unsafe{
            ErasedMutabilityRef::new_unchecked(genref.ptr)
        }
    }

    /// Casts the reference into a `NonNull` pointer.
    /// 
    /// The returned pointer is guaranteed to be valid for reads for `'s`, and also for writes if `M` is `Mutable`.
    /// 
    /// The `GenRef` must not be used while the pointer is active. 
    /// The exact semantics of this depends on the memory model adopted by Rust.
    pub fn as_ptr(genref: &Self) -> NonNull<T> {
        genref.ptr
    }

    /// Converts a `&mut T` into a generic `GenRef<'_, M, T>`, downgrading the reference if `M` is `Shared`.
    /// 
    /// If `M` is `Mutable` it behaves exactly the same way as `gen_from_mut` without requiring a proof for mutability.
    /// In this case, the difference between the two is purely semantic: if you have proof that `M` is `Mutable`, you should use `gen_from_mut`.
    pub fn gen_from_mut_downgrading(reference: &'s mut T) -> Self {
        let erased = ErasedMutabilityRef::from(reference);

        unsafe {
            Self::from_erased_unchecked(erased)
        }
    }

    /// Converts a generic `GenRef<'_, M, T>` into `&T`, downgrading the reference if `M` is `Mutable`.
    /// 
    /// If `M` is `Shared` it behaves exactly the same way as `gen_into_shared` without requiring a proof for sharedness.
    /// In this case, the difference between the two is purely semantic: if you have proof that `M` is `Shared`, you should use `gen_into_shared`.
    pub fn gen_into_shared_downgrading(genref: Self) -> &'s T  {
        Self::into_erased(genref).into_ref()
    }

    /// Converts a generic `GenRef<'_, M, T>` into `&mut T`. 
    /// This is available in a generic context.
    /// 
    /// Once the transformations are done, the result can be converted back into a `GenRef` using the `gen_from_mut` function.
    /// 
    /// The conversion requires that `M` is `Mutable`, this must be proven by passing an `IsMutable<M>` value.
    /// That can be obtained by `match`ing on `M::mutability()`.
    pub fn gen_into_mut(genref: Self, _proof: IsMutable<M>) -> &'s mut T {
        let erased = GenRef::into_erased(genref);

        unsafe{
            erased.into_mut()
        }
    }
    /// Converts a `&mut T` into a generic `GenRef<'_, M, T>`. 
    /// This is available in a generic context.
    /// 
    /// The conversion requires that `M` is `Mutable`, this must be proven by passing an `IsMutable<M>` value.
    /// That can be obtained by `match`ing on `M::mutability()`.
    /// 
    /// If you want to force the conversion even if `M` is `Shared`, you can use the `gen_from_mut_downgrading` function.
    pub fn gen_from_mut(reference: &'s mut T, _proof: IsMutable<M>) -> Self {
        let erased = ErasedMutabilityRef::from(reference);
        unsafe{
            GenRef::from_erased_unchecked(erased)
        }
    }

    /// Converts a generic `GenRef<'_, M, T>` into `&T`. 
    /// This is available in a generic context.
    /// 
    /// Once the transformations are done, the result can be converted back into a `GenRef` using the `gen_from_shared` function.
    /// 
    /// The conversion requires that `M` is `Shared`, this must be proven by passing an `IsShared<M>` value.
    /// That can be obtained by `match`ing on `M::mutability()`.
    /// 
    /// If you want to force the conversion even if `M` is `Mutable`, you can use the `gen_into_shared_downgrading` function.
    pub fn gen_into_shared(genref: Self, _proof: IsShared<M>) -> &'s T {
        GenRef::into_erased(genref).into_ref()
    }
    /// Converts a `&T` into a generic `GenRef<'_, M, T>`. 
    /// This is available in a generic context.
    /// 
    /// The conversion requires that `M` is `Shared`, this must be proven by passing an `IsShared<M>` value.
    /// That can be obtained by `match`ing on `M::mutability()`.
    pub fn gen_from_shared(reference: &'s T, _proof: IsShared<M>) -> Self {
        let erased = ErasedMutabilityRef::from(reference);
        unsafe{
            GenRef::from_erased_unchecked(erased)
        }
    }
    /// Generically reborrows a `GenRef`. 
    /// That is, it creates a shorter-lived owned `GenRef` from a `&mut GenRef`.
    /// This is available in a generic context.
    /// 
    /// This requires the variable to be marked `mut`, even if `M` is `Shared` and thus no mutation takes place.
    pub fn reborrow(genref: &mut Self) -> GenRef<'_, M, T> {
        let erased = unsafe{
            ErasedMutabilityRef::new_unchecked(genref.ptr)
        };
        unsafe{
            Self::from_erased_unchecked(erased)
        }
    }

    /// Maps a generic `GenRef` into another one using either `f_mut` or `f_shared`. 
    /// This is available in a generic context.
    /// 
    /// Using this function is usually sufficient.
    /// For mapping over field access, you can use the `field!` macro instead.
    /// If you need more flexibility, you can use the `gen_mut!` macro or `match`ing over `M::mutability()`.
    pub fn map<U: ?Sized>(
        genref: Self, 
        f_mut: impl FnOnce(&mut T) -> &mut U, 
        f_shared: impl FnOnce(&T) -> &U
    ) -> GenRef<'s, M, U> {
        use crate::MutabilityEnum::*;

        match M::mutability() {
            Mutable(proof) => GenRef::gen_from_mut(
                f_mut(GenRef::gen_into_mut(genref, proof)), 
                proof
            ),
            Shared(proof) => GenRef::gen_from_shared(
                f_shared(GenRef::gen_into_shared(genref, proof)), 
                proof
            ),

        }
    }

    /// Generically dereferences the value contained in the `GenRef`.
    /// This is available in a generic context.
    pub fn as_deref(genref: Self) -> GenRef<'s, M, T::Target>
        where T: Deref + DerefMut
    {
        GenRef::map(genref, DerefMut::deref_mut, Deref::deref)
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
        unsafe{
            self.ptr.as_ref()
        }
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
        &**self
    }
}
/// This is only implemented for `GenRef<'_, Mutable, T>`, and is not available in a generic context.
impl<T: ?Sized> BorrowMut<T> for GenRef<'_, Mutable, T> {
    fn borrow_mut(&mut self) -> &mut T {
        &mut **self
    }
}

impl<M: Mutability, T: ?Sized> fmt::Pointer for GenRef<'_, M, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.ptr.fmt(f)
    }
}

/// This implementation requires `T: Sync` even when `M` is `Mutable`. 
/// With specialisation, this requirement could be lifted.
unsafe impl<M: Mutability, T: ?Sized> Send for GenRef<'_, M, T> 
    where T: Send + Sync 
{}


unsafe impl<M: Mutability, T: ?Sized> Sync for GenRef<'_, M, T> 
    where T: Sync 
{}

impl<M: Mutability, T: ?Sized> Hash for GenRef<'_, M, T> 
    where T: Hash
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        Hash::hash(&**self, state)
    }
}

macro_rules! impl_fmt_traits {
    ($($trait:ident),+) => {
        $(
            impl<'s, M: Mutability, T: ?Sized> fmt::$trait for GenRef<'s, M, T> 
                where T: fmt::$trait
            {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    T::fmt(&**self, f)
                }
            }
        )+
    };
}
impl_fmt_traits!(Debug, Display, LowerExp, UpperExp, Binary, Octal, LowerHex, UpperHex);

macro_rules! impl_partial_eq_ord_for_refs {
    ($rhs_ty: ident) => {
        impl<MT: Mutability, T: ?Sized, U: ?Sized> PartialEq<$rhs_ty<'_, U>> for GenRef<'_, MT, T> 
            where T: PartialEq<U>
        {
            fn eq(&self, other: & $rhs_ty<'_, U>) -> bool {
                T::eq(&**self, &**other)
            }
        }

        impl<MT: Mutability, T: ?Sized, U: ?Sized> PartialOrd<$rhs_ty<'_, U>> for GenRef<'_, MT, T> 
            where T: PartialOrd<U>
        {
            fn partial_cmp(&self, other: & $rhs_ty<'_, U>) -> Option<Ordering> {
                T::partial_cmp(&**self, &**other)
            }
        }
    };
}

type Ref<'s, T> = &'s T;
impl_partial_eq_ord_for_refs!(Ref);
type MutRef<'s, T> = &'s mut T;
impl_partial_eq_ord_for_refs!(MutRef);


impl<MT: Mutability, MU: Mutability, T: ?Sized, U: ?Sized> PartialEq<GenRef<'_, MU, U>> for GenRef<'_, MT, T> 
    where T: PartialEq<U>
{
    fn eq(&self, other: &GenRef<'_, MU, U>) -> bool {
        T::eq(&**self, &**other)
    }
}

impl<M: Mutability, T: ?Sized> Eq for GenRef<'_, M, T> 
    where T: Eq 
{}

impl<MT: Mutability, MU: Mutability, T: ?Sized, U: ?Sized> PartialOrd<GenRef<'_, MU, U>> for GenRef<'_, MT, T> 
    where T: PartialOrd<U>
{
    fn partial_cmp(&self, other: &GenRef<'_, MU, U>) -> Option<Ordering> {
        T::partial_cmp(&**self, &**other)
    }
}

impl<MT: Mutability, T: ?Sized> Ord for GenRef<'_, MT, T> 
    where T: Ord
{
    fn cmp(&self, other: &Self) -> Ordering {
        T::cmp(&**self, &**other)
    }
}

#[cfg(any(feature = "std", doc))]
/// This is only implemented for `GenRef<'_, Shared, T>`, and is not available in a generic context.
/// 
/// This is only available with the feature flag `std`.
impl<T: ?Sized> std::net::ToSocketAddrs for GenRef<'_, Shared, T> 
    where T: std::net::ToSocketAddrs
{
    type Iter = T::Iter;
    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        std::net::ToSocketAddrs::to_socket_addrs(&**self)
    }
}

/// This is only implemented for `GenRef<'_, Mutable, T>`, and is not available in a generic context.
impl<T: ?Sized> fmt::Write for GenRef<'_, Mutable, T> 
    where T: fmt::Write
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        T::write_str(&mut **self, s)
    }
}

/// This is only implemented for `GenRef<'_, Mutable, T>`, and is not available in a generic context.
impl<T: ?Sized> Iterator for GenRef<'_, Mutable, T> 
    where T: Iterator
{
    type Item = T::Item;
    fn next(&mut self) -> Option<Self::Item> {
        T::next(&mut **self)
    }
}


/// This is only implemented for `GenRef<'_, Mutable, T>`, and is not available in a generic context.
impl<T: ?Sized> DoubleEndedIterator for GenRef<'_, Mutable, T> 
    where T: DoubleEndedIterator
{
    fn next_back(&mut self) -> Option<Self::Item> {
        T::next_back(&mut **self)
    }
}

/// This is only implemented for `GenRef<'_, Mutable, T>`, and is not available in a generic context.
impl<T: ?Sized> ExactSizeIterator for GenRef<'_, Mutable, T> 
    where T: ExactSizeIterator
{
    fn len(&self) -> usize {
        T::len(&**self)
    }
}

/// This is only implemented for `GenRef<'_, Mutable, T>`, and is not available in a generic context.
impl<T: ?Sized> FusedIterator for GenRef<'_, Mutable, T>
    where T: FusedIterator
{}

#[cfg(any(feature = "std", doc))]
/// This is only implemented for `GenRef<'_, Mutable, T>`, and is not available in a generic context.
/// 
/// This is only available with the feature flag `std`.
impl<T: ?Sized> std::io::Write for GenRef<'_, Mutable, T> 
    where T: std::io::Write
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        T::write(&mut **self, buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        T::flush(&mut **self)
    }
}

#[cfg(any(feature = "std", doc))]
/// This is only implemented for `GenRef<'_, Mutable, T>`, and is not available in a generic context.
/// 
/// This is only available with the feature flag `std`.
impl<T: ?Sized> std::io::Read for GenRef<'_, Mutable, T> 
    where T: std::io::Read
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        T::read(&mut **self, buf)
    }
}

#[cfg(any(feature = "std", doc))]
/// This is only implemented for `GenRef<'_, Mutable, T>`, and is not available in a generic context.
/// 
/// This is only available with the feature flag `std`.
impl<T: ?Sized> std::io::Seek for GenRef<'_, Mutable, T> 
    where T: std::io::Seek
{
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        T::seek(&mut **self, pos)
    }
}

#[cfg(any(feature = "std", doc))]
/// This is only implemented for `GenRef<'_, Mutable, T>`, and is not available in a generic context.
/// 
/// This is only available with the feature flag `std`.
impl<T: ?Sized> std::io::BufRead for GenRef<'_, Mutable, T> 
    where T: std::io::BufRead
{
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        T::fill_buf(&mut **self)
    }
    fn consume(&mut self, amt: usize) {
        T::consume(&mut **self, amt)
    }
}

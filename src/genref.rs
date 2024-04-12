use core::borrow::{Borrow, BorrowMut};
use core::cmp::Ordering;
use core::fmt;
use core::hash::Hash;
use core::iter::FusedIterator;
use core::ops::{Deref, DerefMut};
use core::marker::PhantomData;
use core::ptr::NonNull;

use crate::mutability::{Mutability, Mutable, Shared, IsMutable, IsShared};

#[repr(transparent)]
pub struct GenRef<'s, M: Mutability, T: ?Sized> {
    // This could contain an `ErasedMutRef` instead of `_lifetime` and `ptr`, 
    // but that way it could not implement `Copy`
    _lifetime: PhantomData<&'s mut T>,
    _mutability: PhantomData<*const M>,
    ptr: NonNull<T>
}

impl<'s, M: Mutability, T: ?Sized> GenRef<'s, M, T> {
    pub unsafe fn from_ptr_unchecked(ptr: NonNull<T>) -> Self {
        Self{
            _lifetime: PhantomData,
            _mutability: PhantomData,
            ptr
        }
    }

    pub fn as_ptr(genref: &Self) -> NonNull<T> {
        genref.ptr
    }

    pub fn gen_from_mut_downgrading(reference: &'s mut T) -> Self {
        let ptr = NonNull::from(reference);

        unsafe {
            Self::from_ptr_unchecked(ptr)
        }
    }
    pub fn gen_into_shared_downgrading(genref: Self) -> &'s T  {
        let ptr = GenRef::as_ptr(&genref);

        unsafe{
            ptr.as_ref()
        }
    }

    pub fn gen_into_mut(genref: Self, _proof: IsMutable<M>) -> &'s mut T {
        let mut ptr = GenRef::as_ptr(&genref);

        unsafe{
            ptr.as_mut()
        }
    }
    pub fn gen_from_mut(reference: &'s mut T, _proof: IsMutable<M>) -> Self {
        GenRef::gen_from_mut_downgrading(reference)
    }

    pub fn gen_into_shared(genref: Self, _proof: IsShared<M>) -> &'s T {
        GenRef::gen_into_shared_downgrading(genref)
    }
    pub fn gen_from_shared(reference: &'s T, _proof: IsShared<M>) -> Self {
        let ptr = NonNull::from(reference);

        unsafe {
            Self::from_ptr_unchecked(ptr)
        }
    }

    pub fn reborrow(genref: &mut Self) -> GenRef<'_, M, T> {
        unsafe {
            Self::from_ptr_unchecked(genref.ptr)
        }
    }

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

    pub fn as_deref(genref: Self) -> GenRef<'s, M, T::Target>
        where T: Deref + DerefMut
    {
        GenRef::map(genref, DerefMut::deref_mut, Deref::deref)
    }
}

pub trait GenRefMethods<'s, M: Mutability, T: ?Sized> {
    fn as_ptr(&self) -> NonNull<T>;

    fn gen_into_shared_downgrading(self) -> &'s T;

    fn gen_into_mut(self, _proof: IsMutable<M>) -> &'s mut T;

    fn gen_into_shared(self, _proof: IsShared<M>) -> &'s T ;
    fn gen_from_shared(reference: &'s T, _proof: IsShared<M>) -> Self;
    fn reborrow(&mut self) -> GenRef<'_, M, T>;

    fn map<U: ?Sized>(
        self, 
        f_mut: impl FnOnce(&mut T) -> &mut U, 
        f_shared: impl FnOnce(&T) -> &U
    ) -> GenRef<'s, M, U>;

    fn as_deref(self) -> GenRef<'s, M, T::Target>
        where T: Deref + DerefMut;
}
impl<'s, M: Mutability, T: ?Sized> GenRefMethods<'s, M, T> for GenRef<'s, M, T> {
    fn as_ptr(&self) -> NonNull<T> {
        self.ptr
    }

    fn gen_into_shared_downgrading(self) -> &'s T  {
        let ptr = GenRef::as_ptr(&self);

        unsafe{
            ptr.as_ref()
        }
    }

    fn gen_into_mut(self, _proof: IsMutable<M>) -> &'s mut T {
        let mut ptr = GenRef::as_ptr(&self);

        unsafe{
            ptr.as_mut()
        }
    }

    fn gen_into_shared(self, _proof: IsShared<M>) -> &'s T {
        GenRef::gen_into_shared_downgrading(self)
    }
    fn gen_from_shared(reference: &'s T, _proof: IsShared<M>) -> Self {
        let ptr = NonNull::from(reference);

        unsafe {
            Self::from_ptr_unchecked(ptr)
        }
    }

    fn reborrow(&mut self) -> GenRef<'_, M, T> {
        unsafe {
            Self::from_ptr_unchecked(self.ptr)
        }
    }

    fn map<U: ?Sized>(
        self, 
        f_mut: impl FnOnce(&mut T) -> &mut U, 
        f_shared: impl FnOnce(&T) -> &U
    ) -> GenRef<'s, M, U> {
        use crate::MutabilityEnum::*;

        match M::mutability() {
            Mutable(proof) => GenRef::gen_from_mut(
                f_mut(GenRef::gen_into_mut(self, proof)), 
                proof
            ),
            Shared(proof) => GenRef::gen_from_shared(
                f_shared(GenRef::gen_into_shared(self, proof)), 
                proof
            ),

        }
    }

    fn as_deref(self) -> GenRef<'s, M, T::Target>
        where T: Deref + DerefMut
    {
        GenRef::map(self, DerefMut::deref_mut, Deref::deref)
    }
}

impl<'s, T: ?Sized> GenRef<'s, Shared, T> {
    pub fn into_shared(genref: Self) -> &'s T {
        Self::gen_into_shared(genref, Shared::mutability())
    }
}
impl<'s, T: ?Sized> GenRef<'s, Mutable, T> {
    pub fn into_mut(genref: Self) -> &'s mut T {
        Self::gen_into_mut(genref, Mutable::mutability())
    }
}

impl<'s, T: ?Sized> From<&'s T> for GenRef<'s, Shared, T> {
    fn from(reference: &'s T) -> Self {
        GenRef::gen_from_shared(reference, Shared::mutability())
    }
}
impl<'s, T: ?Sized> From<&'s mut T> for GenRef<'s, Mutable, T> {
    fn from(reference: &'s mut T) -> Self {
        GenRef::gen_from_mut(reference, Mutable::mutability())
    }
}

impl<M: Mutability, T: ?Sized> Deref for GenRef<'_, M, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe{
            self.ptr.as_ref()
        }
    }
}
impl<T: ?Sized> DerefMut for GenRef<'_, Mutable, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        GenRef::into_mut(GenRef::reborrow(self))
    }
}

impl<T: ?Sized> Clone for GenRef<'_, Shared, T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: ?Sized> Copy for GenRef<'_, Shared, T> {}

impl<M: Mutability, T: ?Sized> Borrow<T> for GenRef<'_, M, T> {
    fn borrow(&self) -> &T {
        &**self
    }
}
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

// With specialization, the `T: Sync` bound could be dropped if `M` is `Mutable`
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

#[cfg(feature = "std")]
impl<M: Mutability, T: ?Sized> std::net::ToSocketAddrs for GenRef<'_, Shared, T> {
    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        std::net::ToSocketAddrs::to_socket_addrs(&**self)
    }
}

impl<T: ?Sized> fmt::Write for GenRef<'_, Mutable, T> 
    where T: fmt::Write
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        T::write_str(&mut **self, s)
    }
}

impl<T: ?Sized> Iterator for GenRef<'_, Mutable, T> 
    where T: Iterator
{
    type Item = T::Item;
    fn next(&mut self) -> Option<Self::Item> {
        T::next(&mut **self)
    }
}


impl<T: ?Sized> DoubleEndedIterator for GenRef<'_, Mutable, T> 
    where T: DoubleEndedIterator
{
    fn next_back(&mut self) -> Option<Self::Item> {
        T::next_back(&mut **self)
    }
}

impl<T: ?Sized> ExactSizeIterator for GenRef<'_, Mutable, T> 
    where T: ExactSizeIterator
{
    fn len(&self) -> usize {
        T::len(&**self)
    }
}

impl<T: ?Sized> FusedIterator for GenRef<'_, Mutable, T>
    where T: FusedIterator
{}

#[cfg(feature = "std")]
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

#[cfg(feature = "std")]
impl<T: ?Sized> std::io::Read for GenRef<'_, Mutable, T> 
    where T: std::io::Read
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        T::read(&mut **self, buf)
    }
}

#[cfg(feature = "std")]
impl<T: ?Sized> std::io::Seek for GenRef<'_, Mutable, T> 
    where T: std::io::Seek
{
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        T::seek(&mut **self, pos)
    }
}

#[cfg(feature = "std")]
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

use core::cmp::Ordering;
use core::fmt;
use core::hash::Hash;
use core::iter::{DoubleEndedIterator, FusedIterator, Iterator};

#[cfg(any(feature = "std", doc))]
extern crate std;

#[allow(unused_imports)]
use crate::{GenRef, Mutability, Mutable, Shared};

impl<M: Mutability, T: ?Sized> Hash for GenRef<'_, M, T>
where
    T: Hash,
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
        where
            T: PartialEq<U>,
        {
            fn eq(&self, other: &$rhs_ty<'_, U>) -> bool {
                T::eq(&**self, &**other)
            }
        }

        impl<MT: Mutability, T: ?Sized, U: ?Sized> PartialOrd<$rhs_ty<'_, U>> for GenRef<'_, MT, T>
        where
            T: PartialOrd<U>,
        {
            fn partial_cmp(&self, other: &$rhs_ty<'_, U>) -> Option<Ordering> {
                T::partial_cmp(&**self, &**other)
            }
        }
    };
}

type Ref<'s, T> = &'s T;
impl_partial_eq_ord_for_refs!(Ref);
type MutRef<'s, T> = &'s mut T;
impl_partial_eq_ord_for_refs!(MutRef);

impl<MT: Mutability, MU: Mutability, T: ?Sized, U: ?Sized> PartialEq<GenRef<'_, MU, U>>
    for GenRef<'_, MT, T>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &GenRef<'_, MU, U>) -> bool {
        T::eq(&**self, &**other)
    }
}

impl<M: Mutability, T: ?Sized> Eq for GenRef<'_, M, T> where T: Eq {}

impl<MT: Mutability, MU: Mutability, T: ?Sized, U: ?Sized> PartialOrd<GenRef<'_, MU, U>>
    for GenRef<'_, MT, T>
where
    T: PartialOrd<U>,
{
    fn partial_cmp(&self, other: &GenRef<'_, MU, U>) -> Option<Ordering> {
        T::partial_cmp(&**self, &**other)
    }
}

impl<MT: Mutability, T: ?Sized> Ord for GenRef<'_, MT, T>
where
    T: Ord,
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
where
    T: std::net::ToSocketAddrs,
{
    type Iter = T::Iter;
    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        std::net::ToSocketAddrs::to_socket_addrs(&**self)
    }
}

/// This is only implemented for `GenRef<'_, Mutable, T>`, and is not available in a generic context.
impl<T: ?Sized> fmt::Write for GenRef<'_, Mutable, T>
where
    T: fmt::Write,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        T::write_str(&mut **self, s)
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        T::write_char(&mut **self, c)
    }

    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> fmt::Result {
        T::write_fmt(&mut **self, args)
    }
}

/// This is only implemented for `GenRef<'_, Mutable, T>`, and is not available in a generic context.
impl<T: ?Sized> Iterator for GenRef<'_, Mutable, T>
where
    T: Iterator,
{
    type Item = T::Item;
    fn next(&mut self) -> Option<Self::Item> {
        T::next(&mut **self)
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        T::nth(&mut **self, n)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        T::size_hint(&**self)
    }
}

/// This is only implemented for `GenRef<'_, Mutable, T>`, and is not available in a generic context.
impl<T: ?Sized> DoubleEndedIterator for GenRef<'_, Mutable, T>
where
    T: DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        T::next_back(&mut **self)
    }
}

/// This is only implemented for `GenRef<'_, Mutable, T>`, and is not available in a generic context.
impl<T: ?Sized> ExactSizeIterator for GenRef<'_, Mutable, T>
where
    T: ExactSizeIterator,
{
    fn len(&self) -> usize {
        T::len(&**self)
    }
}

/// This is only implemented for `GenRef<'_, Mutable, T>`, and is not available in a generic context.
impl<T: ?Sized> FusedIterator for GenRef<'_, Mutable, T> where T: FusedIterator {}

#[cfg(any(feature = "std", doc))]
/// This is only implemented for `GenRef<'_, Mutable, T>`, and is not available in a generic context.
///
/// This is only available with the feature flag `std`.
impl<T: ?Sized> std::io::Write for GenRef<'_, Mutable, T>
where
    T: std::io::Write,
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
where
    T: std::io::Read,
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
where
    T: std::io::Seek,
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
where
    T: std::io::BufRead,
{
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        T::fill_buf(&mut **self)
    }
    fn consume(&mut self, amt: usize) {
        T::consume(&mut **self, amt)
    }
}

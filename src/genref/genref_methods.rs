use super::docs_for;
use crate::{GenRef, IsMutable, IsShared, Mutability};
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

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
    #[inline(always)]
    fn as_ptr(&self) -> NonNull<T> {
        GenRef::as_ptr(self)
    }

    #[inline(always)]
    fn gen_into_shared_downgrading(self) -> &'s T {
        GenRef::gen_into_shared_downgrading(self)
    }
    #[inline(always)]
    fn gen_into_mut(self, proof: IsMutable<M>) -> &'s mut T {
        GenRef::gen_into_mut(self, proof)
    }

    #[inline(always)]
    fn gen_into_shared(self, proof: IsShared<M>) -> &'s T {
        GenRef::gen_into_shared(self, proof)
    }

    #[inline(always)]
    fn reborrow(&mut self) -> GenRef<'_, M, T> {
        GenRef::reborrow(self)
    }

    #[inline(always)]
    fn map<U: ?Sized>(
        self,
        f_mut: impl FnOnce(&mut T) -> &mut U,
        f_shared: impl FnOnce(&T) -> &U,
    ) -> GenRef<'s, M, U> {
        GenRef::map(self, f_shared, f_mut)
    }

    #[inline(always)]
    fn map_deref(self) -> GenRef<'s, M, T::Target>
    where
        T: Deref + DerefMut,
    {
        GenRef::map_deref(self)
    }

    #[inline(always)]
    fn deref(&self) -> &T {
        self
    }
}

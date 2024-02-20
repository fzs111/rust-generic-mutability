use core::ops::{Deref, DerefMut};
use core::marker::PhantomData;

use crate::erased_mut_ref::ErasedMutRef;
use crate::mutability::{Mutability, Mutable, Shared, IsMutable, IsShared};

#[repr(transparent)]
pub struct GenRef<'s, M: Mutability, T: ?Sized> {
    _mutability: PhantomData<*const M>,
    ptr: ErasedMutRef<'s, T>
}

impl<'s, M: Mutability, T: ?Sized> GenRef<'s, M, T> {
    pub unsafe fn from_erased_unchecked(erased: ErasedMutRef<'s, T>) -> Self {
        Self{
            _mutability: PhantomData,
            ptr: erased
        }
    }
    pub fn into_erased(genref: Self) -> ErasedMutRef<'s, T> {
        genref.ptr
    }

    pub fn gen_from_mut_downgrading(reference: &'s mut T) -> Self {
        let erased = ErasedMutRef::from(reference);

        unsafe {
            Self::from_erased_unchecked(erased)
        }
    }
    pub fn gen_into_shared_downgrading(genref: Self) -> &'s T  {
        Self::into_erased(genref).into_ref()
    }

    pub fn gen_into_mut(genref: Self, _proof: IsMutable<M>) -> &'s mut T {
        let erased = GenRef::into_erased(genref);

        unsafe{
            erased.into_mut()
        }
    }
    pub fn gen_from_mut(reference: &'s mut T, _proof: IsMutable<M>) -> Self {
        let erased = ErasedMutRef::from(reference);
        unsafe{
            GenRef::from_erased_unchecked(erased)
        }
    }

    pub fn gen_into_shared(genref: Self, _proof: IsShared<M>) -> &'s T {
        GenRef::into_erased(genref).into_ref()
    }
    pub fn gen_from_shared(reference: &'s T, _proof: IsShared<M>) -> Self {
        let erased = ErasedMutRef::from(reference);
        unsafe{
            GenRef::from_erased_unchecked(erased)
        }
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

impl<'s, M: Mutability, T: ?Sized> Deref for GenRef<'s, M, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.ptr.as_ref()
    }
}
impl<'s, T: ?Sized> DerefMut for GenRef<'s, Mutable, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        todo!()
    }
}


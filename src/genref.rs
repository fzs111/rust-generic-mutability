use core::ops::Deref;
use core::marker::PhantomData;

use crate::{mutability::{IsMutable, IsShared}, ErasedMutRef, Mutability, Mutable, Shared};

#[repr(transparent)]
pub struct GenRef<'s, M: Mutability, T: ?Sized> {
    _mutability: PhantomData<*const M>,
    ptr: ErasedMutRef<'s, T>
}

impl<'s, M: Mutability, T: ?Sized> GenRef<'s, M, T> {
    pub unsafe fn new_unchecked(erased: ErasedMutRef<'s, T>) -> Self {
        Self{
            _mutability: PhantomData,
            ptr: erased
        }
    }
    pub fn erase(genref: Self) -> ErasedMutRef<'s, T> {
        genref.ptr
    }
    
    pub fn gen_to_mut(genref: Self, _proof: IsMutable<M>) -> &'s mut T {
        let erased = GenRef::erase(genref);

        unsafe{
            erased.into_mut()
        }
    }
    pub fn mut_to_gen(reference: &'s mut T, _proof: IsMutable<M>) -> GenRef<'s, M, T> {
        let erased = ErasedMutRef::from(reference);
        unsafe{
            GenRef::new_unchecked(erased)
        }
    }

    pub fn gen_to_shared(genref: Self) -> &'s T {
        GenRef::erase(genref).into_ref()
    }
    pub fn shared_to_gen(reference: &'s T, _proof: IsShared<M>) -> GenRef<'s, M, T> {
        let erased = ErasedMutRef::from(reference);
        unsafe{
            GenRef::new_unchecked(erased)
        }
    }
}

impl<'s, M: Mutability, T: ?Sized> Deref for GenRef<'s, M, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.ptr.as_ref()
    }
}

impl<'s, T: ?Sized> From<&'s mut T> for GenRef<'s, Mutable, T> {
    fn from(reference: &'s mut T) -> Self {
        GenRef::mut_to_gen(reference, Mutable::mutability())
    }
}

impl<'s, T: ?Sized> From<&'s T> for GenRef<'s, Shared, T> {
    fn from(reference: &'s T) -> Self {
        GenRef::shared_to_gen(reference, Shared::mutability())
    }
}

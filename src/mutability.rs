use core::marker::PhantomData;

mod seal{
    use crate::{Mutable, Shared};

    pub trait Sealed {}
    impl Sealed for Mutable {}
    impl Sealed for Shared {}
}

pub unsafe trait Mutability: Copy + Sized + seal::Sealed {
    fn mutability() -> MutabilityEnum<Self>;
}

#[derive(Clone, Copy)]
pub enum Shared {}
unsafe impl Mutability for Shared {
    fn mutability() -> MutabilityEnum<Self> {
        let proof = unsafe{
            IsShared::new()
        };

        MutabilityEnum::Shared(proof)
    }
}
impl Shared {
    pub fn mutability() -> IsShared<Shared> {
        unsafe{
            IsShared::new()
        }
    }
}

#[derive(Clone, Copy)]
pub enum Mutable {}
unsafe impl Mutability for Mutable {
    fn mutability() -> MutabilityEnum<Self> {
        let proof = unsafe{
            IsMutable::new()
        };

        MutabilityEnum::Mutable(proof)
    }
}
impl Mutable {
    pub fn mutability() -> IsMutable<Mutable> {
        unsafe{
            IsMutable::new()
        }
    }
}


#[derive(Clone, Copy)]
pub struct IsMutable<M: Mutability>(PhantomData<M>);

impl<M: Mutability> IsMutable<M> {
    pub(crate) unsafe fn new() -> Self {
        IsMutable(PhantomData)
    }
}

#[derive(Clone, Copy)]
pub struct IsShared<M: Mutability>(PhantomData<M>);

impl<M: Mutability> IsShared<M> {
    pub(crate) unsafe fn new() -> Self {
        IsShared(PhantomData)
    }
}

#[derive(Clone, Copy)]
pub enum MutabilityEnum<M: Mutability> {
    Mutable(IsMutable<M>),
    Shared(IsShared<M>)
}
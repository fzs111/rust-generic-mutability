use core::marker::PhantomData;

pub trait Mutability: Copy + Sized {
    fn mutability() -> MutabilityEnum<Self>;
}

#[derive(Clone, Copy)]
pub enum Shared {}
impl Mutability for Shared {
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
pub struct Mutable<'a> (PhantomData<&'a mut ()>);

impl<'m> Mutability for Mutable<'m> {
    fn mutability() -> MutabilityEnum<Self> {
        let proof = unsafe{
            IsMutable::new()
        };

        MutabilityEnum::Mutable(proof)
    }
}
impl Mutable<'_> {
    pub fn mutability() -> IsMutable<Self> {
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
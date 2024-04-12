use core::marker::PhantomData;

mod seal{
    use crate::{Mutable, Shared};

    pub trait Sealed {}
    impl Sealed for Mutable {}
    impl Sealed for Shared {}
}

/// This trait is used as a bound on generic mutability parameters.
/// 
/// This trait is implemented by two types, `Shared` and `Mutable`, and it is sealed so no other types may implement it.
/// 
/// Not to be confused with `MutabilityEnum<M>`, which represents a proof about a generic mutability parameter.
/// 
/// Note that while mutability parameters are implemented as type parameters, they represent an entirely different kind of generic parameter. 
/// For this reason, the `M: Mutability` bound should be applied even in struct definitions where bounds are generally discouraged.
pub unsafe trait Mutability: Copy + Sized + seal::Sealed {

    /// The result of this method lets you match over the mutability values to obtain a proof, which can be used to access features that are only available for one mutability.
    /// 
    /// Most notably, the `GenRef::gen_{into,from}_{mut,shared}` methods require a proof of this form.
    fn mutability() -> MutabilityEnum<Self>;
}

/// Represents the mutability of a shared reference, `&T`.
/// 
/// Along with `Mutable`, this is one of the two types that can be used as a generic mutability parameter. 
/// It is an empty type because it is always used as a generic parameter and never as a value.
/// 
/// Just as with `&T`, interior mutability types may change while behind a reference of `Shared` mutability.
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


/// Represents the mutability of a unique reference, `&mut T`.
/// 
/// Along with `Shared`, this is one of the two types that can be used as a generic mutability parameter. 
/// It is an empty type because it is always used as a generic parameter and never as a value.
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

/// The existance of a value of this type guarantees that a specific mutability parameter `M` is `Mutable`. 
/// You can obtain this value by matching over `M::mutability()`.
/// 
/// The most notable API that requires this is `GenRef::gen_{into,from}_mut`.
#[derive(Clone, Copy)]
pub struct IsMutable<M: Mutability>(PhantomData<M>);

impl<M: Mutability> IsMutable<M> {
    pub(crate) unsafe fn new() -> Self {
        IsMutable(PhantomData)
    }
}

/// The existance of a value of this type guarantees that a specific mutability parameter `M` is `Shared`. 
/// You can obtain this value by matching over `M::mutability()`.
/// 
/// The most notable API that requires this is `GenRef::gen_{into,from}_shared`.
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
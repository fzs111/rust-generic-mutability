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
        // This is not a recursive call, but a call to the inherent method.
        #[deny(unconditional_recursion)]
        let proof = Shared::mutability();

        MutabilityEnum::Shared(proof)
    }
}
impl Shared {
    /// This method returns a proof for the shared-ness of `Shared`. 
    /// Note: this method shadows `<Shared as Mutability>::mutability()`, which returns the same proof wrapped in `MutabilityEnum::Shared`.
    /// If you have access to this method (i.e. in non-generic contexts), you should not need `<Shared as Mutability>::mutability()`.
    pub fn mutability() -> IsShared<Shared> {
        unsafe{
            // SAFETY: `M` is `Shared`
            IsShared::new()
        }
    }
}


/// Represents the mutability of a mutable (unique) reference, `&mut T`.
/// 
/// Along with `Shared`, this is one of the two types that can be used as a generic mutability parameter. 
/// It is an empty type because it is always used as a generic parameter and never as a value.
#[derive(Clone, Copy)]
pub enum Mutable {}
unsafe impl Mutability for Mutable {
    fn mutability() -> MutabilityEnum<Self> {
        // This is not a recursive call, but a call to the inherent method.
        #[deny(unconditional_recursion)]
        let proof = Mutable::mutability();

        MutabilityEnum::Mutable(proof)
    }
}
impl Mutable {
    /// This method returns a proof for the mutable-ness of `Mutable`. 
    /// Note: this method shadows `<Mutable as Mutability>::mutability()`, which returns the same proof wrapped in `MutabilityEnum::Mutable`.
    /// If you have access to this method (i.e. in non-generic contexts), you should not need `<Mutable as Mutability>::mutability()`.
    pub fn mutability() -> IsMutable<Mutable> {
        unsafe{
            // SAFETY: `M` is `Mutable`
            IsMutable::new()
        }
    }
}

/// The existence of a value of this type guarantees that a specific mutability parameter `M` is `Mutable`. 
/// Unsafe code may rely on this guarantee.
/// You can obtain this value by matching over `M::mutability()`.
/// 
/// The most notable API that requires this is `GenRef::gen_{into,from}_mut`.
#[derive(Clone, Copy)]
pub struct IsMutable<M: Mutability>(PhantomData<M>);

impl<M: Mutability> IsMutable<M> {
    // SAFETY: `M` must be `Mutable`
    pub(crate) unsafe fn new() -> Self {
        IsMutable(PhantomData)
    }
}

/// The existence of a value of this type guarantees that a specific mutability parameter `M` is `Shared`. 
/// Unsafe code may rely on this guarantee.
/// You can obtain this value by matching over `M::mutability()`.
/// 
/// The most notable API that requires this is `GenRef::gen_{into,from}_shared`.
#[derive(Clone, Copy)]
pub struct IsShared<M: Mutability>(PhantomData<M>);

impl<M: Mutability> IsShared<M> {
    // SAFETY: `M` must be `Shared`
    pub(crate) unsafe fn new() -> Self {
        IsShared(PhantomData)
    }
}

/// This enum makes it possible to `match` over a mutability parameter.
/// Not to be confused with the `Mutability` trait, which is used as a bound for mutability parameters; and `Shared` and `Mutable`, which are values of the mutability parameters.
/// 
/// A value of this type can be obtained from the `Mutability::mutability()` method.
/// 
/// Each variant contains a proof about the value of mutability parameter `M`.
/// 
/// Note that the only valid value of type `MutabilityEnum<Shared>` is `MutabilityEnum::Shared(IsShared)`, and the only value of type `MutabilityEnum<Mutable>` is `MutabilityEnum::Mutable(IsMutable)`
#[derive(Clone, Copy)]
pub enum MutabilityEnum<M: Mutability> {
    /// Contains a proof that `M` is `Mutable`. `MutabilityEnum::Mutable` (this enum variant) is not to be confused with `Mutable` (type implementing `Mutability`).
    Mutable(IsMutable<M>),
    /// Contains a proof that `M` is `Shared`. `MutabilityEnum::Shared` (this enum variant) is not to be confused with `Shared` (type implementing `Mutability`).
    Shared(IsShared<M>)
}
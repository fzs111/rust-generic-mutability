use crate::{IsMutable, IsShared, Mutability};

pub trait GenInto<M: Mutability> {
    type Shared;
    type Mutable;

    fn into_shared(self, proof: IsShared<M>) -> Self::Shared;
    fn into_mut(self, proof: IsMutable<M>) -> Self::Mutable;
}

pub trait GenFrom<M: Mutability, Shared, Mutable> {
    fn from_shared(from: Shared, proof: IsShared<M>) -> Self;
    fn from_mut(from: Mutable, proof: IsMutable<M>) -> Self;
}

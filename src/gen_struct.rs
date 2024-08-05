use core::{marker::PhantomData, mem::ManuallyDrop};

use crate::{field, gen_mut, GenInto, GenRef, IsMutable, IsShared, Mutability};

pub struct GenStruct<M: Mutability, T, U> {
    _mutability: PhantomData<*const M>,
    inner: GenStructInner<T, U>,
}

union GenStructInner<T, U> {
    shared: ManuallyDrop<T>,
    mutable: ManuallyDrop<U>,
}
/*
impl<M: Mutability, T, U> GenStruct<M, T, U> {
    fn get<MR: Mutability>(gen_struct: GenRef<'_, MR, Self>) -> GenRef<'_, >
}
*/
impl<M: Mutability, T, U> Drop for GenStruct<M, T, U> {
    fn drop(&mut self) {
        gen_mut!(M => {
            let md = unsafe { &mut switch_shared_mut!(self.inner.shared, self.inner.mutable) };

            unsafe { ManuallyDrop::drop(md); }
        })
    }
}

impl<M: Mutability, Sh, Mut> GenInto<M> for GenStruct<M, Sh, Mut> {
    type Shared = Sh;
    type Mutable = Mut;

    fn into_shared(self, _proof: IsShared<M>) -> Self::Shared {
        // We are forgetting self, because we are moving out of it and so the destructor must not run
        let mut md_self = ManuallyDrop::new(self);

        let md = unsafe { &mut md_self.inner.shared };

        unsafe { ManuallyDrop::take(md) }
    }
    fn into_mut(self, _proof: IsMutable<M>) -> Self::Mutable {
        // We are forgetting self, because we are moving out of it and so the destructor must not run
        let mut md_self = ManuallyDrop::new(self);

        let md = unsafe { &mut md_self.inner.mutable };

        unsafe { ManuallyDrop::take(md) }
    }
}

impl<'s, MOuter: Mutability, MInner: Mutability, Sh, Mut> GenInto<MInner>
    for GenRef<'s, MOuter, GenStruct<MInner, Sh, Mut>>
{
    type Shared = GenRef<'s, MOuter, Sh>;
    type Mutable = GenRef<'s, MOuter, Mut>;

    fn into_shared(self, _proof: IsShared<MInner>) -> Self::Shared {
        unsafe { field!(&gen self.inner.shared) }
    }
    fn into_mut(self, _proof: IsMutable<MInner>) -> Self::Mutable {
        unsafe { field!(&gen self.inner.mutable) }
    }
}

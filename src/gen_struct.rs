use core::{marker::PhantomData, mem::ManuallyDrop};

use crate::{field, gen_mut, GenFrom, GenInto, GenRef, IsMutable, IsShared, Mutability};

pub struct GenStruct<M: Mutability, Sh, Mut> {
    _mutability: PhantomData<*const M>,
    inner: GenStructInner<Sh, Mut>,
}

union GenStructInner<Sh, Mut> {
    shared: ManuallyDrop<Sh>,
    mutable: ManuallyDrop<Mut>,
}

impl<M: Mutability, Sh, Mut> Drop for GenStruct<M, Sh, Mut> {
    fn drop(&mut self) {
        gen_mut!(M => {
            let md = unsafe { &mut switch_shared_mut!(self.inner.shared, self.inner.mutable) };

            unsafe { ManuallyDrop::drop(md) }
        })
    }
}

impl<M: Mutability, Sh, Mut> GenStruct<M, Sh, Mut> {
    pub fn as_ref<MOuter: Mutability>(
        gen_struct: GenRef<'_, MOuter, GenStruct<M, Sh, Mut>>,
    ) -> GenStructAsRef<GenRef<'_, MOuter, GenStruct<M, Sh, Mut>>> {
        GenStructAsRef(gen_struct)
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

pub struct GenStructAsRef<T>(T);

impl<'s, MOuter: Mutability, MInner: Mutability, Sh, Mut> GenInto<MInner>
    for GenStructAsRef<GenRef<'s, MOuter, GenStruct<MInner, Sh, Mut>>>
{
    type Shared = GenRef<'s, MOuter, Sh>;
    type Mutable = GenRef<'s, MOuter, Mut>;

    fn into_shared(self, _proof: IsShared<MInner>) -> Self::Shared {
        let md = unsafe { field!(&gen {self.0}.inner.shared) };

        GenRef::map_deref(md)
    }
    fn into_mut(self, _proof: IsMutable<MInner>) -> Self::Mutable {
        let md = unsafe { field!(&gen {self.0}.inner.mutable) };

        GenRef::map_deref(md)
    }
}

impl<M: Mutability, Sh, Mut> GenFrom<M, Sh, Mut> for GenStruct<M, Sh, Mut> {
    fn from_shared(from: Sh, _proof: IsShared<M>) -> Self {
        GenStruct {
            _mutability: PhantomData,
            inner: GenStructInner {
                shared: ManuallyDrop::new(from),
            },
        }
    }
    fn from_mut(from: Mut, _proof: IsMutable<M>) -> Self {
        GenStruct {
            _mutability: PhantomData,
            inner: GenStructInner {
                mutable: ManuallyDrop::new(from),
            },
        }
    }
}

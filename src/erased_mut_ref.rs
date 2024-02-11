use core::marker::PhantomData;
use core::ptr::NonNull;

#[repr(transparent)]
pub struct ErasedMutRef<'s, T: ?Sized> {
    _lifetime: PhantomData<&'s mut T>,
    ptr: NonNull<T>
}

impl<'s, T: ?Sized> ErasedMutRef<'s, T> {
    pub unsafe fn new_unchecked(ptr: NonNull<T>) -> Self {
        Self{
            _lifetime: PhantomData,
            ptr
        }
    }
    pub fn as_ref(&self) -> &T {
        unsafe{
            self.ptr.as_ref()
        }
    }
    pub unsafe fn as_mut(&mut self) -> &mut T {
        unsafe{
            self.ptr.as_mut()
        }
    }
    pub fn into_ref(self) -> &'s T {
        unsafe{
            self.ptr.as_ref()
        }
    }
    pub unsafe fn into_mut(mut self) -> &'s mut T {
        unsafe{
            self.ptr.as_mut()
        }
    }
}

impl<'s, T: ?Sized> From<&'s mut T> for ErasedMutRef<'s, T> {
    fn from(reference: &'s mut T) -> Self {
        unsafe{
            Self::new_unchecked(NonNull::from(reference))
        }
    }
}
impl<'s, T: ?Sized> From<&'s T> for ErasedMutRef<'s, T> {
    fn from(reference: &'s T) -> Self {
        unsafe{
            Self::new_unchecked(NonNull::from(reference))
        }
    }
}
#![no_std]

use core::ptr::NonNull;
use core::marker::PhantomData;

mod seal{
    pub trait MutabilitySealed {}
}

use seal::MutabilitySealed;
impl MutabilitySealed for Mutable{}
impl MutabilitySealed for Immutable{}

pub unsafe trait Mutability: MutabilitySealed{

    //TODO: Add safety note
    unsafe fn dispatch<'b, T, U, FM, FIM>(ptr: NonNull<T>, fn_mut: FM, fn_immut: FIM) -> U
    where 
        T: 'b,
        FM: FnOnce(&'b mut T) -> U,
        FIM: FnOnce(&'b T) -> U;
    
    //TODO: Add safety note
    unsafe fn map<'b, 'c, T, U, FM, FIM>(ptr: NonNull<T>, f_mut: FM, f_immut: FIM) -> NonNull<U>
    where 
        T: 'b,
        U: 'c,
        FM: FnOnce(&'b mut T) -> &'c mut U,
        FIM: FnOnce(&'b T) -> &'c U;

    fn is_mutable() -> bool;
}
pub struct Mutable;
unsafe impl Mutability for Mutable{
    
    #[inline]
    unsafe fn dispatch<'b, T, U, FM, FIM>(mut ptr: NonNull<T>, fn_mut: FM, _fn_immut: FIM) -> U
    where 
        T: 'b,
        FM: FnOnce(&'b mut T) -> U,
        FIM: FnOnce(&'b T) -> U
    {
        fn_mut(ptr.as_mut())
    }
    
    #[inline]
    unsafe fn map<'b, 'c, T, U, FM, FIM>(mut ptr: NonNull<T>, f_mut: FM, _f_immut: FIM) -> NonNull<U>
    where 
        T: 'b,
        U: 'c,
        FM: FnOnce(&'b mut T) -> &'c mut U,
        FIM: FnOnce(&'b T) -> &'c U
    {

        f_mut(ptr.as_mut()).into()
    }

    #[inline]
    fn is_mutable() -> bool {
        true
    }
}
pub struct Immutable;
unsafe impl Mutability for Immutable{
    
    #[inline]
    unsafe fn dispatch<'b, T, U, FM, FIM>(ptr: NonNull<T>, _fn_mut: FM, fn_immut: FIM) -> U
    where 
        T: 'b,
        FM: FnOnce(&'b mut T) -> U,
        FIM: FnOnce(&'b T) -> U
    {
        fn_immut(ptr.as_ref())
    }
    
    #[inline]
    unsafe fn map<'b, 'c, T, U, FM, FIM>(ptr: NonNull<T>, _f_mut: FM, f_immut: FIM) -> NonNull<U>
    where 
        T: 'b,
        U: 'c,
        FM: FnOnce(&'b mut T) -> &'c mut U,
        FIM: FnOnce(&'b T) -> &'c U
    {

        f_immut(ptr.as_ref()).into()
    }

    #[inline]
    fn is_mutable() -> bool {
        false
    }
}

pub enum MaybeMutEnum<'a, T>{
    Mutable(&'a mut T),
    Immutable(&'a T)
}

#[repr(transparent)]
pub struct MaybeMut<'a, M: Mutability, T>{
    _lifetime: PhantomData<&'a mut T>,
    _mutability: PhantomData<M>, 
    ptr: NonNull<T>,
}

impl<'a, M: Mutability, T> MaybeMut<'a, M, T> {
    
    #[inline]
    pub unsafe fn new(ptr: NonNull<T>) -> Self {
        //TODO: Add safety note
        Self { 
            _lifetime: PhantomData, 
            _mutability: PhantomData, 
            ptr
        }
    }
    
    #[inline]
    pub fn dispatch<'b, U, FM, FIM>(&'b mut self, fn_mut: FM, fn_immut: FIM) -> U
    where 
        T: 'b,
        FM: FnOnce(&'b mut T) -> U,
        FIM: FnOnce(&'b T) -> U
    {
        unsafe{
            //TODO: Add safety comment
            M::dispatch(self.ptr, fn_mut, fn_immut)
        }
    }

    #[inline]
    pub fn map<'b, 'c, U, FM, FIM>(&mut self, f_mut: FM, f_immut: FIM) -> MaybeMut<'c, M, U>
    where 
        'b: 'c,
        T: 'b,
        U: 'c,
        FM: FnOnce(&'b mut T) -> &'c mut U,
        FIM: FnOnce(&'b T) -> &'c U 
    {
        unsafe {
            //TODO: Add safety comment
            MaybeMut::new(M::map(self.ptr, f_mut, f_immut))
        }
    }

    pub fn as_enum<'b>(&'b mut self) -> MaybeMutEnum<'b, T> {
        unsafe{
            //TODO: Add safety comment
            M::dispatch(
                self.ptr, 
                |r| MaybeMutEnum::Mutable(r), 
                |r| MaybeMutEnum::Immutable(r)
            )
        }
    }

    #[inline]
    pub fn as_immut<'b>(&'b self) -> &'b T {
        unsafe {
            //TODO: Add safety comment
            self.ptr.as_ref()
        }
    }
}

impl<'a, T> MaybeMut<'a, Mutable, T> {

    #[inline]
    pub fn as_mut<'b>(&'b mut self) -> &'b mut T {
        unsafe {
            //TODO: Add safety comment
            self.ptr.as_mut()
        }
    }
}

impl<'a, T> From<&'a mut T> for MaybeMut<'a, Mutable, T> {
    fn from(reference: &'a mut T) -> Self {
        unsafe{
            //TODO: Add safety comment
            Self::new(NonNull::from(reference))
        }
    }
}

impl<'a, T> From<&'a T> for MaybeMut<'a, Immutable, T> {
    fn from(reference: &'a T) -> Self {
        unsafe{
            //TODO: Add safety comment
            Self::new(NonNull::from(reference))
        }
    }
}
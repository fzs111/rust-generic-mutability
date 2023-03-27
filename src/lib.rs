#![no_std]

use core::ptr::NonNull;
use core::marker::PhantomData;

mod seal{
    pub trait MutabilitySealed {}
}

pub enum Immutable{}
pub enum Mutable{}

use seal::MutabilitySealed;
impl MutabilitySealed for Mutable{}
impl MutabilitySealed for Immutable{}

pub unsafe trait Mutability: MutabilitySealed{

    //TODO: Add safety note
    unsafe fn dispatch<'i, T, U, FM, FIM>(ptr: NonNull<T>, fn_mut: FM, fn_immut: FIM) -> U
    where 
        T: 'i,
        FM: FnOnce(&'i mut T) -> U,
        FIM: FnOnce(&'i T) -> U;
    
    //TODO: Add safety note
    unsafe fn map<'i, 'o, T, U, FM, FIM>(ptr: NonNull<T>, fn_mut: FM, fn_immut: FIM) -> NonNull<U>
    where 
        T: 'i,
        U: 'o,
        FM: FnOnce(&'i mut T) -> &'o mut U,
        FIM: FnOnce(&'i T) -> &'o U;

    fn is_mutable() -> bool;
}

unsafe impl Mutability for Mutable{
    
    #[inline]
    unsafe fn dispatch<'i, T, U, FM, FIM>(mut ptr: NonNull<T>, fn_mut: FM, _fn_immut: FIM) -> U
    where 
        T: 'i,
        FM: FnOnce(&'i mut T) -> U,
        FIM: FnOnce(&'i T) -> U,
    {
        fn_mut(ptr.as_mut())
    }
    
    #[inline]
    unsafe fn map<'i, 'o, T, U, FM, FIM>(mut ptr: NonNull<T>, fn_mut: FM, _fn_immut: FIM) -> NonNull<U>
    where 
        T: 'i,
        U: 'o,
        FM: FnOnce(&'i mut T) -> &'o mut U,
        FIM: FnOnce(&'i T) -> &'o U,
    {

        fn_mut(ptr.as_mut()).into()
    }

    #[inline]
    fn is_mutable() -> bool {
        true
    }
}

unsafe impl Mutability for Immutable{
    
    #[inline]
    unsafe fn dispatch<'i, T, U, FM, FIM>(ptr: NonNull<T>, _fn_mut: FM, fn_immut: FIM) -> U
    where 
        T: 'i,
        FM: FnOnce(&'i mut T) -> U,
        FIM: FnOnce(&'i T) -> U,
    {
        fn_immut(ptr.as_ref())
    }
    
    #[inline]
    unsafe fn map<'i, 'o, T, U, FM, FIM>(ptr: NonNull<T>, _fn_mut: FM, fn_immut: FIM) -> NonNull<U>
    where 
        T: 'i,
        U: 'o,
        FM: FnOnce(&'i mut T) -> &'o mut U,
        FIM: FnOnce(&'i T) -> &'o U,
    {

        fn_immut(ptr.as_ref()).into()
    }

    #[inline]
    fn is_mutable() -> bool {
        false
    }
}

pub enum MaybeMutEnum<'s, T>{
    Mutable(&'s mut T),
    Immutable(&'s T),
}

#[repr(transparent)]
pub struct MaybeMut<'s, M: Mutability, T>{
    _lifetime: PhantomData<&'s mut T>,
    _mutability: PhantomData<*const M>, 
    ptr: NonNull<T>,
}

impl<'s, M: Mutability, T> MaybeMut<'s, M, T> {
    
    #[inline]
    pub unsafe fn new(ptr: NonNull<T>) -> Self {
        //TODO: Add safety note
        Self { 
            _lifetime: PhantomData, 
            _mutability: PhantomData, 
            ptr,
        }
    }
    
    #[inline]
    pub fn dispatch<'i, U, FM, FIM>(&'i mut self, fn_mut: FM, fn_immut: FIM) -> U
    where 
        's: 'i,
        FM: FnOnce(&'i mut T) -> U,
        FIM: FnOnce(&'i T) -> U,
    {
        unsafe{
            //TODO: Add safety comment
            M::dispatch(self.ptr, fn_mut, fn_immut)
        }
    }

    #[inline]
    pub fn map<'i, 'o, U, FM, FIM>(&mut self, fn_mut: FM, fn_immut: FIM) -> MaybeMut<'o, M, U>
    where 
        's: 'i,
        U: 'o,
        FM: FnOnce(&'i mut T) -> &'o mut U,
        FIM: FnOnce(&'i T) -> &'o U,
    {
        unsafe {
            //TODO: Add safety comment
            MaybeMut::new(M::map(self.ptr, fn_mut, fn_immut))
        }
    }

    pub fn as_enum<'o>(&'o mut self) -> MaybeMutEnum<'o, T> 
    where
        's: 'o,
    {
        unsafe{
            //TODO: Add safety comment
            M::dispatch(
                self.ptr, 
                |r| MaybeMutEnum::Mutable(r), 
                |r| MaybeMutEnum::Immutable(r),
            )
        }
    }

    #[inline]
    pub fn as_immut<'o>(&'o self) -> &'o T 
    where
        's: 'o,
    {
        unsafe {
            //TODO: Add safety comment
            self.ptr.as_ref()
        }
    }
}

impl<'s, T> MaybeMut<'s, Mutable, T> {

    #[inline]
    pub fn as_mut<'o>(&'o mut self) -> &'o mut T 
    where
        's: 'o,
    {
        unsafe {
            //TODO: Add safety comment
            self.ptr.as_mut()
        }
    }
}

impl<'i, 'o, T> From<&'i mut T> for MaybeMut<'o, Mutable, T> 
where
    'i: 'o,
{
    fn from(reference: &'i mut T) -> Self {
        unsafe{
            //TODO: Add safety comment
            Self::new(NonNull::from(reference))
        }
    }
}

impl<'i, 'o, T> From<&'i T> for MaybeMut<'o, Immutable, T> 
where
    'i: 'o,
{
    fn from(reference: &'i T) -> Self {
        unsafe{
            //TODO: Add safety comment
            Self::new(NonNull::from(reference))
        }
    }
}
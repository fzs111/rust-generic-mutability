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

    //TODO: Add safety comment
    unsafe fn dispatch<'i, T, U, X, FM, FIM>(ptr: NonNull<T>, moved: X, fn_mut: FM, fn_immut: FIM) -> U
        where 
            T: 'i + ?Sized,
            FM: FnOnce(X, &'i mut T) -> U,
            FIM: FnOnce(X, &'i T) -> U;

    //TODO: Add safety comment
    unsafe fn map<'i, 'o, T, U, X, FM, FIM>(ptr: NonNull<T>, moved: X, fn_mut: FM, fn_immut: FIM) -> NonNull<U>
        where 
            T: 'i + ?Sized,
            U: 'o + ?Sized,
            FM: FnOnce(X, &'i mut T) -> &'o mut U,
            FIM: FnOnce(X, &'i T) -> &'o U;

    fn is_mutable() -> bool;
}

unsafe impl Mutability for Mutable{
    
    #[inline]
    unsafe fn dispatch<'i, T, U, X, FM, FIM>(mut ptr: NonNull<T>, moved: X, fn_mut: FM, _fn_immut: FIM) -> U
        where 
            T: 'i + ?Sized,
            FM: FnOnce(X, &'i mut T) -> U,
            FIM: FnOnce(X, &'i T) -> U 
    {
        fn_mut(moved, ptr.as_mut())
    }
    
    #[inline]
    unsafe fn map<'i, 'o, T, U, X, FM, FIM>(mut ptr: NonNull<T>, moved: X, fn_mut: FM, _fn_immut: FIM) -> NonNull<U>
        where 
            T: 'i + ?Sized,
            U: 'o + ?Sized,
            FM: FnOnce(X, &'i mut T) -> &'o mut U,
            FIM: FnOnce(X, &'i T) -> &'o U 
    {
        fn_mut(moved, ptr.as_mut()).into()
    }

    #[inline]
    fn is_mutable() -> bool {
        true
    }
}

unsafe impl Mutability for Immutable{
    
    #[inline]
    unsafe fn dispatch<'i, T, U, X, FM, FIM>(ptr: NonNull<T>, moved: X, _fn_mut: FM, fn_immut: FIM) -> U
        where 
            T: 'i + ?Sized,
            FM: FnOnce(X, &'i mut T) -> U,
            FIM: FnOnce(X, &'i T) -> U 
    {
        fn_immut(moved, ptr.as_ref())
    }
    
    #[inline]
    unsafe fn map<'i, 'o, T, U, X, FM, FIM>(ptr: NonNull<T>, moved: X, _fn_mut: FM, fn_immut: FIM) -> NonNull<U>
        where 
            T: 'i + ?Sized,
            U: 'o + ?Sized,
            FM: FnOnce(X, &'i mut T) -> &'o mut U,
            FIM: FnOnce(X, &'i T) -> &'o U 
    {
        fn_immut(moved, ptr.as_ref()).into()
    }

    #[inline]
    fn is_mutable() -> bool {
        false
    }
}


#[repr(transparent)]
pub struct GenRef<'s, M: Mutability, T: ?Sized>{
    _lifetime: PhantomData<&'s mut T>,
    _mutability: PhantomData<*const M>, 
    ptr: NonNull<T>,
}
pub enum GenRefEnum<'s, T: ?Sized>{
    Mutable(&'s mut T),
    Immutable(&'s T),
}

impl<'s, M: Mutability, T: ?Sized> GenRef<'s, M, T> {
    
    #[inline]
    pub unsafe fn new(ptr: NonNull<T>) -> Self {
        //TODO: Add safety comment
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
            M::dispatch(self.ptr, (), |(), t| fn_mut(t), |(), t| fn_immut(t))
        }
    }

    #[inline]
    pub fn dispatch_with_move<'i, U, X, FM, FIM>(&'i mut self, moved: X, fn_mut: FM, fn_immut: FIM) -> U
        where 
            's: 'i,
            FM: FnOnce(X, &'i mut T) -> U,
            FIM: FnOnce(X, &'i T) -> U,
    {
        unsafe{
            //TODO: Add safety comment
            M::dispatch(self.ptr, moved, fn_mut, fn_immut)
        }
    }

    #[inline]
    pub fn map<'i, 'o, U, FM, FIM>(&mut self, fn_mut: FM, fn_immut: FIM) -> GenRef<'o, M, U>
        where 
            's: 'i,
            U: 'o + ?Sized,
            FM: FnOnce(&'i mut T) -> &'o mut U,
            FIM: FnOnce(&'i T) -> &'o U,
    {
        unsafe {
            //TODO: Add safety comment
            GenRef::new(M::map(self.ptr, (), |(), t| fn_mut(t), |(), t| fn_immut(t)))
        }
    }

    #[inline]
    pub fn map_with_move<'i, 'o, U, X, FM, FIM>(&mut self, moved: X, fn_mut: FM, fn_immut: FIM) -> GenRef<'o, M, U>
        where 
            's: 'i,
            U: 'o + ?Sized,
            FM: FnOnce(X, &'i mut T) -> &'o mut U,
            FIM: FnOnce(X, &'i T) -> &'o U,
    {
        unsafe {
            //TODO: Add safety comment
            GenRef::new(M::map(self.ptr, moved, fn_mut, fn_immut))
        }
    }

    #[inline]
    pub fn as_ptr(&self) -> NonNull<T> {
        self.ptr
    }

    #[inline]
    pub fn as_enum<'o>(&'o mut self) -> GenRefEnum<'o, T> 
        where
            's: 'o,
    {
        unsafe{
            //TODO: Add safety comment
            M::dispatch(
                self.ptr, 
                (),
                |(), r| GenRefEnum::Mutable(r), 
                |(), r| GenRefEnum::Immutable(r),
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

    #[inline]
    pub fn reborrow<'o>(&'o mut self) -> GenRef<'o, M, T> {
        unsafe{
            //TODO: Add safety comment
            Self::new(self.ptr)
        }
    }
}

impl<'s, T: ?Sized> GenRef<'s, Mutable, T> {

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

impl<'i, 'o, T: ?Sized> From<&'i mut T> for GenRef<'o, Mutable, T> 
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

impl<'i, 'o, T: ?Sized> From<&'i T> for GenRef<'o, Immutable, T> 
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



/// ```compile_fail
/// let mut string = String::from("asd");
///
/// let mut genref = MaybeMut::from(&mut string);
///
/// let mut_ref = genref.as_mut();
///
/// assert_eq!(string, String::from("asdf"));
///
/// mut_ref.push('f');
/// ```
#[cfg(doctest)]
fn mut_create_extract(){}
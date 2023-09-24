#![no_std]

use core::borrow::{Borrow, BorrowMut};
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
            FM: FnOnce(&'i mut T, X) -> &'o mut U,
            FIM: FnOnce(&'i T, X) -> &'o U;

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
            FM:  FnOnce(&'i mut T, X) -> &'o mut U,
            FIM: FnOnce(&'i T,     X) -> &'o U 
    {
        fn_mut(ptr.as_mut(), moved).into()
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
            FM:  FnOnce(&'i mut T, X) -> &'o mut U,
            FIM: FnOnce(&'i T,     X) -> &'o U 
    {
        fn_immut(ptr.as_ref(), moved).into()
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
    pub fn map<'i, 'o, U, FM, FIM>(self, fn_mut: FM, fn_immut: FIM) -> GenRef<'o, M, U>
        where 
            's: 'i,
            U: 'o + ?Sized,
            FM: FnOnce(&'i mut T) -> &'o mut U,
            FIM: FnOnce(&'i T) -> &'o U,
    {
        unsafe {
            //TODO: Add safety comment
            GenRef::new(M::map(self.ptr, (), |t, ()| fn_mut(t), |t, ()| fn_immut(t)))
        }
    }

    #[inline]
    pub fn map_with_move<'i, 'o, U, X, FM, FIM>(&mut self, moved: X, fn_mut: FM, fn_immut: FIM) -> GenRef<'o, M, U>
        where 
            's: 'i,
            U: 'o + ?Sized,
            FM:  FnOnce(&'i mut T, X) -> &'o mut U,
            FIM: FnOnce(&'i T,     X) -> &'o U,
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
    #[inline]
    pub fn into_immut(self) -> &'s T {
        unsafe{
            //TODO: Add safety comment
            & *self.ptr.as_ptr()
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

    #[inline]
    pub fn into_mut(self) -> &'s mut T {
        unsafe{
            //TODO: Add safety comment
            &mut *self.ptr.as_ptr()
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

impl<M: Mutability, T> AsRef<T> for GenRef<'_, M, T> {
    fn as_ref(&self) -> &T {
        self.as_immut()
    }
}
impl<T> AsMut<T> for GenRef<'_, Mutable, T> {
    fn as_mut(&mut self) -> &mut T {
        self.as_mut()
    }
}

impl<M: Mutability, T> Borrow<T> for GenRef<'_, M, T> {
    fn borrow(&self) -> &T {
        self.as_immut()
    }
}
impl<T> BorrowMut<T> for GenRef<'_, Mutable, T> {
    fn borrow_mut(&mut self) -> &mut T {
        self.as_mut()
    }
}

#[macro_export]
macro_rules! gen_ref {
    ($gen_ref:ident -> &gen $place:expr) => {
        GenRef::map($gen_ref, |$gen_ref| &mut $place, |$gen_ref| & $place)
    };

    //? Are there any use cases for this branch?
    ($gen_ref:ident move ($($moved:ident),+) -> &gen $place:expr) => {
        GenRef::map_with_move($gen_ref, ($($moved),+), |$gen_ref, ($($moved),+)| &mut $place, |$x| & $place)
    };
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

/// ```compile_fail
/// let gen_v = {
///     let mut v = vec![1, 2, 3, 4];
/// 
///     let mut gen_v = GenRef::from(&mut v);
/// 
///     gen_v.reborrow()
/// };
/// gen_v.
/// ```
#[cfg(doctest)]
fn reborrow_with_longer_lifetime(){}
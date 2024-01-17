#![no_std]

use core::borrow::{Borrow, BorrowMut};
use core::fmt::{Debug, Display};
use core::hash::Hash;
use core::ptr::NonNull;
use core::marker::PhantomData;

mod seal{
    pub trait MutabilitySealed {}
}

pub enum Immutable{}
pub enum Mutable{}

impl seal::MutabilitySealed for Mutable{}
impl seal::MutabilitySealed for Immutable{}

pub unsafe trait Mutability: seal::MutabilitySealed{

    //TODO: Add safety comment
    unsafe fn dispatch<'a, T, U, X, FM, FIM>(ptr: NonNull<T>, moved: X, fn_mut: FM, fn_immut: FIM) -> U
        where 
            T: 'a + ?Sized,
            FM:  FnOnce(&'a mut T, X) -> U,
            FIM: FnOnce(&'a T,     X) -> U;

    //TODO: Add safety comment
    unsafe fn map<'a, T, U, X, FM, FIM>(ptr: NonNull<T>, moved: X, fn_mut: FM, fn_immut: FIM) -> NonNull<U>
        where 
            T: 'a + ?Sized,
            U: 'a + ?Sized,
            FM:  FnOnce(&'a mut T, X) -> &'a mut U,
            FIM: FnOnce(&'a T,     X) -> &'a U;

    //TODO: Add safety comment
    unsafe fn split<'a, T, U, V, X, FM, FIM>(ptr: NonNull<T>, moved: X, fn_mut: FM, fn_immut: FIM) -> (NonNull<U>, NonNull<V>)
        where 
            T: 'a + ?Sized,
            U: 'a + ?Sized,
            V: 'a + ?Sized,
            FM:  FnOnce(&'a mut T, X) -> (&'a mut U, &'a mut V),
            FIM: FnOnce(&'a T,     X) -> (&'a U, &'a V);

    fn is_mutable() -> bool;
}

unsafe impl Mutability for Mutable{
    
    #[inline]
    unsafe fn dispatch<'a, T, U, X, FM, FIM>(mut ptr: NonNull<T>, moved: X, fn_mut: FM, _fn_immut: FIM) -> U
        where 
            T: 'a + ?Sized,
            FM:  FnOnce(&'a mut T, X) -> U,
            FIM: FnOnce(&'a T,     X) -> U, 
    {
        fn_mut(ptr.as_mut(), moved)
    }
    
    #[inline]
    unsafe fn map<'a, T, U, X, FM, FIM>(mut ptr: NonNull<T>, moved: X, fn_mut: FM, _fn_immut: FIM) -> NonNull<U>
        where 
            T: 'a + ?Sized,
            U: 'a + ?Sized,
            FM:  FnOnce(&'a mut T, X) -> &'a mut U,
            FIM: FnOnce(&'a T,     X) -> &'a U,
    {
        fn_mut(ptr.as_mut(), moved).into()
    }

    #[inline]
    unsafe fn split<'a, T, U, V, X, FM, FIM>(mut ptr: NonNull<T>, moved: X, fn_mut: FM, _fn_immut: FIM) -> (NonNull<U>, NonNull<V>)
        where 
            T: 'a + ?Sized,
            U: 'a + ?Sized,
            V: 'a + ?Sized,
            FM:  FnOnce(&'a mut T, X) -> (&'a mut U, &'a mut V),
            FIM: FnOnce(&'a T,     X) -> (&'a U, &'a V) 
    {
        let (a, b) = fn_mut(ptr.as_mut(), moved);
        (a.into(), b.into())
    }

    #[inline]
    fn is_mutable() -> bool {
        true
    }
}

unsafe impl Mutability for Immutable{
    
    #[inline]
    unsafe fn dispatch<'a, T, U, X, FM, FIM>(ptr: NonNull<T>, moved: X, _fn_mut: FM, fn_immut: FIM) -> U
        where 
            T: 'a + ?Sized,
            FM:  FnOnce(&'a mut T, X) -> U,
            FIM: FnOnce(&'a T,     X) -> U,
    {
        fn_immut(ptr.as_ref(), moved)
    }
    
    #[inline]
    unsafe fn map<'a, T, U, X, FM, FIM>(ptr: NonNull<T>, moved: X, _fn_mut: FM, fn_immut: FIM) -> NonNull<U>
        where 
            T: 'a + ?Sized,
            U: 'a + ?Sized,
            FM:  FnOnce(&'a mut T, X) -> &'a mut U,
            FIM: FnOnce(&'a T,     X) -> &'a U,
    {
        fn_immut(ptr.as_ref(), moved).into()
    }

    
    unsafe fn split<'a, T, U, V, X, FM, FIM>(ptr: NonNull<T>, moved: X, _fn_mut: FM, fn_immut: FIM) -> (NonNull<U>, NonNull<V>)
        where 
            T: 'a + ?Sized,
            U: 'a + ?Sized,
            V: 'a + ?Sized,
            FM:  FnOnce(&'a mut T, X) -> (&'a mut U, &'a mut V),
            FIM: FnOnce(&'a T,     X) -> (&'a U, &'a V)
    {
        let (a, b) = fn_immut(ptr.as_ref(), moved);
        (a.into(), b.into())
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
    pub fn dispatch<U, FM, FIM>(self, fn_mut: FM, fn_immut: FIM) -> U
        where 
            FM:  FnOnce(&'s mut T) -> U,
            FIM: FnOnce(&'s T) -> U,
    {
        unsafe{
            //TODO: Add safety comment
            M::dispatch(self.ptr, (), |t, ()| fn_mut(t), |t, ()| fn_immut(t))
        }
    }

    #[inline]
    pub fn dispatch_with_move<U, X, FM, FIM>(self, moved: X, fn_mut: FM, fn_immut: FIM) -> U
        where 
            FM:  FnOnce(&'s mut T, X) -> U,
            FIM: FnOnce(&'s T,     X) -> U,
    {
        unsafe{
            //TODO: Add safety comment
            M::dispatch(self.ptr, moved, fn_mut, fn_immut)
        }
    }

    #[inline]
    pub fn map<'a, U, FM, FIM>(self, fn_mut: FM, fn_immut: FIM) -> GenRef<'a, M, U>
        where 
            's: 'a,
            U: 'a + ?Sized,
            FM:  FnOnce(&'a mut T) -> &'a mut U,
            FIM: FnOnce(&'a T) -> &'a U,
    {
        unsafe {
            //TODO: Add safety comment
            GenRef::new(M::map(self.ptr, (), |t, ()| fn_mut(t), |t, ()| fn_immut(t)))
        }
    }

    #[inline]
    pub fn map_with_move<U, X, FM, FIM>(self, moved: X, fn_mut: FM, fn_immut: FIM) -> GenRef<'s, M, U>
        where 
            U: ?Sized,
            FM:  FnOnce(&'s mut T, X) -> &'s mut U,
            FIM: FnOnce(&'s T,     X) -> &'s U,
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
    pub fn as_immut(&self) -> &T {
        unsafe {
            //TODO: Add safety comment
            self.ptr.as_ref()
        }
    }

    #[inline]
    pub fn reborrow(&mut self) -> GenRef<'_, M, T> {
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

    #[inline]
    pub fn call<U, F>(self, f: F) -> U
        where F: FnOnce(Self) -> U
    {
        f(self)
    }

    #[inline]
    pub fn split<U, V, FM, FIM>(self, fn_mut: FM, fn_immut: FIM) -> (GenRef<'s, M, U>, GenRef<'s, M, V>)
        where 
            U: ?Sized,
            V: ?Sized,
            FM:  FnOnce(&mut T) -> (&mut U, &mut V),
            FIM: FnOnce(&T) -> (&U, &V)
    {
        unsafe {
            //TODO: Add safety comment
            let (a, b) = M::split(self.ptr, (), |t, ()| fn_mut(t), |t, ()| fn_immut(t));
            (GenRef::new(a), GenRef::new(b))
        }
    }
    
    #[inline]
    pub fn split_with_move<U, V, X, FM, FIM>(self, moved: X, fn_mut: FM, fn_immut: FIM) -> (GenRef<'s, M, U>, GenRef<'s, M, V>)
        where 
            U: ?Sized,
            V: ?Sized,
            FM:  FnOnce(&mut T, X) -> (&mut U, &mut V),
            FIM: FnOnce(&T,     X) -> (&U, &V)
    {
        unsafe {
            //TODO: Add safety comment
            let (a, b) = M::split(self.ptr, moved, fn_mut, fn_immut);
            (GenRef::new(a), GenRef::new(b))
        }
    }
}

impl<'s, T: ?Sized> GenRef<'s, Mutable, T> {

    #[inline]
    pub fn as_mut(&mut self) -> &mut T {
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

impl<'a, T: ?Sized> From<&'a mut T> for GenRef<'a, Mutable, T> {
    fn from(reference: &'a mut T) -> Self {
        unsafe{
            //TODO: Add safety comment
            Self::new(NonNull::from(reference))
        }
    }
}

impl<'a, T: ?Sized> From<&'a T> for GenRef<'a, Immutable, T> {
    fn from(reference: &'a T) -> Self {
        unsafe{
            //TODO: Add safety comment
            Self::new(NonNull::from(reference))
        }
    }
}

impl<MT: Mutability, MU: Mutability, T: ?Sized, U: ?Sized> PartialEq<GenRef<'_, MU, U>> for GenRef<'_, MT, T>
    where T: PartialEq<U>
{
    fn eq(&self, other: &GenRef<'_, MU, U>) -> bool {
        self.as_immut().eq(other.as_immut())
    }
}

impl<M: Mutability, T: Eq + ?Sized> Eq for GenRef<'_, M, T> {}

impl<MT: Mutability, MU: Mutability, T: ?Sized, U: ?Sized> PartialOrd<GenRef<'_, MU, U>> for GenRef<'_, MT, T>
    where T: PartialOrd<U>
{
    fn partial_cmp(&self, other: &GenRef<'_, MU, U>) -> Option<core::cmp::Ordering> {
        self.as_immut().partial_cmp(other.as_immut())
    }
}

impl<M: Mutability, T: Ord + ?Sized> Ord for GenRef<'_, M, T>
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_immut().cmp(other.as_immut())
    }
}

impl<M: Mutability, T: Hash + ?Sized> Hash for GenRef<'_, M, T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_immut().hash(state)
    }
}

//TODO: Add safety comment
unsafe impl<M: Mutability, T: Sync + ?Sized> Sync for GenRef<'_, M, T> {}

//TODO: Add safety comment
unsafe impl<M: Mutability, T: Send + Sync + ?Sized> Send for GenRef<'_, M, T> {}

impl<M: Mutability, T: ?Sized, U: ?Sized> AsRef<U> for GenRef<'_, M, T> 
    where T: AsRef<U>
{
    fn as_ref(&self) -> &U {
        self.as_immut().as_ref()
    }
}

impl<T: AsMut<U> + ?Sized, U: ?Sized> AsMut<U> for GenRef<'_, Mutable, T> {
    fn as_mut(&mut self) -> &mut U {
        self.as_mut().as_mut()
    }
}

impl<M: Mutability, T: ?Sized> Borrow<T> for GenRef<'_, M, T> {
    fn borrow(&self) -> &T {
        self.as_immut()
    }
}
impl<T: ?Sized> BorrowMut<T> for GenRef<'_, Mutable, T> {
    fn borrow_mut(&mut self) -> &mut T {
        self.as_mut()
    }
}

impl<'a, M: Mutability, T: ?Sized> From<GenRef<'a, M, T>> for GenRefEnum<'a, T> {
    fn from(genref: GenRef<'a, M, T>) -> Self {
        genref.dispatch(
            |r| GenRefEnum::Mutable(r),
            |r| GenRefEnum::Immutable(r),
        )
    }
}

impl<M: Mutability, T: Debug + ?Sized> Debug for GenRef<'_, M, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.as_immut().fmt(f)
    }
}

impl<M: Mutability, T: Display + ?Sized> Display for GenRef<'_, M, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.as_immut().fmt(f)
    }
}

pub struct IncorrectMutability{
    target_mutable: bool,
}

// This shouldn't require std when this is stabilised: https://github.com/rust-lang/rust/issues/103765
#[cfg(feature = "std")]
impl std::error::Error for IncorrectMutability{}

impl Debug for IncorrectMutability{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self.target_mutable {
            true => "IncorrectMutability(immut -> mut)",
            false => "IncorrectMutability(mut -> immut)"
        })
    }
}

impl Display for IncorrectMutability{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("\
Failed to convert GenRefEnum<'_, T> into GenRef<'_, {target}, T>:
Mutability of target differs from source

Expected enum variant GenRefEnum::{target}
   Found enum variant GenRefEnum::{source}",
            target = match self.target_mutable {
                true => "Mutable",
                false => "Immutable"
            },
            source = match !self.target_mutable {
                true => "Mutable",
                false => "Immutable"
            }
        ))
    }
}

impl<'a, M: Mutability, T: ?Sized> TryFrom<GenRefEnum<'a, T>> for GenRef<'a, M, T> {
    type Error = IncorrectMutability;
    fn try_from(genref_enum: GenRefEnum<'a, T>) -> Result<Self, Self::Error> {
        match (M::is_mutable(), genref_enum) {
            (true, GenRefEnum::Mutable(r)) => {
                unsafe{
                    //TODO: Add safety comment
                    Ok(GenRef::new(r.into()))
                }
            },
            (false, GenRefEnum::Immutable(r)) => {
                unsafe{
                    //TODO: Add safety comment
                    Ok(GenRef::new(r.into()))
                }
            },
            (target_mutable, _) => {
                Err(IncorrectMutability{ target_mutable })
            }
        }
    }
}

#[macro_export]
macro_rules! gen_ref {
    ($gen_ref:ident -> (&gen $place_a:expr, &gen $place_b:expr)) => {
        GenRef::split($gen_ref, |$gen_ref| (&mut $place_a, &mut $place_b), |$gen_ref| (& $place_a, & $place_b))
    };

    ($gen_ref:ident move ($($moved:ident),+)  -> (&gen $place_a:expr, &gen $place_b:expr)) => {
        GenRef::split_with_move($gen_ref, ($($moved),+), |$gen_ref, ($($moved),+)| (&mut $place_a, &mut $place_b), |$gen_ref, ($($moved),+)| (& $place_a, & $place_b))
    };

    ($gen_ref:ident -> &gen $place:expr) => {
        GenRef::map($gen_ref, |$gen_ref| &mut $place, |$gen_ref| & $place)
    };

    //? Are there any use cases for this branch?
    ($gen_ref:ident move ($($moved:ident),+) -> &gen $place:expr) => {
        GenRef::map_with_move($gen_ref, ($($moved),+), |$gen_ref, ($($moved),+)| &mut $place, |$gen_ref, ($($moved),+)| & $place)
    };
}

/// ```compile_fail
/// # use generic_mutability::GenRef;
/// let mut string = String::from("asd");
///
/// let mut genref = GenRef::from(&mut string);
///
/// let mut_ref = genref.as_mut();
/// 
/// mut_ref.push('f');
///
/// assert_eq!(string, String::from("asdf"));
///
/// mut_ref.push('g');
/// ```
#[cfg(doctest)]
fn mut_create_extract(){}

/// ```compile_fail
/// # use generic_mutability::GenRef;
/// let mut gen_v = {
///     let mut v = vec![1, 2, 3, 4];
/// 
///     let mut gen_v = GenRef::from(&mut v);
/// 
///     gen_v.reborrow()
/// };
/// gen_v.as_mut()[0] += 10;
/// ```
#[cfg(doctest)]
fn reborrow_with_longer_lifetime(){}

/// ```compile_fail
/// let s = String::from("asdf");
/// 
/// let gen_s = GenRef::from(&s);
/// 
/// let ref_s = gen_s.into_immut();
/// 
/// drop(s);
/// 
/// println!("{}", ref_s);
/// ```
#[cfg(doctest)]
fn convert_into_longer_lifetime() {}
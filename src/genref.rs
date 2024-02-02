use core::hint::unreachable_unchecked;
use core::marker::PhantomData;
use core::ptr::NonNull;

use core::ops::{ Deref, DerefMut };
use core::hash::Hash;
use core::borrow::{ Borrow, BorrowMut };
use core::fmt::{ Debug, Display };

use crate::{ Immutable, Mutability, Mutable };

/// `GenRef` is the main type of this crate. It is a safe type; it represents a reference with generic mutability.
/// 
/// Library code can take a `GenRef` as an argument, and use the `map`, `split`, `dispatch` and `as_immut` methods to process the reference and return another one of the same mutability.
/// It can also use the `gen_ref!` macro for more convenient `map`ping or `split`ting. 
/// It is also convert to/from a dynamic representation (`GenRefEnum`), which offers more flexibility for a little runtime overhead.
/// 
/// User code can create a `GenRef` from both `&T` and `&mut T` using the `From` implementation, pass it to library code, then unwrap the result with the `into_[im]mut` or `as_[im]mut` methods.

// INVARIANT: `ptr` must be valid for reads and also for writes if `M` is `Mutable`, for lifetime `'s`

// `#[repr(transparent)]` is used to enable niche optimization, but is *not* a layout guarantee! (You must not transmute between a `GenRef` and `NonNull`.)
#[repr(transparent)]
pub struct GenRef<'s, M: Mutability, T: ?Sized>{
    _lifetime: PhantomData<&'s mut T>,
    _mutability: PhantomData<*const M>, 
    ptr: NonNull<T>,
}

impl<'s, M: Mutability, T: ?Sized> GenRef<'s, M, T> {
    
    /// Creates a `GenRef` from a pointer with the chosen mutability and lifetime, without checking.
    ///
    /// To create a `GenRef` from a reference, use the `From` implementation.
    /// 
    /// To safely attempt to convert a known-mutability reference to a generic `GenRef`, use the `TryFrom<GenRefEnum<'_, T>>` implementation.
    ///
    /// ## Safety
    ///
    /// `GenRef` is a safe reference type. Using this method is equivalent to dereferencing the pointer and creating a reference from it. As such:
    ///
    /// - The pointer must be properly aligned.
    /// - The pointer must point to an initialized instance of `T`.
    /// - The lifetime `'a` is arbitrarily chosen and doesn't reflect the actual lifetime of the data. Extra care must be taken to ensure that the correct lifetime is used.
    /// - Furthermore:
    ///     - If the mutability is `Immutable`:
    ///         - The pointer must be valid for reads for lifetime `'a`.
    ///         - The pointed-to value must not be written to by other pointers and no mutable references to it may exist during `'a`.
    ///     - If the mutability is `Mutable`:
    ///         - The pointer must be valid for reads and writes for lifetime `'a`.
    ///         - The pointed-to value must not be accessed (read or written) by other pointers, and no references to it may exist during `'a`.
    #[inline]
    pub unsafe fn new(ptr: NonNull<T>) -> Self {
        Self { 
            _lifetime: PhantomData, 
            _mutability: PhantomData, 
            ptr,
        }
    }
    
    /// Calls either `fn_mut` or `fn_immut` depending on the mutability.
    /// Returns the value returned by the called closure.
    ///
    /// Capturing the same values with both closures will not work: if you need to do that, use the `dispatch_with_move` method instead.
    #[inline]
    pub fn dispatch<U, FM, FIM>(self, fn_mut: FM, fn_immut: FIM) -> U
        where 
            FM:  FnOnce(&'s mut T) -> U,
            FIM: FnOnce(&'s T) -> U,
    {
        unsafe{
            // SAFETY: the struct invariants ensure safety
            M::dispatch(
                self.ptr, 
                (), 
                |t, ()| fn_mut(t), 
                |t, ()| fn_immut(t)
            )
        }
    }

    /// Calls either `fn_mut` or `fn_immut` depending on the mutability, moving an arbitrary value `moved` into it. 
    /// Returns the value returned by the called closure.
    /// 
    /// This method is a helper for the case where both closures try to capture (move) the same value.
    /// You can move these values into the closure via the `moved` argument. If you need to move more than one values, use a tuple as the `moved` argument; if you do not need to move any values, you can use the `dispatch` method instead.
    #[inline]
    pub fn dispatch_with_move<U, X, FM, FIM>(self, moved: X, fn_mut: FM, fn_immut: FIM) -> U
        where 
            FM:  FnOnce(&'s mut T, X) -> U,
            FIM: FnOnce(&'s T,     X) -> U,
    {
        unsafe{
            // SAFETY: the struct invariants ensure safety
            M::dispatch(self.ptr, moved, fn_mut, fn_immut)
        }
    }
    
    /// Calls either `fn_mut` or `fn_immut` depending on the mutability.
    /// Returns the reference returned by the closure as a `GenRef`.
    /// 
    /// Capturing the same values with both closures will not work: if you need to do that, use the `map_with_move` method instead.
    /// If you want to call a function with the value of `self` (without unwrapping it), use the `call` method.
    #[inline]
    pub fn map<U, FM, FIM>(self, fn_mut: FM, fn_immut: FIM) -> GenRef<'s, M, U>
        where 
            U: ?Sized,
            FM:  FnOnce(&'s mut T) -> &'s mut U,
            FIM: FnOnce(&'s T) -> &'s U,
    {
        self.map_with_move(
            (),
            |t, ()| fn_mut(t),
            |t, ()| fn_immut(t)
        )
    }

    /// Calls either `fn_mut` or `fn_immut` depending on the mutability, moving an arbitrary value `moved` into it.
    /// Returns the reference returned by the closure as a `GenRef`.
    ///
    /// This method is a helper for the case where both closures try to capture (move) the same value.
    /// You can move these values into the closure via the `moved` argument. If you need to move more than one values, use a tuple as the `moved` argument; if you do not need to move any values, you can use the `map` method instead.
    #[inline]
    pub fn map_with_move<U, X, FM, FIM>(self, moved: X, fn_mut: FM, fn_immut: FIM) -> GenRef<'s, M, U>
        where 
            U: ?Sized,
            FM:  FnOnce(&'s mut T, X) -> &'s mut U,
            FIM: FnOnce(&'s T,     X) -> &'s U,
    {
        unsafe {
            // SAFETY: the struct invariants ensure safety for `map`
            // SAFETY: `map` guarantees the returned pointer is safe for `GenRef::new`
            // SAFETY: all lifetimes are constrained to `'s` by the function signature
            GenRef::new(M::map(self.ptr, moved, fn_mut, fn_immut))
        }
    }

    /// Gets the underlying pointer.
    /// 
    /// It is valid for reads and also for writes if the mutability is `Mutable`, for lifetime `'s`.
    /// 
    /// As with normal references, you are not allowed to use the `GenRef` while the pointer is in use.
    #[inline]
    pub fn as_ptr(&self) -> NonNull<T> {
        self.ptr
    }

    /// Gets a shared reference.
    /// 
    /// This method can be used in a generic context to gain immutable access to the referenced value.
    /// 
    /// It is identical to `Borrow<T>::borrow`.
    #[inline]
    pub fn as_immut(&self) -> &T {
        unsafe {
            // SAFETY: the struct invariants ensure safety
            // SAFETY: the returned lifetime is constrained to an elided lifetime by the function signature
            self.ptr.as_ref()
        }
    }

    /// Reborrows the `GenRef`.
    ///
    /// This method can be used to create a shorter-lived copy of the `GenRef`, while retaining ownership of the original.
    /// Normal references are reborrowed automatically when they are passed to a function, but you have to manually reborrow `GenRef`s.
    ///
    /// This method takes a unique borrow to the original `GenRef`, regardless of mutability.
    /// This means that you have to mark the binding mutable, even when no mutation takes place.
    #[inline]
    pub fn reborrow(&mut self) -> GenRef<'_, M, T> {
        unsafe{
            // SAFETY: the unique borrow on `self` prevents any use that would alias the created reference
            // SAFETY: all other invariants are inherited from the original reference
            Self::new(self.ptr)
        }
    }

    /// Calls the provided function, passing the `GenRef` as a parameter.
    ///
    /// This is a helper function to allow chaining function calls as if they were methods.
    #[inline]
    pub fn call<U, F>(self, f: F) -> U
        where F: FnOnce(Self) -> U
    {
        f(self)
    }

    /// Maps the reference into two derived references using `fn_mut` or `fn_immut` depending on the mutability.
    /// Returns the two references as `GenRef`s.
    ///
    /// Capturing the same values with both closures will not work: if you need to do that, use the `split_with_move` method instead.
    ///
    /// Note: it is not yet decided whether this will be generalized to support n-way splitting.
    /// Two implementations already exist, see the `split-tuples-macros` and `split-cons` branches.
    /// If you have a use case that requires more than 2-way splitting, please tell me about it in an issue.
    #[inline]
    pub fn split<U, V, FM, FIM>(self, fn_mut: FM, fn_immut: FIM) -> (GenRef<'s, M, U>, GenRef<'s, M, V>)
        where 
            U: ?Sized,
            V: ?Sized,
            FM:  FnOnce(&'s mut T) -> (&'s mut U, &'s mut V),
            FIM: FnOnce(&'s T) -> (&'s U, &'s V)
    {
        self.split_with_move(
            (),
            |t, ()| fn_mut(t),
            |t, ()| fn_immut(t)
        )
    }

    /// Maps the reference into two derived references using `fn_mut` or `fn_immut` depending on the mutability, moving an arbitrary value `moved` into it.
    /// Returns the two references as `GenRef`s.
    ///
    /// This method is a helper for the case where both closures try to capture (move) the same value.
    /// You can move these values into the closure via the `moved` argument. If you need to move more than one values, use a tuple as the `moved` argument; if you do not need to move any values, you can use the `split` method instead.
    ///
    /// Note: it is not yet decided whether this will be generalized to support n-way splitting.
    /// Two implementations already exist, see the `split-tuples-macros` and `split-cons` branches.
    /// If you have a use case that requires more than 2-way splitting, please tell me about it in an issue.
    #[inline]
    pub fn split_with_move<U, V, X, FM, FIM>(self, moved: X, fn_mut: FM, fn_immut: FIM) -> (GenRef<'s, M, U>, GenRef<'s, M, V>)
        where 
            U: ?Sized,
            V: ?Sized,
            FM:  FnOnce(&'s mut T, X) -> (&'s mut U, &'s mut V),
            FIM: FnOnce(&'s T,     X) -> (&'s U, &'s V)
    {
        unsafe {
            // SAFETY: the struct invariants ensure safety for `split`
            // SAFETY: `split` guarantees both returned pointers are safe for `GenRef::new`
            // SAFETY: all lifetimes are constrained to `'s` by the function signature
            let (a, b) = M::split(self.ptr, moved, fn_mut, fn_immut);
            (GenRef::new(a), GenRef::new(b))
        }
    }
}

impl<'s, T: ?Sized> GenRef<'s, Immutable, T> {

    /// Converts the `GenRef` into an immutable reference for the entire lifetime of the `GenRef`.
    ///
    /// This is used for unwrapping a `GenRef<'_, Immutable, T>` from the caller code, after the transformations are done.
    /// It is not accessible from generic code.
    /// 
    /// Note: implementing this method for a generic context would be sound, but using it that way would discard any mutable access the reference had. 
    /// The same behvior can be achieved using `dispatch`.
    #[inline]
    pub fn into_immut(self) -> &'s T {
        self.dispatch(
            |r| {
                //This branch is not reachable, but the implementation is correct.
                &*r
            },
            |r| r
        )
    }
}

impl<'s, T: ?Sized> GenRef<'s, Mutable, T> {

    /// Gets a mutable reference.
    ///
    /// This method is only implemented for `GenRef<'_, Mutable, T>`, so it is not accessible from generic code.
    #[inline]
    pub fn as_mut(&mut self) -> &mut T {
        self.reborrow().dispatch(
            |r| r,
            |_| unsafe {
                // SAFETY: only implemented where `M` is `Mutable`
                unreachable_unchecked()
            }
        )
    }

    /// Converts the `GenRef` into a mutable reference for the entire lifetime of the `GenRef`.
    ///
    /// This is used for unwrapping a `GenRef<'_ Mutable, T>` from the caller code, after the transformations are done.
    /// It is not accessible from generic code.
    #[inline]
    pub fn into_mut(self) -> &'s mut T {
        self.dispatch(
            |r| r,
            |_| unsafe {
                // SAFETY: only implemented where `M` is `Mutable`
                unreachable_unchecked()
            }
        )
    }
}

impl<'s, M: Mutability, T: DerefMut + ?Sized> GenRef<'s, M, T> {

    /// Dereferences the value inside the `GenRef` with `Deref` or `DerefMut`.
    pub fn as_deref(self) -> GenRef<'s, M, T::Target> {
        self.map(DerefMut::deref_mut, Deref::deref)
    }
}

impl<'a, T: ?Sized> From<&'a mut T> for GenRef<'a, Mutable, T> {

    /// Creates a `GenRef<'_, Mutable, T>` from a mutable reference.
    ///
    /// This is the primary way to create `GenRef`s in caller code.
    ///
    /// To create a generic `GenRef` from a reference, you either have to use `TryFrom<GenRefEnum<'_, T>>` or the unchecked `GenRef::new()`.
    fn from(reference: &'a mut T) -> Self {
        unsafe{
            // SAFETY: the pointer is obtained from a unique reference,
            //         so it satisfies all invariants of `GenRef<'a, Mutable, T>`
            Self::new(NonNull::from(reference))
        }
    }
}

impl<'a, T: ?Sized> From<&'a T> for GenRef<'a, Immutable, T> {

    /// Creates a `GenRef<'_, Immutable, T>` from a shared reference.
    ///
    /// This is the primary way to create `GenRef`s in caller code.
    ///
    /// To create a generic `GenRef` from a reference, you either have to use `TryFrom<GenRefEnum<'_, T>>` or the unchecked `GenRef::new()`.
    fn from(reference: &'a T) -> Self {
        unsafe{
            // SAFETY: the pointer is obtained from a shared reference,
            //         so it satisfies all invariants of `GenRef<'a, Immutable, T>`
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

impl<MT: Mutability, T: ?Sized, U: ?Sized> PartialEq<&U> for GenRef<'_, MT, T>
    where T: PartialEq<U>
{
    fn eq(&self, other: &&U) -> bool {
        self.as_immut().eq(other)
    }
}

impl<MT: Mutability, T: ?Sized, U: ?Sized> PartialEq<&mut U> for GenRef<'_, MT, T>
    where T: PartialEq<U>
{
    fn eq(&self, other: &&mut U) -> bool {
        self.as_immut().eq(other)
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

impl<MT: Mutability, T: ?Sized, U: ?Sized> PartialOrd<&U> for GenRef<'_, MT, T>
    where T: PartialOrd<U>
{
    fn partial_cmp(&self, other: &&U) -> Option<core::cmp::Ordering> {
        self.as_immut().partial_cmp(other)
    }
}

impl<MT: Mutability, T: ?Sized, U: ?Sized> PartialOrd<&mut U> for GenRef<'_, MT, T>
    where T: PartialOrd<U>
{
    fn partial_cmp(&self, other: &&mut U) -> Option<core::cmp::Ordering> {
        self.as_immut().partial_cmp(other)
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

// SAFETY: all references can soundly implement `Sync` if `T` does
unsafe impl<M: Mutability, T: Sync + ?Sized> Sync for GenRef<'_, M, T> {}

// SAFETY: all references can implement `Send` if `T: Send + Sync`
// `Send` could also be implemented for `GenRef<'_, Mutable, T>` for `T: Send + ?Sync`, 
// but that would collide with the impl for generic `M`
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

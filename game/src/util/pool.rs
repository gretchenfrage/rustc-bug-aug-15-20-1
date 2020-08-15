//! Generic pool for resource re-use. 

use std::{
    cell::{RefCell, RefMut},
    cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd},
    fmt::{self, Debug, Formatter},
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
};

/// Logic for producing and recycling in a pool.
pub trait PoolLogic<T> {
    /// Produce a new element for the pool.
    fn create(&self) -> T;

    /// Restore a used element to its fresh state.
    fn recycle(&self, elem: &mut T);
}

/// Generic pool for resource re-use. 
pub struct Pool<T, L: PoolLogic<T>> {
    logic: L,
    cells: Vec<RefCell<Option<T>>>,
}

impl<T, L: PoolLogic<T>> Pool<T, L> {
    /// Create a pool which can hold up to `size` pooled
    /// object before it just starts creating new objects.
    pub fn new(logic: L, size: usize) -> Self {
        let cells = (0..size).map(|_| RefCell::new(None)).collect();
        Pool { logic, cells }
    }

    /// Borrow a pooled element.
    pub fn get(&self) -> PoolGuard<'_, T> {
        if let Some(mut r) = self
            .cells
            .iter()
            .map(RefCell::try_borrow_mut)
            .filter_map(Result::ok)
            .filter(|r| r.is_some())
            .next()
        {
            self.logic.recycle(r.as_mut().unwrap());
            PoolGuard {
                inner: GuardInner::Borrowed(r),
            }
        } else if let Some(mut r) = self
            .cells
            .iter()
            .map(RefCell::try_borrow_mut)
            .filter_map(Result::ok)
            .next()
        {
            *r = Some(self.logic.create());
            PoolGuard {
                inner: GuardInner::Borrowed(r),
            }
        } else {
            let e = self.logic.create();
            PoolGuard {
                inner: GuardInner::Owned(e),
            }
        }
    }
}

/// Element borrowed from a `Pool`.
pub struct PoolGuard<'a, T> {
    inner: GuardInner<'a, T>,
}

enum GuardInner<'a, T> {
    Borrowed(RefMut<'a, Option<T>>),
    Owned(T),
}

impl<'a, T> PoolGuard<'a, T> {
    /// Extract the guarded element from the pool.
    pub fn take(self) -> T {
        match self.inner {
            GuardInner::Borrowed(mut r) => r.take().unwrap(),
            GuardInner::Owned(t) => t,
        }
    }
}

impl<'a, T> Deref for PoolGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        match &self.inner {
            &GuardInner::Borrowed(ref r) => r.as_ref().unwrap(),
            &GuardInner::Owned(ref t) => t,
        }
    }
}

impl<'a, T> DerefMut for PoolGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        match &mut self.inner {
            &mut GuardInner::Borrowed(ref mut r) => r.as_mut().unwrap(),
            &mut GuardInner::Owned(ref mut t) => t,
        }
    }
}

impl<T, L> Clone for Pool<T, L>
where
    L: PoolLogic<T> + Clone,
{
    fn clone(&self) -> Self { Pool::new(self.logic.clone(), self.cells.len()) }
}

impl<T, L> Debug for Pool<T, L>
where
    L: PoolLogic<T> + Debug,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Debug::fmt(&self.logic, f)
    }
}

impl<T, L> Eq for Pool<T, L> where L: PoolLogic<T> + Eq {}

impl<T, L> PartialEq for Pool<T, L>
where
    L: PoolLogic<T> + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self.logic, &other.logic)
    }
}

impl<T, L> Ord for Pool<T, L>
where
    L: PoolLogic<T> + Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&self.logic, &other.logic)
    }
}

impl<T, L> PartialOrd for Pool<T, L>
where
    L: PoolLogic<T> + PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        PartialOrd::partial_cmp(&self.logic, &other.logic)
    }
}

impl<T, L> Hash for Pool<T, L>
where
    L: PoolLogic<T> + Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) { Hash::hash(&self.logic, state) }
}

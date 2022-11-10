use crate::cell::Cell;
use std::marker::PhantomData;
use std::ptr::NonNull;
// hold the value because keep count in Rc struct each clone of the Rc
// would have it's own reference count
struct RcInner<T> {
    value: T,
    refcount: Cell<usize>,
}
pub struct Rc<T> {
    // why is unsafe complier doesn't know  whether this pointer is still valid
    inner: NonNull<RcInner<T>>,
    // this mine we own a T in the here.
    // https://doc.rust-lang.org/nomicon/dropck.html
    _marker: PhantomData<RcInner<T>>,
}

impl<T> Rc<T> {
    pub fn new(value: T) -> Self {
        let inner: Box<RcInner<T>> = Box::new(RcInner {
            value,
            refcount: Cell::new(1),
        });
        Rc {
            //raw 原始
            //inner: Box::into_raw(inner),//consumes the Box, returning the wrapperd raw pointer
            // SAFETY: Box does not give us to a null pointer.
            inner: unsafe { NonNull::new_unchecked(Box::into_raw(inner)) },
            _marker: PhantomData,
        }

        // why is Rc { inner: &* inner }?
        // when this scope ends then the box gets dropped and so the memory get freed
        // we've needed to not drop the box even though we don't have a box anymore
    }
}

impl<T> Clone for Rc<T> {
    fn clone(&self) -> Self {
        let inner = unsafe { self.inner.as_ref() };
        let c = inner.refcount.get();
        inner.refcount.set(c + 1);
        Rc {
            inner: self.inner,
            _marker: PhantomData,
        }
    }
}

impl<T> std::ops::Deref for Rc<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: self.inner is a  Box that is only deallocated when the last Rc gose away
        // we have an Rc, therefore the Box has not been deallocated, so deref is fine.
        &unsafe { self.inner.as_ref() }.value
    }
}

impl<T> Drop for Rc<T> {
    fn drop(&mut self) {
        let inner = unsafe { self.inner.as_ref() };
        let c = inner.refcount.get();
        if c == 1 {
            // SAFETY: we are the _only_ Rc left, and we are being dropped.
            // therefore,  after us, there will be no Rc's and no references to T,
            drop(inner);
            let _ = unsafe { Box::from_raw(self.inner.as_ptr()) };
        } else {
            // there are other Rcs, so don't drop the Box!;
            inner.refcount.set(c - 1);
        }
    }
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn bad() {
        // let (y, x);
        // x = String::from("foo");
        // y = Rc::new(&x); error
        let five = Rc::new(5);
    }
}

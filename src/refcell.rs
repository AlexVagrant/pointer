//A mutable memory location with dynamically checked borrow rules
use std::cell::UnsafeCell;
use crate::cell::Cell;

#[derive(Copy, Clone)]
enum RefState {
  UnShared,
  Shared(i32),
  Exclusive,
}

pub struct RefCell<T> {
  value: UnsafeCell<T>,
  //reference: isize,
  state: Cell<RefState>, // Cell provide an abiliy of mutable shared reference
}
impl<T> RefCell<T> {
  pub fn new(value: T) -> Self {
    Self {
      value: UnsafeCell::new(value),
      //reference: 0,
      state: Cell::new(RefState::UnShared),
    }
  }

  //pub fn borrow(&self) -> Option<&T> {
  pub fn borrow(&self) -> Option<Ref<'_, T>> {
    match self.state.get() {
      RefState::UnShared => {
        self.state.set(RefState::Shared(1));
        // SAFETY: no exclusive reference have been given out since state would be Exclusive
        Some(Ref { refcell: self })
        //Some(unsafe { &*self.value.get() })
      },
      RefState::Shared(n) => {
        self.state.set(RefState::Shared(n+1));
        // SAFETY: no exclusive reference have been given out since state would be Exclusive
        Some(Ref { refcell: self })
        //Some(unsafe { &*self.value.get() })
      },
      RefState::Exclusive => None,
    }
  }

  //pub fn borrow_mut(&self) -> Option<&mut T> {
  pub fn borrow_mut(&self) -> Option<RefMut<'_, T>> {
    if let RefState::UnShared = self.state.get() {
      self.state.set(RefState::Exclusive);
      // SAFETY: no other reference have been given out since state would be Shared or Exclusive
      //Some(unsafe { &mut *self.value.get() })
      Some(RefMut { refcell: self })
    } else {
      None
    }
  }
}

pub struct Ref<'refcell, T> {
  refcell: &'refcell RefCell<T>,
}

// borrow want get a T but now we give it a Ref<T>
// by impl Deref for Ref borrow can get a T
// this is why we need impl Deref for Ref
impl<T> std::ops::Deref for Ref<'_, T> {
  type Target = T;
  fn deref(&self) -> &Self::Target {
    // SAFETY: 
    // a Ref is only created if no exclusive reference have been given out 
    // once it is given out, state is set to Shared , so no exclusive references are given out .
    // so dereferencing into a shared reference is fine.
    unsafe { &*self.refcell.value.get() }
  }
}
 
impl<T> Drop for Ref<'_, T> {
  fn drop(&mut self) {
    match self.refcell.state.get() {
      RefState::Exclusive | RefState::UnShared => unreachable!(),
      RefState::Shared(1) => {
        self.refcell.state.set(RefState::UnShared);
      },
      RefState::Shared(n) => {
        self.refcell.state.set(RefState::Shared(n-1));
      }
    }
  }
}

pub struct RefMut<'refcell, T> {
  refcell: &'refcell RefCell<T>,
}

impl<T> std::ops::Deref for RefMut<'_, T> {
  type Target = T;
  fn deref(&self) -> &Self::Target {
    // SAFETY: 
    // see safety for DerefMut 
    unsafe { &*self.refcell.value.get() }
  }
}

impl<T> std::ops::DerefMut for RefMut<'_, T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    // SAFETY: 
    // a RefMut is only created if no other reference have been given out 
    // once it is given out, state is set to Exclusive , so no future references are given out .
    // so we have an exclusive lease on the inner value, so mutably dereferencing is fine.
    unsafe { &mut *self.refcell.value.get() }
  }
}
 
impl<T> Drop for RefMut<'_, T> {
  fn drop(&mut self) {
    match self.refcell.state.get() {
      RefState::Shared(_) | RefState::UnShared => unreachable!(),
      RefState::Exclusive => {
        self.refcell.state.set(RefState::UnShared);
      },
    }
  }
}

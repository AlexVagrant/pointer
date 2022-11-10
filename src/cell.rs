use std::cell::UnsafeCell;

// the cell type allow you to modify a value through a shared reference 
// because no  other threads  reference to it
pub struct Cell<T> {
  value: UnsafeCell<T>, 
}
// implied by UnsafeCell
// impl<T> !Sync for Cell<T> {}

impl<T> Cell<T> {
  pub fn new(value: T) -> Self {
      Cell {
        value: UnsafeCell::new(value),
      }
  }

  pub fn set(&self, value: T) {
      // SAFETY: we kown no-one else is concurrently mutating self.value (because !Sync)
      // SAFETY: we kown we're not invalidating any references, because we never give any out
      unsafe {
          *self.value.get() = value;
      }
  }

  pub fn get(&self) -> T 
  where
      T: Copy, // there is no a  reference
  {
      unsafe {
        // SAFETY: we kown no-one else is concurrently mutating since only this thread can mutate
        // (because !Sync), and it is exectuing this function insread; 
        *self.value.get()
      }
  }
}

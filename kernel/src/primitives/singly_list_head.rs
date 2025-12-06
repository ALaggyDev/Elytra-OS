use core::ptr::null_mut;

/// A singly linked list head structure.
#[derive(Debug, Default)]
pub struct SinglyListHead {
    pub next: *mut SinglyListHead,
}

impl SinglyListHead {
    // Create a new empty SinglyListHead.
    pub const fn new() -> Self {
        SinglyListHead { next: null_mut() }
    }

    // Check if the list is empty.
    pub fn is_empty(&mut self) -> bool {
        self.next.is_null()
    }

    // Insert a new entry after this head.
    pub unsafe fn insert_after(&mut self, new: *mut Self) {
        unsafe {
            (*new).next = self.next;
            self.next = new;
        }
    }

    // Pop and return the entry after this head from the list.
    pub unsafe fn pop(&mut self) -> *mut SinglyListHead {
        unsafe {
            let to_delete = self.next;
            if !to_delete.is_null() {
                self.next = (*to_delete).next;
            }
            to_delete
        }
    }
}

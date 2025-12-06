use core::ptr::null_mut;

/// A doubly linked circular list head structure. Basically Linux kernel's `list_head` in Rust.
#[derive(Debug)]
pub struct DoublyListHead {
    pub next: *mut DoublyListHead,
    pub prev: *mut DoublyListHead,
}

impl DoublyListHead {
    /// Initialize a `DoublyListHead` to point to itself.
    pub unsafe fn new_empty(head: *mut Self) {
        unsafe {
            (*head).next = head;
            (*head).prev = head;
        }
    }

    /// Check if the list is empty.
    pub unsafe fn is_empty(head: *mut Self) -> bool {
        unsafe { (*head).next == head }
    }

    /// Insert a new entry after this head.
    pub unsafe fn insert_after(head: *mut Self, new: *mut Self) {
        unsafe {
            (*new).next = (*head).next;
            (*new).prev = head;
            (*(*head).next).prev = new;
            (*head).next = new;
        }
    }

    /// Insert a new entry before this head.
    pub unsafe fn insert_before(head: *mut Self, new: *mut Self) {
        unsafe {
            (*new).next = head;
            (*new).prev = (*head).prev;
            (*(*head).prev).next = new;
            (*head).prev = new;
        }
    }

    /// Delete this entry from the list. This entry will be poisoned after this call (next and prev set to null).
    pub unsafe fn delete(head: *mut Self) {
        unsafe {
            (*(*head).prev).next = (*head).next;
            (*(*head).next).prev = (*head).prev;
            (*head).next = null_mut();
            (*head).prev = null_mut();
        }
    }
}

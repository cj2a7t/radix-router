//! FFI bindings for the C radix tree implementation

use std::ffi::c_void;

#[repr(C)]
#[allow(non_camel_case_types, dead_code)]
pub struct rax {
    _unused: [u8; 0],
}

extern "C" {
    pub fn radix_tree_new() -> *mut c_void;
    pub fn radix_tree_destroy(t: *mut c_void) -> i32;
    pub fn radix_tree_insert(t: *mut c_void, buf: *const u8, len: usize, idx: i32) -> i32;
    pub fn radix_tree_remove(t: *mut c_void, buf: *const u8, len: usize) -> i32;
    pub fn radix_tree_find(t: *mut c_void, buf: *const u8, len: usize) -> *mut c_void;
    pub fn radix_tree_search(
        t: *mut c_void,
        it: *mut c_void,
        buf: *const u8,
        len: usize,
    ) -> *mut c_void;
    pub fn radix_tree_up(it: *mut c_void, buf: *const u8, len: usize) -> i32;
    pub fn radix_tree_stop(it: *mut c_void) -> i32;
    pub fn radix_tree_new_it(t: *mut c_void) -> *mut c_void;
}

/// Safe Rust wrapper around C radix tree
pub struct RadixTreeRaw {
    tree: *mut c_void,
    iterator: *mut c_void,
}

impl RadixTreeRaw {
    pub fn new() -> Self {
        unsafe {
            let tree = radix_tree_new();
            if tree.is_null() {
                panic!("Failed to create radix tree");
            }

            let iterator = radix_tree_new_it(tree);
            if iterator.is_null() {
                radix_tree_destroy(tree);
                panic!("Failed to create radix tree iterator");
            }

            Self { tree, iterator }
        }
    }

    pub fn insert(&mut self, key: &[u8], idx: i32) -> bool {
        unsafe { radix_tree_insert(self.tree, key.as_ptr(), key.len(), idx) == 1 }
    }

    pub fn find(&self, key: &[u8]) -> Option<usize> {
        unsafe {
            let result = radix_tree_find(self.tree, key.as_ptr(), key.len());
            if result.is_null() {
                None
            } else {
                Some(result as usize)
            }
        }
    }

    pub fn search(&mut self, key: &[u8]) -> bool {
        unsafe {
            let result = radix_tree_search(self.tree, self.iterator, key.as_ptr(), key.len());
            !result.is_null()
        }
    }

    pub fn tree_up(&mut self, key: &[u8]) -> Option<usize> {
        unsafe {
            let idx = radix_tree_up(self.iterator, key.as_ptr(), key.len());
            if idx > 0 {
                Some(idx as usize)
            } else {
                None
            }
        }
    }

    pub fn remove(&mut self, key: &[u8]) -> bool {
        unsafe { radix_tree_remove(self.tree, key.as_ptr(), key.len()) == 1 }
    }
}

impl Drop for RadixTreeRaw {
    fn drop(&mut self) {
        unsafe {
            if !self.iterator.is_null() {
                radix_tree_stop(self.iterator);
                libc::free(self.iterator);
                self.iterator = std::ptr::null_mut();
            }
            if !self.tree.is_null() {
                radix_tree_destroy(self.tree);
                self.tree = std::ptr::null_mut();
            }
        }
    }
}

unsafe impl Send for RadixTreeRaw {}
unsafe impl Sync for RadixTreeRaw {}

impl Default for RadixTreeRaw {
    fn default() -> Self {
        Self::new()
    }
}

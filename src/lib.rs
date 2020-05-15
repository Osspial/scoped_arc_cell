//! Shared mutable datastructure, with the mutability tied to the liveliness of a owner struct.

#![no_std]
extern crate alloc;

use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, Ordering};
use crossbeam_utils::atomic::AtomicCell;

pub fn scoped_arc_cell<T: Copy>(val: T) -> (ScopedArcCell<T>, ScopedArcCellOwner<T>) {
    let data = Arc::new(Data {
        val: AtomicCell::new(val),
        is_read_only: AtomicBool::new(false),
    });
    (ScopedArcCell{ data: data.clone() }, ScopedArcCellOwner{ data })
}

pub struct ScopedArcCell<T: Copy> {
    data: Arc<Data<T>>,
}

pub struct ScopedArcCellOwner<T: Copy> {
    data: Arc<Data<T>>,
}

pub struct StoreError<T>(pub T);

struct Data<T> {
    val: AtomicCell<T>,
    is_read_only: AtomicBool,
}

impl<T: Copy> ScopedArcCell<T> {
    pub fn store(&self, val: T) -> Result<(), StoreError<T>> {
        match self.data.is_read_only.load(Ordering::Acquire) {
            false => Ok(self.data.val.store(val)),
            true => Err(StoreError(val)),
        }
    }
    pub fn swap(&self, val: T) -> Result<T, StoreError<T>> {
        match self.data.is_read_only.load(Ordering::Acquire) {
            false => Ok(self.data.val.swap(val)),
            true => Err(StoreError(val)),
        }
    }

    pub fn load(&self) -> T {
        self.data.val.load()
    }

    pub fn as_ptr(&self) -> *mut T {
        self.data.val.as_ptr()
    }
}

impl<T: Copy> ScopedArcCellOwner<T> {
    pub fn store(&self, val: T) {
        self.data.val.store(val)
    }

    pub fn swap(&self, val: T) -> T {
        self.data.val.swap(val)
    }

    pub fn load(&self) -> T {
        self.data.val.load()
    }

    pub fn as_ptr(&self) -> *mut T {
        self.data.val.as_ptr()
    }
}

impl<T: Copy> Drop for ScopedArcCellOwner<T> {
    fn drop(&mut self) {
        self.data.is_read_only.store(true, Ordering::Release);
    }
}

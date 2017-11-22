//! Types encapsulating _handles_ to managed JavaScript memory.
//!
//! 

use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use neon_runtime;
use neon_runtime::raw;
use js::Value;
use js::internal::SuperType;
use js::error::{JsError, Kind};
use vm::{JsResult, Lock};
use vm::internal::LockState;
use scope::Scope;

pub trait Managed: Copy {
    fn to_raw(self) -> raw::Local;

    fn from_raw(h: raw::Local) -> Self;
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Handle<'j, T: Managed + 'j> {
    value: T,
    phantom: PhantomData<&'j T>
}

impl<'j, T: Value + 'j> Handle<'j, T> {
    pub fn lock(self) -> LockedHandle<'j, T> {
        LockedHandle::new(self)
    }
}

impl<'j, T: Managed + 'j> PartialEq for Handle<'j, T> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { neon_runtime::mem::same_handle(self.to_raw(), other.to_raw()) }
    }
}

impl<'j, T: Managed + 'j> Eq for Handle<'j, T> { }

impl<'j, T: Managed + 'j> Handle<'j, T> {
    pub(crate) fn new_internal(value: T) -> Handle<'j, T> {
        Handle {
            value: value,
            phantom: PhantomData
        }
    }
}

impl<'j, T: Value> Handle<'j, T> {
    // This method does not require a scope because it only copies a handle.
    pub fn upcast<U: Value + SuperType<T>>(&self) -> Handle<'j, U> {
        Handle::new_internal(SuperType::upcast_internal(self.value))
    }

    pub fn is_a<U: Value>(&self) -> bool {
        U::downcast(self.value).is_some()
    }

    pub fn downcast<U: Value>(&self) -> Option<Handle<'j, U>> {
        U::downcast(self.value).map(Handle::new_internal)
    }

    pub fn check<U: Value>(&self) -> JsResult<'j, U> {
        match U::downcast(self.value) {
            Some(v) => Ok(Handle::new_internal(v)),
            None => JsError::throw(Kind::TypeError, "type error")
        }
    }
}

impl<'j, T: Managed> Deref for Handle<'j, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<'j, T: Managed> DerefMut for Handle<'j, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct LockedHandle<'j, T: Value + 'j>(Handle<'j, T>);

unsafe impl<'j, T: Value + 'j> Sync for LockedHandle<'j, T> { }

impl<'j, T: Value + 'j> LockedHandle<'j, T> {
    pub fn new(h: Handle<'j, T>) -> LockedHandle<'j, T> {
        LockedHandle(h)
    }

    pub fn unlock<'b, U: Scope<'b>>(self, _: &mut U) -> Handle<'j, T> { self.0 } // unused function?
}

impl<'j, T: Value> Lock for LockedHandle<'j, T> {
    type Internals = LockedHandle<'j, T>;

    unsafe fn expose(self, _: &mut LockState) -> Self::Internals {
        self
    }
}

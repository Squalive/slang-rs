extern crate alloc;
extern crate slang_sys as sys;
extern crate std;

mod types;

pub use types::*;

use std::ffi::c_void;
use std::ptr::{null_mut, NonNull};

unsafe trait Interface: Sized {
	type Vtable;

	#[inline(always)]
	unsafe fn vtable(&self) -> &Self::Vtable {
		unsafe { &**self.as_raw::<*mut Self::Vtable>() }
	}

	#[inline(always)]
	unsafe fn as_raw<T>(&self) -> *mut T {
		self as *const Self as *mut T
	}
}

macro_rules! vcall {
    ($self:expr, $method:ident($($args:expr),*)) => {
		unsafe { ($self.vtable().$method)($self.as_raw(), $($args),*) }
	};
}

#[repr(transparent)]
pub struct Unknown(NonNull<c_void>);

unsafe impl Interface for Unknown {
	type Vtable = sys::ISlangUnknown__bindgen_vtable;
}

impl Clone for Unknown {
	fn clone(&self) -> Self {
		vcall!(self, ISlangUnknown_addRef());
		Self(self.0)
	}
}

impl Drop for Unknown {
	fn drop(&mut self) {
		vcall!(self, ISlangUnknown_release());
	}
}

impl Unknown {
	fn new<T>(ptr: *mut T) -> Option<Self> {
		NonNull::new(ptr).map(|p| Self(p.cast()))
	}
}

#[repr(transparent)]
#[derive(Clone)]
pub struct Blob(Unknown);

unsafe impl Interface for Blob {
	type Vtable = sys::ISlangBlob_vtable;
}

impl Blob {
	pub fn as_slice(&self) -> &[u8] {
		let ptr = vcall!(self, getBufferPointer());
		let size = vcall!(self, getBufferSize());
		unsafe { core::slice::from_raw_parts(ptr as _, size) }
	}
}

#[repr(transparent)]
pub struct GlobalSession(Unknown);

unsafe impl Interface for GlobalSession {
	type Vtable = sys::IGlobalSession_vtable;
}

impl GlobalSession {
	pub fn new() -> Option<Self> {
		let mut ptr = null_mut();
		unsafe { sys::slang_createGlobalSession(sys::SLANG_API_VERSION as _, &mut ptr) };
		Some(Self(Unknown::new(ptr)?))
	}

	pub fn create_session(&self, desc: &SessionDesc) -> Option<Session> {
		let mut ptr = null_mut();
		vcall!(self, createSession(desc.as_raw(), &mut ptr));
		Some(Session(Unknown::new(ptr)?))
	}
}

#[repr(transparent)]
#[derive(Clone)]
pub struct Session(Unknown);
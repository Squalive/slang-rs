#![no_std]

extern crate alloc;
extern crate slang_sys as sys;
extern crate std;

mod types;
mod error;

use core::fmt::Debug;
use core::str::Utf8Error;
pub use error::*;
pub use sys::{
	SlangCompileTarget as CompileTarget,
	SlangDebugInfoLevel as DebugInfoLevel,
	SlangFloatingPointMode as FloatingPointMode,
	SlangLineDirectiveMode as LineDirectiveMode,
	SlangOptimizationLevel as OptimizationLevel,
	SlangSourceLanguage as SourceLanguage,
	SlangStage as Stage,
	SlangUUID as Uuid,
};
pub use types::*;

pub type Result<T> = core::result::Result<T, Error>;

use std::ffi::{c_void, CString};
use std::ptr::{null_mut, NonNull};

unsafe trait Interface: Sized {
	type Vtable;
	const UUID: Uuid;

	#[inline(always)]
	unsafe fn vtable(&self) -> &Self::Vtable {
		unsafe { &**self.as_raw::<*mut Self::Vtable>() }
	}

	#[inline(always)]
	unsafe fn as_raw<T>(&self) -> *mut T {
		unsafe { std::mem::transmute_copy(self) }
	}
}

macro_rules! vcall {
    ($self:expr, $method:ident($($args:expr),*)) => {
		unsafe { ($self.vtable().$method)($self.as_raw(), $($args),*) }
	};
}

macro_rules! vcall_maybe {
    ($self:expr, $method:ident($($args:expr),*)) => {
		{
			let result = vcall!($self, $method($($args),*));
			if result >= 0 {
				Ok(())
			} else {
				Err(result)
			}
		}
	};
}

const fn uuid(data1: u32, data2: u16, data3: u16, data4: [u8; 8]) -> Uuid {
	Uuid {
		data1,
		data2,
		data3,
		data4,
	}
}

#[repr(transparent)]
pub struct Unknown(NonNull<c_void>);

unsafe impl Interface for Unknown {
	type Vtable = sys::ISlangUnknown__bindgen_vtable;
	const UUID: Uuid = uuid(0x00000000, 0x0000, 0x0000, [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46]);
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
	fn new<T>(ptr: *mut T) -> Result<Self> {
		NonNull::new(ptr)
			.map(|p| Self(p.cast()))
			.ok_or(Error::InvalidPtr)
	}
}

#[repr(transparent)]
#[derive(Clone)]
pub struct Castable(Unknown);

unsafe impl Interface for Castable {
	type Vtable = sys::ISlangCastable_vtable;
	const UUID: Uuid = uuid(0x87ede0e1, 0x4852, 0x44b0, [0x8b, 0xf2, 0xcb, 0x31, 0x87, 0x4d, 0xe2, 0x39]);
}

#[repr(transparent)]
#[derive(Clone)]
pub struct Blob(Unknown);

unsafe impl Interface for Blob {
	type Vtable = sys::ISlangBlob_vtable;
	const UUID: Uuid = uuid(0x8BA5FB08, 0x5195, 0x40e2, [0xAC, 0x58, 0x0D, 0x98, 0x9C, 0x3A, 0x01, 0x02]);
}

impl Debug for Blob {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		write!(f, "{}", self.as_str().unwrap_or_default())
	}
}

impl Blob {
	pub fn as_slice(&self) -> &[u8] {
		let ptr = vcall!(self, getBufferPointer());
		let size = vcall!(self, getBufferSize());
		unsafe { core::slice::from_raw_parts(ptr as _, size) }
	}

	pub fn as_str(&self) -> core::result::Result<&str, Utf8Error> {
		core::str::from_utf8(self.as_slice())
	}
}

#[derive(Clone, Copy)]
pub struct ProfileId(sys::SlangProfileID);

impl ProfileId {
	pub const UNKNOWN: ProfileId = ProfileId(sys::SlangProfileID::SlangProfileUnknown);

	pub fn is_unknown(&self) -> bool {
		self.0 == sys::SlangProfileID::SlangProfileUnknown
	}
}

#[repr(transparent)]
pub struct GlobalSession(Unknown);

unsafe impl Interface for GlobalSession {
	type Vtable = sys::IGlobalSession_vtable;
	const UUID: Uuid = uuid(0xc140b5fd, 0xc78, 0x452e, [0xba, 0x7c, 0x1a, 0x1e, 0x70, 0xc7, 0xf7, 0x1c]);
}

impl GlobalSession {
	pub fn new() -> Result<Self> {
		let mut ptr = null_mut();
		unsafe { sys::slang_createGlobalSession(sys::SLANG_API_VERSION as _, &mut ptr) };
		Ok(Self(Unknown::new(ptr)?))
	}

	pub fn create_session(&self, desc: &SessionDesc) -> Result<Session> {
		let mut ptr = null_mut();
		vcall_maybe!(self, createSession(desc.as_raw(), &mut ptr))?;
		// SAFETY: We checked above with maybe
		Ok(Session(Unknown::new(ptr)?))
	}

	pub fn find_profile(&self, name: &str) -> ProfileId {
		let name = CString::new(name).unwrap();
		ProfileId(vcall!(self, findProfile(name.as_ptr())))
	}
}

#[repr(transparent)]
#[derive(Clone)]
pub struct Session(Unknown);
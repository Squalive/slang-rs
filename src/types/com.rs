use crate::Result;
use crate::Uuid;
use alloc::vec::Vec;
use std::ffi::c_void;
use std::path::Path;

pub trait ISlangUnknown {
	fn query_interface(&mut self, uuid: &Uuid) -> Result<*mut c_void>;

	fn add_ref(&mut self) -> u32;

	fn release(&mut self) -> u32;
}

fn slang_unknown_query_interface<T: ISlangUnknown>(
	this: *mut c_void,
	uuid: *const Uuid,
	out_object: *mut *mut c_void,
) -> sys::SlangResult {
	let this: &mut T = unsafe { &mut *(this as *mut T) };

	match this.query_interface(unsafe { &*uuid }) {
		Ok(out) => {
			this.add_ref();
			unsafe { *out_object = out };
			0
		}
		Err(err) => err.into()
	}
}

fn slang_unknown_add_ref<T: ISlangUnknown>(
	this: *mut c_void
) -> u32 {
	let this: &mut T = unsafe { &mut *(this as *mut T) };
	this.add_ref()
}

fn slang_unknown_release<T: ISlangUnknown>(
	this: *mut c_void
) -> u32 {
	let this: &mut T = unsafe { &mut *(this as *mut T) };
	this.release()
}

pub trait ISlangCastable: ISlangUnknown {
	fn cast_as(&mut self, uuid: &Uuid) -> *mut c_void;
}

fn slang_castable_cast_as<T: ISlangCastable>(
	this: *mut c_void,
	uuid: *const Uuid,
) -> *mut c_void {
	let this: &mut T = unsafe { &mut *(this as *mut T) };
	this.cast_as(unsafe { &*uuid })
}

pub trait ISlangBlob: ISlangUnknown {
	fn get(&mut self) -> &[u8];
}

fn slang_blob_get_buffer_pointer<T: ISlangBlob>(
	this: *mut c_void
) -> *const c_void {
	let this: &mut T = unsafe { &mut *(this as *mut T) };
	this.get().as_ptr() as _
}

fn slang_blob_get_buffer_size<T: ISlangBlob>(
	this: *mut c_void,
) -> usize {
	let this: &mut T = unsafe { &mut *(this as *mut T) };
	this.get().len()
}

pub trait ISlangFileSystem: ISlangCastable {
	fn load_file(&mut self, path: &Path, buf: &mut Vec<u8>) -> Result<usize>;
}


// pub(crate) struct RawCom<T> {
// 	value: Option<T>,
// 	ref_count: u32,
// }
//
// impl<T> RawCom<T> {
// 	pub(crate) fn new(value: T) -> Self {
// 		Self {
// 			value: Some(value),
// 			ref_count: 0,
// 		}
// 	}
//
// 	pub(crate) fn add_ref(&mut self) -> u32 {
// 		self.ref_count += 1;
// 		self.ref_count
// 	}
//
// 	pub(crate) fn release(&mut self) -> u32 {
// 		self.ref_count -= 1;
// 		if self.ref_count == 0 {
// 			self.value = None;
// 		}
// 		self.ref_count
// 	}
// }
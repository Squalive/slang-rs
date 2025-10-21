use std::marker::PhantomData;
use std::mem::zeroed;

#[repr(transparent)]
pub struct SessionDesc<'a> {
	inner: sys::slang_SessionDesc,
	_marker: PhantomData<&'a ()>,
}

impl Default for SessionDesc<'_> {
	fn default() -> Self {
		Self {
			inner: sys::slang_SessionDesc {
				structureSize: size_of::<sys::slang_SessionDesc>(),
				..unsafe { zeroed() }
			},
			_marker: PhantomData,
		}
	}
}

impl<'a> SessionDesc<'a> {
	pub(crate) fn as_raw(&self) -> *const sys::slang_SessionDesc {
		&self.inner
	}
}
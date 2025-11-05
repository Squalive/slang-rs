use crate::{Blob, Interface, Result, Unknown, Uuid};
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::ptr::NonNull;
use core::ptr::null_mut;
use std::ffi::{CStr, c_char, c_void};
use std::path::Path;

#[repr(C)]
pub struct RawCom<T> {
    vtable: NonNull<()>,
    ref_count: u32,
    value: T,
}

pub struct Com<T>(Arc<RawCom<T>>);

impl<T> Clone for Com<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Com<T> {
    pub fn new(vtable: *mut (), value: T) -> Option<Self> {
        Some(Self(Arc::new(RawCom {
            vtable: NonNull::new(vtable)?,
            ref_count: 0,
            value,
        })))
    }

    /// # Safety
    /// ISlangUnknown::release() must be called when not being used
    pub unsafe fn into_raw(self) -> *mut RawCom<T> {
        Arc::into_raw(self.0) as *mut RawCom<T>
    }
}

pub trait ISlangUnknown {
    fn is_interface_compatible(&self, uuid: &Uuid) -> bool {
        Unknown::matches(uuid)
    }
}

impl ISlangUnknown for Box<dyn ISlangUnknown> {
    fn is_interface_compatible(&self, uuid: &Uuid) -> bool {
        ISlangUnknown::is_interface_compatible(&**self, uuid)
    }
}

impl ISlangUnknown for () {}

extern "C" fn slang_unknown_query_interface<T: ISlangUnknown>(
    this: *mut sys::ISlangUnknown,
    uuid: *const Uuid,
    out_object: *mut *mut c_void,
) -> sys::SlangResult {
    // SAFETY: this is always Com<T>
    let com = unsafe { &mut *(this as *mut RawCom<T>) };

    if com.value.is_interface_compatible(unsafe { &*uuid }) {
        com.ref_count += 1;
        unsafe { *out_object = this as _ };
        0
    } else {
        -1
    }
}

extern "C" fn slang_unknown_add_ref<T: ISlangUnknown>(this: *mut sys::ISlangUnknown) -> u32 {
    // SAFETY: this is always Com<T>
    let com = unsafe { &mut *(this as *mut RawCom<T>) };
    com.ref_count += 1;
    com.ref_count
}

extern "C" fn slang_unknown_release<T: ISlangUnknown>(this: *mut sys::ISlangUnknown) -> u32 {
    // SAFETY: this is always Com<T>
    let com_ptr = this as *mut RawCom<T>;
    let com = unsafe { &mut *com_ptr };
    com.ref_count -= 1;
    if com.ref_count == 0 {
        drop(unsafe { Arc::from_raw(com_ptr) });
    }
    com.ref_count
}

impl<T: ISlangUnknown> Com<T> {
    const UNKNOWN_VTABLE: sys::ISlangUnknown__bindgen_vtable = sys::ISlangUnknown__bindgen_vtable {
        ISlangUnknown_queryInterface: slang_unknown_query_interface::<T>,
        ISlangUnknown_addRef: slang_unknown_add_ref::<T>,
        ISlangUnknown_release: slang_unknown_release::<T>,
    };

    pub fn new_unknown(value: T) -> Self {
        Self::new(
            &Self::UNKNOWN_VTABLE as *const sys::ISlangUnknown__bindgen_vtable as _,
            value,
        )
        .expect("unknown vtable is invalid")
    }

    pub fn into_unknown(self) -> Unknown {
        // SAFETY: into_raw is always not null
        let unknown = Unknown(unsafe { NonNull::new_unchecked(self.into_raw() as _) });
        unsafe { (unknown.vtable().ISlangUnknown_addRef)(unknown.0.as_ptr().cast()) };
        unknown
    }
}

pub trait ISlangCastable: ISlangUnknown {}

impl ISlangUnknown for Box<dyn ISlangCastable> {
    fn is_interface_compatible(&self, uuid: &Uuid) -> bool {
        ISlangUnknown::is_interface_compatible(&**self, uuid)
    }
}

impl ISlangCastable for Box<dyn ISlangCastable> {}

extern "C" fn slang_castable_cast_as<T: ISlangCastable>(
    this: *mut c_void,
    uuid: *const Uuid,
) -> *mut c_void {
    // SAFETY: this is always Com<T>
    let com = unsafe { &mut *(this as *mut RawCom<T>) };

    if com.value.is_interface_compatible(unsafe { &*uuid }) {
        this
    } else {
        null_mut()
    }
}

impl<T: ISlangCastable> Com<T> {
    const CASTABLE_VTABLE: sys::ISlangCastable_vtable = sys::ISlangCastable_vtable {
        _base: Self::UNKNOWN_VTABLE,
        castAs: slang_castable_cast_as::<T>,
    };

    pub fn new_castable(value: T) -> Self {
        Self::new(
            &Self::CASTABLE_VTABLE as *const sys::ISlangCastable_vtable as _,
            value,
        )
        .expect("castable vtable is invalid")
    }
}

pub trait ISlangBlob: ISlangUnknown {
    fn get(&self) -> &[u8];
}

extern "C" fn slang_blob_get_buffer_pointer<T: ISlangBlob>(this: *mut c_void) -> *const c_void {
    // SAFETY: this is always Com<T>
    let com = unsafe { &mut *(this as *mut RawCom<T>) };
    com.value.get().as_ptr() as _
}

extern "C" fn slang_blob_get_buffer_size<T: ISlangBlob>(this: *mut c_void) -> usize {
    // SAFETY: this is always Com<T>
    let com = unsafe { &mut *(this as *mut RawCom<T>) };
    com.value.get().len()
}

impl<T: ISlangBlob> Com<T> {
    const BLOB_VTABLE: sys::ISlangBlob_vtable = sys::ISlangBlob_vtable {
        _base: Self::UNKNOWN_VTABLE,
        getBufferPointer: slang_blob_get_buffer_pointer::<T>,
        getBufferSize: slang_blob_get_buffer_size::<T>,
    };

    pub fn new_blob(value: T) -> Self {
        Self::new(
            &Self::BLOB_VTABLE as *const sys::ISlangBlob_vtable as _,
            value,
        )
        .expect("blob vtable is invalid")
    }
}

impl ISlangUnknown for Vec<u8> {
    fn is_interface_compatible(&self, uuid: &Uuid) -> bool {
        Unknown::matches(uuid) || Blob::matches(uuid)
    }
}

impl ISlangBlob for Vec<u8> {
    fn get(&self) -> &[u8] {
        &self[..]
    }
}

impl ISlangUnknown for &'_ [u8] {
    fn is_interface_compatible(&self, uuid: &Uuid) -> bool {
        Unknown::matches(uuid) || Blob::matches(uuid)
    }
}

impl ISlangBlob for &'_ [u8] {
    fn get(&self) -> &[u8] {
        self
    }
}

pub trait ISlangFileSystem: ISlangCastable {
    fn load_file(&self, path: &Path, buf: &mut Vec<u8>) -> Result<usize>;
}

impl ISlangUnknown for Box<dyn ISlangFileSystem> {
    fn is_interface_compatible(&self, uuid: &Uuid) -> bool {
        ISlangUnknown::is_interface_compatible(&**self, uuid)
    }
}

impl ISlangCastable for Box<dyn ISlangFileSystem> {}

impl ISlangFileSystem for Box<dyn ISlangFileSystem> {
    fn load_file(&self, path: &Path, buf: &mut Vec<u8>) -> Result<usize> {
        ISlangFileSystem::load_file(&**self, path, buf)
    }
}

extern "C" fn slang_file_system_load_file<T: ISlangFileSystem>(
    this: *mut c_void,
    path: *const c_char,
    out_blob: *mut *mut sys::ISlangBlob,
) -> sys::SlangResult {
    // SAFETY: this is always Com<T>
    let com = unsafe { &mut *(this as *mut RawCom<T>) };

    let path_cstr = unsafe { CStr::from_ptr(path) };
    match path_cstr.to_str() {
        Ok(path_str) => {
            let path = Path::new(path_str);
            let mut buf = Vec::new();
            match com.value.load_file(path, &mut buf) {
                Ok(_) => unsafe {
                    let blob = Com::new_blob(buf).into_raw();
                    let vtable =
                        &*((&*blob).vtable.as_ptr() as *const sys::ISlangUnknown__bindgen_vtable);
                    (vtable.ISlangUnknown_addRef)(blob as _);
                    *out_blob = blob as _;
                    0
                },
                Err(err) => err.into(),
            }
        }
        Err(_) => {
            unsafe { *out_blob = null_mut() };
            -1
        }
    }
}

impl<T: ISlangFileSystem> Com<T> {
    const FILE_SYSTEM_VTABLE: sys::ISlangFileSystem_vtable = sys::ISlangFileSystem_vtable {
        _base: Self::CASTABLE_VTABLE,
        loadFile: slang_file_system_load_file::<T>,
    };

    pub fn new_file_system(value: T) -> Self {
        Self::new(
            &Self::FILE_SYSTEM_VTABLE as *const sys::ISlangFileSystem_vtable as _,
            value,
        )
        .expect("filesystem vtable is invalid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use core::sync::atomic::{AtomicU32, Ordering};

    #[test]
    #[should_panic(expected = "drop")]
    fn make_sure_unknown_drop_is_working() {
        struct DropTest;

        impl Drop for DropTest {
            fn drop(&mut self) {
                panic!("drop");
            }
        }

        impl ISlangUnknown for DropTest {}

        let _ = Com::new_unknown(DropTest);
    }

    #[test]
    fn make_sure_unknown_release_is_working() {
        struct DropTest;

        static COUNTER: AtomicU32 = AtomicU32::new(0);

        impl Drop for DropTest {
            fn drop(&mut self) {
                COUNTER.fetch_add(1, Ordering::Relaxed);
            }
        }

        impl ISlangUnknown for DropTest {}

        let unknown = Com::new_unknown(DropTest).into_unknown();
        drop(unknown);
        assert_eq!(COUNTER.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn vec_blob_is_working() {
        let buf = vec![0u8];
        let blob = Com::new_blob(buf);
        drop(blob);
    }
}

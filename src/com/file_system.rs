use super::{E_CANNOT_OPEN, E_INVALIDARG, E_NOT_FOUND, S_OK, cast_as_impl, query_interface_impl};
use crate::{Castable, Interface, Unknown, uuid};
use alloc::sync::Arc;
use core::sync::atomic::{AtomicU32, Ordering};
use std::{
    ffi::{CStr, c_char, c_void},
    panic::AssertUnwindSafe,
};
use sys::SlangUUID as Uuid;

pub trait FileSystem: Send {
    fn load_file(&self, path: &str) -> std::io::Result<Box<[u8]>>;
}

const FILE_SYSTEM_UUID: Uuid = uuid(
    0x003A09FC,
    0x3A4D,
    0x4BA0,
    [0xAD, 0x60, 0x1F, 0xD8, 0x63, 0xA9, 0x15, 0xAB],
);

#[repr(C)]
pub(crate) struct FileSystemImpl {
    vtable: *const sys::ISlangFileSystem_vtable,
    ref_count: AtomicU32,
    inner: Arc<dyn FileSystem>,
}

impl FileSystemImpl {
    const VTABLE: sys::ISlangFileSystem_vtable = sys::ISlangFileSystem_vtable {
        _base: sys::ISlangCastable_vtable {
            _base: sys::ISlangUnknown__bindgen_vtable {
                ISlangUnknown_queryInterface: Self::query_interface,
                ISlangUnknown_addRef: Self::add_ref,
                ISlangUnknown_release: Self::release,
            },
            castAs: Self::cast_as,
        },
        loadFile: Self::load_file,
    };

    pub(crate) fn new(inner: Arc<dyn FileSystem>) -> Self {
        Self {
            vtable: &Self::VTABLE,
            ref_count: AtomicU32::new(1),
            inner,
        }
    }

    unsafe extern "C" fn query_interface(
        this: *mut sys::ISlangUnknown,
        uuid: *const Uuid,
        out_objecct: *mut *mut c_void,
    ) -> sys::SlangResult {
        unsafe {
            query_interface_impl(
                this,
                uuid,
                out_objecct,
                &[FILE_SYSTEM_UUID, Castable::UUID, Unknown::UUID],
            )
        }
    }

    unsafe extern "C" fn add_ref(this: *mut sys::ISlangUnknown) -> u32 {
        unsafe {
            let prev = (*(this as *mut FileSystemImpl))
                .ref_count
                .fetch_add(1, Ordering::SeqCst);
            prev + 1
        }
    }

    unsafe extern "C" fn release(this: *mut sys::ISlangUnknown) -> u32 {
        unsafe {
            let prev = (*(this as *mut FileSystemImpl))
                .ref_count
                .fetch_sub(1, Ordering::SeqCst);
            if prev == 1 {
                let _ = Box::from_raw(this as *mut FileSystemImpl);
                0
            } else {
                prev - 1
            }
        }
    }

    unsafe extern "C" fn cast_as(this: *mut c_void, uuid: *const Uuid) -> *mut c_void {
        unsafe {
            cast_as_impl(
                this.cast(),
                uuid,
                &[FILE_SYSTEM_UUID, Castable::UUID, Unknown::UUID],
            )
        }
    }

    unsafe extern "C" fn load_file(
        this: *mut c_void,
        path: *const c_char,
        out_blob: *mut *mut sys::ISlangBlob,
    ) -> sys::SlangResult {
        match std::panic::catch_unwind(AssertUnwindSafe(|| unsafe {
            if out_blob.is_null() || path.is_null() {
                return E_INVALIDARG;
            }

            let wrapper = &(*this.cast::<FileSystemImpl>()).inner;

            let path = CStr::from_ptr(path).to_string_lossy();
            match wrapper.load_file(&path) {
                Ok(blob) => {
                    // *Copy* the data into a Slang-owned blob.
                    let blob = sys::slang_createBlob(blob.as_ptr().cast(), blob.len());
                    out_blob.write(blob);
                    S_OK
                }
                Err(err) => {
                    *out_blob = core::ptr::null_mut();
                    match err.kind() {
                        std::io::ErrorKind::NotFound => E_NOT_FOUND,
                        _ => E_CANNOT_OPEN,
                    }
                }
            }
        })) {
            Ok(v) => v,
            Err(_) => E_CANNOT_OPEN,
        }
    }
}

mod file_system;

pub use file_system::*;

use std::ffi::c_void;
use sys::SlangUUID as Uuid;

const fn slang_make_error(fac: i32, code: i32) -> sys::SlangResult {
    let fac_u = fac as u32;
    let code_u = code as u32;
    let v = (fac_u << 16) | code_u | 0x8000_0000;
    v as i32
}

const fn slang_make_core_error(code: i32) -> sys::SlangResult {
    slang_make_error(sys::SLANG_FACILITY_CORE as i32, code)
}

const S_OK: sys::SlangResult = 0;
const E_INVALIDARG: sys::SlangResult = -2147024809;
const E_NOINTERFACE: sys::SlangResult = -2147467262;
const E_CANNOT_OPEN: sys::SlangResult = slang_make_core_error(4);
const E_NOT_FOUND: sys::SlangResult = slang_make_core_error(5);

unsafe fn query_interface_impl(
    this: *mut sys::ISlangUnknown,
    uuid: *const Uuid,
    out_objecct: *mut *mut c_void,
    supported_interfaces: &[Uuid],
) -> sys::SlangResult {
    unsafe {
        if out_objecct.is_null() {
            return E_INVALIDARG;
        } else {
            *out_objecct = core::ptr::null_mut();
        }

        if uuid.is_null() {
            return E_INVALIDARG;
        }

        let lhs = core::slice::from_raw_parts(uuid.cast::<u8>(), core::mem::size_of::<Uuid>());

        let eq = |id: &Uuid| {
            let rhs = core::slice::from_raw_parts(
                (id as *const Uuid).cast::<u8>(),
                core::mem::size_of::<Uuid>(),
            );
            lhs == rhs
        };

        for iface in supported_interfaces {
            if eq(iface) {
                ((*(*this).vtable_).ISlangUnknown_addRef)(this);
                *out_objecct = this.cast();
                return S_OK;
            }
        }

        E_NOINTERFACE
    }
}

unsafe fn cast_as_impl(
    this: *mut sys::ISlangUnknown,
    uuid: *const Uuid,
    supported_interfaces: &[Uuid],
) -> *mut c_void {
    unsafe {
        if uuid.is_null() {
            return core::ptr::null_mut();
        }

        let lhs = core::slice::from_raw_parts(uuid.cast::<u8>(), core::mem::size_of::<Uuid>());

        let eq = |id: &Uuid| {
            let rhs = core::slice::from_raw_parts(
                (id as *const Uuid).cast::<u8>(),
                core::mem::size_of::<Uuid>(),
            );
            lhs == rhs
        };

        for iface in supported_interfaces {
            if eq(iface) {
                // castAs is non-refcounting by contract.
                return this.cast();
            }
        }

        core::ptr::null_mut()
    }
}

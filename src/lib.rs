#![no_std]

extern crate alloc;
extern crate slang_sys as sys;
extern crate std;

pub mod reflect;
pub mod helper;

#[cfg(feature = "preprocess")]
mod preprocess;
mod types;
mod error;

pub use error::*;
#[cfg(feature = "preprocess")]
pub use preprocess::{get_file_type, preprocess, FileType};
pub use sys::{
	SlangCompileTarget as CompileTarget,
	SlangDebugInfoLevel as DebugInfoLevel,
	SlangFloatingPointMode as FloatingPointMode,
	SlangLineDirectiveMode as LineDirectiveMode,
	SlangMatrixLayoutMode as MatrixLayoutMode,
	SlangOptimizationLevel as OptimizationLevel,
	SlangParameterCategory as ParameterCategory,
	SlangResourceAccess as ResourceAccess,
	SlangResourceShape as ResourceShape,
	SlangScalarType as ScalarType,
	SlangSourceLanguage as SourceLanguage,
	SlangStage as Stage,
	SlangUUID as Uuid,
};
pub use types::*;

pub type Result<T> = core::result::Result<T, Error>;

use core::fmt::Debug;
use core::marker::PhantomData;
use core::ptr::{null_mut, NonNull};
use core::str::Utf8Error;
use std::ffi::{c_void, CStr, CString};
use std::path::Path;

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

	#[inline(always)]
	fn matches(uuid: &Uuid) -> bool {
		uuid.eq(&Self::UUID)
	}
}

pub unsafe trait Downcast<T> {
	fn downcast(&self) -> &T;
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
				Err(Error::Code(result))
			}
		}
	};
}

macro_rules! vcall_maybe_diagnostics {
    ($self:expr, $method:ident($($args:expr),*)) => {
		{
			let mut out_diagnostics = null_mut();
			let result = vcall!($self, $method($($args),* , &mut out_diagnostics));
			if result >= 0 {
				Ok(())
			} else if !out_diagnostics.is_null() {
				Err(Error::Blob(Blob(Unknown::new_with_ref(out_diagnostics).unwrap())))
			} else {
				Err(Error::Code(result))
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

// SAFETY: Unknown ptr should guarantee this
unsafe impl Send for Unknown {}

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

	fn new_with_ref<T>(ptr: *mut T) -> Result<Self> {
		let unknown = Self::new(ptr)?;
		vcall!(unknown, ISlangUnknown_addRef());
		Ok(unknown)
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

impl Blob {
	pub fn is_interface_compatible(uuid: &Uuid) -> bool {
		Unknown::matches(uuid) || Blob::matches(uuid)
	}
}

#[repr(transparent)]
#[derive(Clone)]
pub struct FileSystem(Unknown);

unsafe impl Interface for FileSystem {
	type Vtable = sys::ISlangFileSystem_vtable;
	const UUID: Uuid = uuid(0x003A09FC, 0x3A4D, 0x4BA0, [0xAD, 0x60, 0x1F, 0xD8, 0x63, 0xA9, 0x15, 0xAB]);
}

impl FileSystem {
	pub fn is_interface_compatible(uuid: &Uuid) -> bool {
		Unknown::matches(uuid) || Castable::matches(uuid) || FileSystem::matches(uuid)
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

		let mut raw_desc = desc.inner;
		if let Some(file_system) = desc.file_system.as_ref() {
			raw_desc.fileSystem = unsafe { file_system.clone().into_raw() as _ };
		}

		vcall_maybe!(self, createSession(&raw_desc, &mut ptr))?;

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

unsafe impl Interface for Session {
	type Vtable = sys::ISession_vtable;
	const UUID: Uuid = uuid(0x67618701, 0xd116, 0x468f, [0xab, 0x3b, 0x47, 0x4b, 0xed, 0xce, 0xe, 0x3d]);
}

macro_rules! into_module {
    ($module:ident, $diagnostics:ident) => {
		match Unknown::new_with_ref($module) {
			Ok(u) => {
				if let Ok(diagnostics) = Unknown::new_with_ref($diagnostics) {
					tracing::warn!("{}", Blob(diagnostics).as_str().unwrap());
				}
				Ok(Module::new(u))
			}
			Err(err) => {
				if let Ok(diagnostics) = Unknown::new_with_ref($diagnostics) {
					tracing::error!("{}", Blob(diagnostics).as_str().unwrap());
				}
				Err(err)
			}
		}
	};
}

impl Session {
	/** Load a module as it would be by code using `import`. */
	pub fn load_module(&self, module_name: &str) -> Result<Module<'_>> {
		let module_name = CString::new(module_name).map_err(|_| Error::Unknown)?;
		let mut out_diagnostics = null_mut();
		let module = vcall!(self, loadModule(module_name.as_ptr(), &mut out_diagnostics));
		into_module!(module, out_diagnostics)
	}

	/** Load a module from a Slang module blob.*/
	pub fn load_module_from_ir_blob(
		&self,
		module_name: &str,
		path: &str,
		blob: impl ISlangBlob,
	) -> Result<Module<'_>> {
		let module_name = CString::new(module_name).map_err(|_| Error::Unknown)?;
		let path = CString::new(path).map_err(|_| Error::Unknown)?;
		let blob = Com::new_blob(blob).into_unknown();
		let mut out_diagnostics = null_mut();
		let module = vcall!(self, loadModuleFromIRBlob(module_name.as_ptr(), path.as_ptr(), blob.0.as_ptr().cast(), &mut out_diagnostics));
		into_module!(module, out_diagnostics)
	}

	/** Load a module from a string.*/
	pub fn load_module_from_source_string(
		&self,
		module_name: &str,
		path: &str,
		source: &str,
	) -> Result<Module<'_>> {
		let module_name = CString::new(module_name).map_err(|_| Error::Unknown)?;
		let path = CString::new(path).map_err(|_| Error::Unknown)?;
		let source = CString::new(source).map_err(|_| Error::Unknown)?;
		let mut out_diagnostics = null_mut();
		let module = vcall!(self, loadModuleFromSourceString(module_name.as_ptr(), path.as_ptr(), source.as_ptr(), &mut out_diagnostics));
		into_module!(module, out_diagnostics)
	}

	pub fn load_module_from_source(
		&self,
		module_name: &str,
		path: &str,
		source: impl ISlangBlob,
	) -> Result<Module<'_>> {
		let module_name = CString::new(module_name).map_err(|_| Error::Unknown)?;
		let path = CString::new(path).map_err(|_| Error::Unknown)?;
		let source = Com::new_blob(source).into_unknown();
		let mut out_diagnostics = null_mut();
		let module = vcall!(self, loadModuleFromSource(module_name.as_ptr(), path.as_ptr(), source.0.as_ptr().cast(), &mut out_diagnostics));
		into_module!(module, out_diagnostics)
	}

	pub fn loaded_module_count(&self) -> usize {
		vcall!(self, getLoadedModuleCount()) as _
	}

	pub fn get_loaded_module(&self, index: usize) -> Option<Module<'_>> {
		let module = vcall!(self, getLoadedModule(index as _));
		let module = Module::new(Unknown::new_with_ref(module).ok()?);
		Some(module)
	}

	pub fn loaded_modules(&self) -> impl ExactSizeIterator<Item=Module<'_>> {
		(0..self.loaded_module_count()).map(|i| self.get_loaded_module(i).unwrap())
	}

	/** Checks if a precompiled binary module is up-to-date with the current compiler
	     option settings and the source file contents.
	 */
	pub fn is_binary_module_up_to_date(&self, module_path: &str, binary_module_blob: impl ISlangBlob) -> bool {
		let Ok(module_path) = CString::new(module_path) else { return false; };
		let binary_module_blob = Com::new_blob(binary_module_blob).into_unknown();
		vcall!(self, isBinaryModuleUpToDate(module_path.as_ptr(), binary_module_blob.0.as_ptr().cast()))
	}

	/** Combine multiple component types to create a composite component type.

	   The `componentTypes` array must contain `componentTypeCount` pointers
	   to component types that were loaded or created using the same session.

	   The shader parameters and specialization parameters of the composite will
	   be the union of those in `componentTypes`. The relative order of child
	   component types is significant, and will affect the order in which
	   parameters are reflected and laid out.

	   The entry-point functions of the composite will be the union of those in
	   `componentTypes`, and will follow the ordering of `componentTypes`.

	   The requirements of the composite component type will be a subset of
	   those in `componentTypes`. If an entry in `componentTypes` has a requirement
	   that can be satisfied by another entry, then the composition will
	   satisfy the requirement and it will not appear as a requirement of
	   the composite. If multiple entries in `componentTypes` have a requirement
	   for the same type, then only the first such requirement will be retained
	   on the composite. The relative ordering of requirements on the composite
	   will otherwise match that of `componentTypes`.

	   If any diagnostics are generated during creation of the composite, they
	   will be written to `outDiagnostics`. If an error is encountered, the
	   function will return null.

	   It is an error to create a composite component type that recursively
	   aggregates a single module more than once.
	*/
	pub fn create_composite_component_type(&self, component_types: &[ComponentType]) -> Result<ComponentType> {
		let mut composite_component_type = null_mut();
		vcall_maybe_diagnostics!(self,createCompositeComponentType(component_types.as_ptr() as _,component_types.len() as _,&mut composite_component_type))?;
		Ok(ComponentType(Unknown::new_with_ref(composite_component_type)?))
	}
}

#[repr(transparent)]
#[derive(Clone)]
pub struct Metadata(Unknown);

unsafe impl Interface for Metadata {
	type Vtable = sys::IMetadata_vtable;
	const UUID: Uuid = uuid(0x8044a8a3, 0xddc0, 0x4b7f, [0xaf, 0x8e, 0x2, 0x6e, 0x90, 0x5d, 0x73, 0x32]);
}

impl Metadata {
	/** Returns whether a resource parameter at the specified binding location is actually being used
		in the compiled shader. */
	pub fn is_parameter_location_used(
		&self,
		category: ParameterCategory,
		space_index: u64,
		register_index: u64,
	) -> Result<bool> {
		let mut used = false;
		vcall_maybe!(self, isParameterLocationUsed(category, space_index, register_index, &mut used))?;
		Ok(used)
	}

	/** Returns the debug build identifier for a base and debug spirv pair. */
	pub fn get_debug_build_id(&self) -> Result<&str> {
		let id = vcall!(self, getDebugBuildIdentifier());
		unsafe { CStr::from_ptr(id) }
			.to_str()
			.map_err(|_| Error::Unknown)
	}
}

#[repr(transparent)]
#[derive(Clone)]
pub struct ComponentType(Unknown);

unsafe impl Interface for ComponentType {
	type Vtable = sys::IComponentType_vtable;
	const UUID: Uuid = uuid(0x5bc42be8, 0x5c50, 0x4929, [0x9e, 0x5e, 0xd1, 0x5e, 0x7c, 0x24, 0x1, 0x5f]);
}

impl ComponentType {
	pub fn layout(&self, target: i64) -> Result<&reflect::Shader> {
		let mut out_diagnostics = null_mut();
		let ptr = vcall!(self, getLayout(target, &mut out_diagnostics));
		if let Ok(diagnostics) = Unknown::new_with_ref(out_diagnostics) {
			tracing::warn!("{}", Blob(diagnostics).as_str().unwrap());
		}
		Ok(unsafe { &*(ptr as *const reflect::Shader) })
	}

	/** Link this component type against all of its unsatisfied dependencies.

	   A component type may have unsatisfied dependencies. For example, a module
	   depends on any other modules it `import`s, and an entry point depends
	   on the module that defined it.

	   A user can manually satisfy dependencies by creating a composite
	   component type, and when doing so they retain full control over
	   the relative ordering of shader parameters in the resulting layout.

	   It is an error to try to generate/access compiled kernel code for
	   a component type with unresolved dependencies, so if dependencies
	   remain after whatever manual composition steps an application
	   cares to perform, the `link()` function can be used to automatically
	   compose in any remaining dependencies. The order of parameters
	   (and hence the global layout) that results will be deterministic,
	   but is not currently documented. */
	pub fn link(&self) -> Result<ComponentType> {
		let mut out_linked_component_type = null_mut();
		vcall_maybe_diagnostics!(self, link(&mut out_linked_component_type))?;
		Ok(ComponentType(Unknown::new_with_ref(out_linked_component_type)?))
	}

	pub fn target_code(&self, target: i64) -> Result<Blob> {
		let mut code = null_mut();
		vcall_maybe_diagnostics!(self, getTargetCode(target, &mut code))?;
		Ok(Blob(Unknown::new_with_ref(code)?))
	}

	/** Get the compiled code for the entry point at `entryPointIndex` for the chosen `targetIndex`

	   Entry point code can only be computed for a component type that
	   has no specialization parameters (it must be fully specialized)
	   and that has no requirements (it must be fully linked).

	   If code has not already been generated for the given entry point and target,
	   then a compilation error may be detected, in which case `outDiagnostics`
	   (if non-null) will be filled in with a blob of messages diagnosing the error.
	*/
	pub fn entry_point_code(&self, index: i64, target: i64) -> Result<Blob> {
		let mut code = null_mut();
		vcall_maybe_diagnostics!(self, getEntryPointCode(index, target, &mut code))?;
		Ok(Blob(Unknown::new_with_ref(code)?))
	}
}

#[repr(transparent)]
#[derive(Clone)]
pub struct EntryPoint(Unknown);

unsafe impl Interface for EntryPoint {
	type Vtable = sys::IEntryPoint_vtable;
	const UUID: Uuid = uuid(0x8f241361, 0xf5bd, 0x4ca0, [0xa3, 0xac, 0x2, 0xf7, 0xfa, 0x24, 0x2, 0xb8]);
}

unsafe impl Downcast<ComponentType> for EntryPoint {
	fn downcast(&self) -> &ComponentType {
		// SAFETY: Same memory layout
		unsafe { core::mem::transmute(self) }
	}
}

#[repr(transparent)]
#[derive(Clone)]
pub struct Module<'s> {
	base: Unknown,
	_marker: PhantomData<&'s Session>,
}

unsafe impl Interface for Module<'_> {
	type Vtable = sys::IModule_vtable;
	const UUID: Uuid = uuid(0xc720e64, 0x8722, 0x4d31, [0x89, 0x90, 0x63, 0x8a, 0x98, 0xb1, 0xc2, 0x79]);
}

unsafe impl Downcast<ComponentType> for Module<'_> {
	fn downcast(&self) -> &ComponentType {
		// SAFETY: Module have the same memory layout as ComponentType
		unsafe { core::mem::transmute(self) }
	}
}

impl<'s> Module<'s> {
	fn new(base: Unknown) -> Self {
		Self {
			base,
			_marker: PhantomData,
		}
	}

	/// Find and an entry point by name.
	/// Note that this does not work in case the function is not explicitly designated as an entry
	/// point, e.g. using a `[shader("...")]` attribute. In such cases, consider using
	/// `IModule::findAndCheckEntryPoint` instead.
	pub fn find_entry_point_by_name(&self, name: &str) -> Result<EntryPoint> {
		let name = CString::new(name).map_err(|_| Error::Unknown)?;
		let mut out_entry_point = null_mut();
		vcall_maybe!(self, findEntryPointByName(name.as_ptr(), &mut out_entry_point))?;
		let entry_point = EntryPoint(Unknown::new_with_ref(out_entry_point)?);
		Ok(entry_point)
	}

	/// Get number of entry points defined in the module. An entry point defined in a module
	/// is by default not included in the linkage, so calls to `IComponentType::getEntryPointCount`
	/// on an `IModule` instance will always return 0. However `IModule::getDefinedEntryPointCount`
	/// will return the number of defined entry points.
	pub fn entry_point_count(&self) -> usize {
		vcall!(self, getDefinedEntryPointCount()) as _
	}

	/// Get the name of an entry point defined in the module.
	pub fn get_entry_point(&self, index: usize) -> Result<EntryPoint> {
		let mut out_entry_point = null_mut();
		vcall_maybe!(self, getDefinedEntryPoint(index as _, &mut out_entry_point))?;
		Ok(EntryPoint(Unknown::new_with_ref(out_entry_point)?))
	}

	/// Returns a iterator over the entry points defined in this module
	pub fn entry_points(&self) -> impl ExactSizeIterator<Item=EntryPoint> {
		(0..self.entry_point_count()).map(|i| self.get_entry_point(i).unwrap())
	}

	/// Get a serialized representation of the checked module.
	pub fn serialize(&self) -> Result<Blob> {
		let mut out = null_mut();
		vcall_maybe!(self, serialize(&mut out))?;
		Ok(Blob(Unknown::new_with_ref(out)?))
	}

	/// Write the serialized representation of this module to a file.
	pub fn write_to_file(&self, file_name: &Path) -> Result<()> {
		let file_name = CString::new(file_name.to_str().ok_or(Error::Unknown)?).map_err(|_| Error::Unknown)?;
		vcall_maybe!(self, writeToFile(file_name.as_ptr()))
	}

	/// Get the name of the module.
	pub fn name(&self) -> Result<&str> {
		let str = vcall!(self, getName());
		unsafe { CStr::from_ptr(str) }
			.to_str()
			.map_err(|_| Error::Unknown)
	}

	/// Get the path of the module.
	pub fn file_path(&self) -> Result<&str> {
		let str = vcall!(self, getFilePath());
		unsafe { CStr::from_ptr(str) }
			.to_str()
			.map_err(|_| Error::Unknown)
	}

	/// Get the unique identity of the module.
	pub fn unique_id(&self) -> Result<&str> {
		let str = vcall!(self, getUniqueIdentity());
		unsafe { CStr::from_ptr(str) }
			.to_str()
			.map_err(|_| Error::Unknown)
	}

	/// Get the number of dependency files that this module depends on.
	/// This includes both the explicit source files, as well as any
	/// additional files that were transitively referenced (e.g., via
	/// a `#include` directive).
	pub fn dependency_file_count(&self) -> usize {
		vcall!(self, getDependencyFileCount()) as _
	}

	/// Get the path to a file this module depends on.
	pub fn get_dependency_file_path(&self, index: usize) -> Result<&str> {
		let str = vcall!(self, getDependencyFilePath(index as _));
		unsafe { CStr::from_ptr(str) }
			.to_str()
			.map_err(|_| Error::Unknown)
	}

	/// Returns an iterator over the dependency file paths
	pub fn dependency_file_paths(&self) -> impl ExactSizeIterator<Item=&str> {
		(0..self.dependency_file_count()).map(|i| self.get_dependency_file_path(i).unwrap())
	}
}
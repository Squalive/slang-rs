#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use std::ffi::{c_char, c_void};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

macro_rules! fp {
    ( ($($pn:ident: $pt:ty),*) ) => {
		unsafe extern "C" fn(*mut c_void, $($pn: $pt),*)
	};
    ( ($($pn:ident: $pt:ty),*) -> $ret:ty ) => {
		unsafe extern "C" fn(*mut c_void, $($pn: $pt),*) -> $ret
	};
}

#[repr(C)]
pub struct ISlangCastable_vtable {
	pub _base: ISlangUnknown__bindgen_vtable,

	pub castAs: fp!((guid: *const SlangUUID) -> *mut c_void),
}

#[repr(C)]
pub struct ISlangBlob_vtable {
	pub _base: ISlangUnknown__bindgen_vtable,

	pub getBufferPointer: fp!(() -> *const c_void),
	pub getBufferSize: fp!(() -> usize),
}

#[repr(C)]
pub struct ISlangFileSystem_vtable {
	pub _base: ISlangCastable_vtable,

	pub loadFile: fp!((path: *const c_char, outBlob: *mut *mut ISlangBlob) -> SlangResult),
}

#[repr(C)]
pub struct IGlobalSession_vtable {
	pub _base: ISlangUnknown__bindgen_vtable,

	pub createSession: fp!((desc: *const slang_SessionDesc, outSession: *mut *mut slang_ISession) -> SlangResult),
	pub findProfile: fp!((name: *const c_char) -> SlangProfileID),
	pub setDownstreamCompilerPath: fp!((passThrough: SlangPassThrough, path: *const c_char)),
	#[deprecated(note = "Use setLanguagePrelude")]
	pub setDownstreamCompilerPrelude: fp!((passThrough: SlangPassThrough, preludeText: *const c_char)),
	#[deprecated(note = "Use getLanguagePrelude")]
	pub getDownstreamCompilerPrelude: fp!((passThrough: SlangPassThrough, outPrelude: *mut *mut ISlangBlob)),
	pub getBuildTagString: fp!(() -> *const c_char),
	pub setDefaultDownstreamCompiler: fp!((sourceLanguage: SlangSourceLanguage, defaultCompiler: SlangPassThrough) -> SlangResult),
	pub getDefaultDownstreamCompiler: fp!((sourceLanguage: SlangSourceLanguage) -> SlangPassThrough),
	pub setLanguagePrelude: fp!((sourceLanguage: SlangSourceLanguage, preludeText: *const c_char)),
	pub getLanguagePrelude: fp!((sourceLanguage: SlangSourceLanguage, outPrelude: *mut *mut ISlangBlob)),
	pub createCompileRequest: fp!((outCompileRequest: *mut *mut slang_ICompileRequest) -> SlangResult),
	pub addBuiltins: fp!((sourcePath: *const c_char, sourceString: *const c_char)),
	pub setSharedLibraryLoader: fp!((loader: *mut ISlangSharedLibraryLoader)),
	pub getSharedLibraryLoader: fp!(() -> *mut ISlangSharedLibraryLoader),
	pub checkCompileTargetSupport: fp!((target: SlangCompileTarget) -> SlangResult),
	pub checkPassThroughSupport: fp!((passThrough: SlangPassThrough) -> SlangResult),
	pub compileCoreModule: fp!((flags: slang_CompileCoreModuleFlags) -> SlangResult),
	pub loadCoreModule: fp!((coreModule: *const c_void, coreModuleSizeInBytes: usize) -> SlangResult),
	pub saveCoreModule: fp!((archiveType: SlangArchiveType, outBlob: *mut *mut ISlangBlob) -> SlangResult),
	pub findCapability: fp!((name: *const c_char) -> SlangCapabilityID),
	pub setDownstreamCompilerForTransition: fp!((source: SlangCompileTarget, target: SlangCompileTarget, compiler: SlangPassThrough)),
	pub getDownstreamCompilerForTransition: fp!((source: SlangCompileTarget, target: SlangCompileTarget) -> SlangPassThrough),
	pub getCompilerElapsedTime: fp!((outTotalTime: *mut f64, outDownstreamTime: *mut f64)),
	pub setSPIRVCoreGrammar: fp!((jsonPath: *const c_char) -> SlangResult),
	pub parseCommandLineArguments: fp!((argc: i32, argv: *const *const c_char, outSessionDesc: *mut slang_SessionDesc, outAuxAllocation: *mut *mut ISlangUnknown) -> SlangResult),
	pub getSessionDescDigest: fp!((sessionDesc: *mut slang_SessionDesc, outBlob: *mut *mut ISlangBlob) -> SlangResult),
	pub compileBuiltinModule: fp!((module: slang_BuiltinModuleName, flags: slang_CompileCoreModuleFlags) -> SlangResult),
	pub loadBuiltinModule: fp!((module: slang_BuiltinModuleName, moduleData: *const c_void, sizeInBytes: usize) -> SlangResult),
	pub saveBuiltinModule: fp!((module: slang_BuiltinModuleName, archiveType: SlangArchiveType, outBlob: *mut *mut ISlangBlob)),
}

#[repr(C)]
pub struct ISession_vtable {
	pub _base: ISlangUnknown__bindgen_vtable,

	pub getGlobalSession: fp!(() -> *mut slang_IGlobalSession),
	pub loadModule: fp!((moduleName: *const c_char, outDiagnostics: *mut *mut ISlangBlob) -> *mut slang_IModule),
	pub loadModuleFromSource: fp!((
		moduleName: *const c_char,
		path: *const c_char,
		source: *mut ISlangBlob,
		outDiagnostics: *mut *mut ISlangBlob
	) -> *mut slang_IModule),
	pub createCompositeComponentType: fp!((
		componentTypes: *mut *const slang_IComponentType,
		componentTypeCount: SlangInt,
		outCompositeComponentType: *mut *mut slang_IComponentType,
		outDiagnostics: *mut *mut ISlangBlob
	)-> SlangResult),
	pub specializeType: fp!((
		type_: *mut slang_TypeReflection,
		specializationArgs: *const slang_SpecializationArg,
		specializationArgCount: SlangInt,
		outDiagnostics: *mut *mut ISlangBlob
	)),
	pub getTypeLayout: fp!((
		type_: *mut slang_TypeReflection,
		targetIndex: SlangInt,
		rules: slang_LayoutRules,
		outDiagnostics: *mut *mut ISlangBlob
	) -> *mut slang_TypeLayoutReflection),
	pub getContainerType: fp!((
		elementType: *mut slang_TypeReflection,
		containerType: slang_ContainerType,
		outDiagnostics: *mut *mut ISlangBlob
	) -> *mut slang_TypeReflection),
	pub getDynamicType: fp!(() -> *mut slang_TypeReflection),
	pub getTypeRTTIMangledName: fp!((type_: *mut slang_TypeReflection, outNameBlob: *mut *mut ISlangBlob) -> SlangResult),
	pub getTypeConformanceWitnessMangledName: fp!((
		type_: *mut slang_TypeReflection,
		interfaceType: *mut slang_TypeReflection,
		outNameBlob: *mut *mut ISlangBlob
	) -> SlangResult),
	pub getTypeConformanceWitnessSequentialID: fp!((
		type_: *mut slang_TypeReflection,
		interfaceType: *mut slang_TypeReflection,
		outId: *mut u32
	) -> SlangResult),
	pub createCompileRequest: fp!((outCompileRequest: *mut *mut SlangCompileRequest) -> SlangResult),
	pub createTypeConformanceComponentType: fp!((
		type_: *mut slang_TypeReflection,
		interfaceType: *mut slang_TypeReflection,
		outConformance: *mut *mut slang_ITypeConformance,
		conformanceIdOverride: SlangInt,
		outDiagnostics: *mut *mut ISlangBlob
	) -> SlangResult),
	pub loadModuleFromIRBlob: fp!((
		moduleName: *const c_char,
		path: *const c_char,
		source: *mut ISlangBlob,
		outDiagnostics: *mut *mut ISlangBlob
	) -> *mut slang_IModule),
	pub getLoadedModuleCount: fp!(() -> SlangInt),
	pub getLoadedModule: fp!((index: SlangInt) -> *mut slang_IModule),
	pub isBinaryModuleUpToDate: fp!((modulePath: *const c_char, binaryModuleBlob: *mut ISlangBlob) -> bool),
	pub loadModuleFromSourceString: fp!((
		moduleName: *const c_char,
		path: *const c_char,
		string: *const c_char,
		outDiagnostics: *mut *mut ISlangBlob
	)),
	pub getDynamicObjectRTTIBytes: fp!((
		type_: *mut slang_TypeReflection,
		interfaceType: *mut slang_TypeReflection,
		outRTTIDataBuffer: *mut u32,
		bufferSizeInBytes: u32
	) -> SlangResult),
}

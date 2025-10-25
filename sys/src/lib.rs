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
	) -> *mut slang_IModule),
	pub getDynamicObjectRTTIBytes: fp!((
		type_: *mut slang_TypeReflection,
		interfaceType: *mut slang_TypeReflection,
		outRTTIDataBuffer: *mut u32,
		bufferSizeInBytes: u32
	) -> SlangResult),
}

#[repr(C)]
pub struct IMetadata_vtable {
	pub _base: ISlangCastable_vtable,

	pub isParameterLocationUsed: fp!((
		category: SlangParameterCategory,
		spaceIndex: SlangUInt,
		registerIndex: SlangUInt,
		outUsed: *mut bool
	) -> SlangResult),
	pub getDebugBuildIdentifier: fp!(() -> *const c_char),
}

#[repr(C)]
pub struct IComponentType_vtable {
	pub _base: ISlangUnknown__bindgen_vtable,

	pub getSession: fp!(() -> *mut slang_ISession),
	pub getLayout: fp!((targetIndex: SlangInt, outDiagnostics: *mut *mut ISlangBlob) -> *mut slang_ProgramLayout),
	pub getSpecializationParamCount: fp!(() -> SlangInt),
	pub getEntryPointCode: fp!((
		entryPointIndex: SlangInt,
		targetIndex: SlangInt,
		outCode: *mut *mut ISlangBlob,
		outDiagnostics: *mut *mut ISlangBlob
	) -> SlangResult),
	pub getResultAsFileSystem: fp!((
		entryPointIndex: SlangInt,
        targetIndex: SlangInt,
        outFileSystem: *mut *mut ISlangMutableFileSystem
	) -> SlangResult),
	pub getEntryPointHash: fp!((entryPointIndex: SlangInt, targetIndex: SlangInt, outHash: *mut *mut ISlangBlob)),
	pub specialize: fp!((
		specializationArgs: *const slang_SpecializationArg,
		specializationArgCount: SlangInt,
		outSpecializedComponentType: *mut *mut slang_IComponentType,
		outDiagnostics: *mut *mut ISlangBlob
	) -> SlangResult),
	pub link: fp!((outLinkedComponentType: *mut *mut slang_IComponentType, outDiagnostics: *mut *mut ISlangBlob) -> SlangResult),
	pub getEntryPointHostCallable: fp!((
		entryPointIndex: i32,
		targetIndex: i32,
		outSharedLibrary: *mut *mut ISlangSharedLibrary,
		outDiagnostics: *mut *mut ISlangBlob
	) -> SlangResult),
	pub renameEntryPoint: fp!((newName: *const c_char, outEntryPoint: *mut *mut slang_IComponentType) -> SlangResult),
	pub linkWithOptions: fp!((
		outLinkedComponentType: *mut *mut slang_IComponentType,
		compilerOptionEntryCount: u32,
		compilerOptionEntries: *mut slang_CompilerOptionEntry,
		outDiagnostics: *mut *mut ISlangBlob
	) -> SlangResult),
	pub getTargetCode: fp!((targetIndex: SlangInt, outCode: *mut *mut ISlangBlob, outDiagnostics: *mut *mut ISlangBlob) -> SlangResult),
	pub getTargetMetadata: fp!((targetIndex: SlangInt, outMetadata: *mut *mut slang_IMetadata, outDiagnostics: *mut *mut ISlangBlob) -> SlangResult),
	pub getEntryPointMetadata: fp!((entryPointIndex: SlangInt, targetIndex: SlangInt, outMetadata: *mut *mut slang_IMetadata, outDiagnostics: *mut *mut ISlangBlob) -> SlangResult),
}

#[repr(C)]
pub struct IEntryPoint_vtable {
	pub _base: IComponentType_vtable,

	pub getFunctionReflection: fp!(() -> *mut slang_FunctionReflection),
}

#[repr(C)]
pub struct ITypeConformance_vtable {
	pub _base: IComponentType_vtable,
}

#[repr(C)]
pub struct IModule_vtable {
	pub _base: IComponentType_vtable,

	pub findEntryPointByName: fp!((name: *const c_char, outEntryPoint: *mut *mut slang_IEntryPoint) -> SlangResult),
	pub getDefinedEntryPointCount: fp!(() -> SlangInt32),
	pub getDefinedEntryPoint: fp!((index: SlangInt32, outEntryPoint: *mut *mut slang_IEntryPoint) -> SlangResult),
	pub serialize: fp!((outSerializedBlob: *mut *mut ISlangBlob) -> SlangResult),
	pub writeToFile: fp!((fileName: *const c_char) -> SlangResult),
	pub getName: fp!(() -> *const c_char),
	pub getFilePath: fp!(() -> *const c_char),
	pub getUniqueIdentity: fp!(() -> *const c_char),
	pub findAndCheckEntryPoint: fp!((
		name: *const c_char,
        stage: SlangStage,
        outEntryPoint: *mut *mut slang_IEntryPoint,
        outDiagnostics: *mut *mut ISlangBlob
	) -> SlangResult),
	pub getDependencyFileCount: fp!(() -> SlangInt32),
	pub getDependencyFilePath: fp!((index: SlangInt32) -> *const c_char),
	pub getModuleReflection: fp!(() -> *mut slang_DeclReflection),
	pub disassemble: fp!((outDisassembledBlob: *mut *mut ISlangBlob) -> SlangResult),
}
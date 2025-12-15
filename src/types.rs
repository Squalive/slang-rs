mod com;

pub use com::*;

use crate::{
    CompileTarget, DebugInfoLevel, FloatingPointMode, LineDirectiveMode, MatrixLayoutMode,
    OptimizationLevel, ProfileId, SourceLanguage, Stage, reflect,
};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::marker::PhantomData;
use core::mem::zeroed;
use core::ptr::null;
use std::ffi::{CString, c_char};

#[repr(C)]
pub struct SessionDesc<'a> {
    pub(crate) inner: sys::slang_SessionDesc,
    pub(crate) file_system: Option<Com<Box<dyn ISlangFileSystem>>>,
    _marker: PhantomData<&'a ()>,
}

impl Default for SessionDesc<'_> {
    fn default() -> Self {
        Self {
            inner: sys::slang_SessionDesc {
                structureSize: size_of::<sys::slang_SessionDesc>(),
                ..unsafe { zeroed() }
            },
            file_system: None,
            _marker: PhantomData,
        }
    }
}

impl<'a> SessionDesc<'a> {
    pub fn matrix_layout_mode(mut self, mode: MatrixLayoutMode) -> Self {
        self.inner.defaultMatrixLayoutMode = mode;
        self
    }

    pub fn targets(mut self, targets: &'a [TargetDesc]) -> Self {
        self.inner.targets = targets.as_ptr() as _;
        self.inner.targetCount = targets.len() as _;
        self
    }

    pub fn search_paths(mut self, search_paths: &'a [*const c_char]) -> Self {
        self.inner.searchPaths = search_paths.as_ptr();
        self.inner.searchPathCount = search_paths.len() as _;
        self
    }

    pub fn options(mut self, options: &'a CompilerOptions) -> Self {
        self.inner.compilerOptionEntries = options.options.as_ptr() as _;
        self.inner.compilerOptionEntryCount = options.options.len() as _;
        self
    }

    pub fn skip_spirv_validation(mut self, yes: bool) -> Self {
        self.inner.skipSPIRVValidation = yes;
        self
    }

    pub fn file_system(mut self, file_system: impl ISlangFileSystem + 'static) -> Self {
        self.file_system = Some(Com::new_file_system(Box::new(file_system)));
        self
    }
}

#[derive(Default)]
pub struct CompilerOptions {
    strings: Vec<CString>,
    options: Vec<sys::slang_CompilerOptionEntry>,
}

impl CompilerOptions {
    fn push_ints(mut self, name: sys::slang_CompilerOptionName, i0: i32, i1: i32) -> Self {
        self.options.push(sys::slang_CompilerOptionEntry {
            name,
            value: sys::slang_CompilerOptionValue {
                kind: sys::slang_CompilerOptionValueKind::Int,
                intValue0: i0,
                intValue1: i1,
                stringValue0: null(),
                stringValue1: null(),
            },
        });

        self
    }

    fn push_strings(
        mut self,
        name: sys::slang_CompilerOptionName,
        s0: *const c_char,
        s1: *const c_char,
    ) -> Self {
        self.options.push(sys::slang_CompilerOptionEntry {
            name,
            value: sys::slang_CompilerOptionValue {
                kind: sys::slang_CompilerOptionValueKind::String,
                intValue0: 0,
                intValue1: 0,
                stringValue0: s0,
                stringValue1: s1,
            },
        });
        self
    }

    fn push_str(mut self, name: sys::slang_CompilerOptionName, s0: &str) -> Self {
        let s0 = CString::new(s0).unwrap();
        let s0_ptr = s0.as_ptr();
        self.strings.push(s0);

        self.push_strings(name, s0_ptr, null())
    }

    fn push_str2(mut self, name: sys::slang_CompilerOptionName, s0: &str, s1: &str) -> Self {
        let s0 = CString::new(s0).unwrap();
        let s0_ptr = s0.as_ptr();
        self.strings.push(s0);

        let s1 = CString::new(s1).unwrap();
        let s1_ptr = s1.as_ptr();
        self.strings.push(s1);

        self.push_strings(name, s0_ptr, s1_ptr)
    }
}

macro_rules! option {
    ($name:ident, $func:ident($p_name:ident: i32 $p_type:ty)) => {
        #[inline(always)]
        pub fn $func(self, $p_name: $p_type) -> Self {
            self.push_ints(sys::slang_CompilerOptionName::$name, $p_name as _, 0)
        }
    };
    ($name:ident, $func:ident($p_name:ident: &str)) => {
        #[inline(always)]
        pub fn $func(self, $p_name: &str) -> Self {
            self.push_str(sys::slang_CompilerOptionName::$name, $p_name)
        }
    };
    ($name:ident, $func:ident($p_name1:ident: &str, $p_name2:ident: &str)) => {
        #[inline(always)]
        pub fn $func(self, $p_name1: &str, $p_name2: &str) -> Self {
            self.push_str2(sys::slang_CompilerOptionName::$name, $p_name1, $p_name2)
        }
    };
}

impl CompilerOptions {
    option!(MacroDefine, macro_define(key: &str, value: &str));
    option!(Include, include(path: &str));
    option!(Language, language(language: i32 SourceLanguage));
    option!(MatrixLayoutColumn, matrix_layout_column(enable: i32 bool));
    option!(MatrixLayoutRow, matrix_layout_row(enable: i32 bool));
    option!(Stage, stage(stage: i32 Stage));
    option!(Target, target(target: i32 CompileTarget));
    option!(WarningsAsErrors, warnings_as_errors(warning_codes: &str));
    option!(DisableWarnings, disable_warnings(warning_codes: &str));
    option!(EnableWarning, enable_warning(warning_code: &str));
    option!(DisableWarning, disable_warning(warning_code: &str));
    option!(ReportDownstreamTime, report_downstream_time(enable: i32 bool));
    option!(ReportPerfBenchmark, report_perf_benchmark(enable: i32 bool));
    option!(SkipSPIRVValidation, skip_spirv_validation(enable: i32 bool));
    option!(DefaultImageFormatUnknown, default_image_format_unknown(enable: i32 bool));
    option!(DisableDynamicDispatch, disable_dynamic_dispatch(enable: i32 bool));
    option!(DisableSpecialization, disable_specialization(enable: i32 bool));
    option!(FloatingPointMode, floating_point_mode(mode: i32 FloatingPointMode));
    option!(DebugInformation, debug_information(level: i32 DebugInfoLevel));
    option!(LineDirectiveMode, line_directive_mode(mode: i32 LineDirectiveMode));
    option!(Optimization, optimization(level: i32 OptimizationLevel));
    option!(Obfuscate, obfuscate(enable: i32 bool));
    option!(VulkanUseEntryPointName, vulkan_use_entry_point_name(enable: i32 bool));
    option!(GLSLForceScalarLayout, glsl_force_scalar_layout(enable: i32 bool));

    // Debugging
    option!(NoCodeGen, no_code_gen(enable: i32 bool));

    // Experimental
    option!(NoMangle, no_mangle(enable: i32 bool));
    option!(ValidateUniformity, validate_uniformity(enable: i32 bool));
    option!(BindlessSpaceIndex, bindless_space_index(index: i32 u32));
}

#[repr(C)]
pub struct TargetDesc<'a> {
    inner: sys::slang_TargetDesc,
    _marker: PhantomData<&'a ()>,
}

impl Default for TargetDesc<'_> {
    fn default() -> Self {
        Self {
            inner: sys::slang_TargetDesc {
                structureSize: size_of::<sys::slang_TargetDesc>(),
                ..unsafe { zeroed() }
            },
            _marker: PhantomData,
        }
    }
}

impl<'a> TargetDesc<'a> {
    pub fn format(mut self, format: CompileTarget) -> Self {
        self.inner.format = format;
        self
    }

    pub fn profile(mut self, profile: ProfileId) -> Self {
        self.inner.profile = profile.0;
        self
    }

    pub fn options(mut self, options: &'a CompilerOptions) -> Self {
        self.inner.compilerOptionEntries = options.options.as_ptr() as _;
        self.inner.compilerOptionEntryCount = options.options.len() as _;
        self
    }
}

#[derive(Clone)]
pub enum SpecializeArgument<'a> {
    Type(&'a reflect::Type),
    Expr(&'a str),
}

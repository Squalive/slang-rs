use crate::{ParameterCategory, Stage};
use std::ffi::CString;

macro_rules! rcall {
    ($f:ident($s:expr $(,$arg:expr)*)) => {
		unsafe { sys::$f($s as *const _ as _ $(,$arg)*) }
	};
	($f:ident($s:expr $(,$arg:expr)*) as Option<&str>) => {
		unsafe {
			let ptr = sys::$f($s as *const _ as _ $(,$arg)*);
			(!ptr.is_null()).then(|| std::ffi::CStr::from_ptr(ptr).to_str().ok()).flatten()
		}
	};
	($f:ident($s:expr $(,$arg:expr)*) as Option<&$cast:ty>) => {
		unsafe {
			let ptr = sys::$f($s as *const _ as _ $(,$arg)*);
			(!ptr.is_null()).then(|| &*(ptr as *const $cast))
		}
	};
}

#[repr(transparent)]
pub struct Shader(sys::SlangProgramLayout);

impl Shader {
    pub fn global_params_var_layout(&self) -> Option<&VariableLayout> {
        rcall!(spReflection_getGlobalParamsVarLayout(self) as Option<&VariableLayout>)
    }

    pub fn entry_point_count(&self) -> usize {
        rcall!(spReflection_getEntryPointCount(self)) as _
    }

    pub fn entry_point_by_index(&self, index: usize) -> Option<&EntryPoint> {
        rcall!(spReflection_getEntryPointByIndex(self, index as _) as Option<&EntryPoint>)
    }

    pub fn entry_points(&self) -> impl ExactSizeIterator<Item = &EntryPoint> {
        (0..self.entry_point_count()).map(|i| self.entry_point_by_index(i).unwrap())
    }

    pub fn find_entry_point_by_name(&self, name: &str) -> Option<&EntryPoint> {
        let name = CString::new(name).unwrap();
        rcall!(spReflection_findEntryPointByName(self, name.as_ptr()) as Option<&EntryPoint>)
    }

    pub fn find_type_by_name(&self, name: &str) -> Option<&Type> {
        let name = CString::new(name).unwrap();
        rcall!(spReflection_FindTypeByName(self, name.as_ptr()) as Option<&Type>)
    }
}

#[repr(transparent)]
pub struct EntryPoint(sys::SlangEntryPointLayout);

impl EntryPoint {
    pub fn name(&self) -> Option<&str> {
        rcall!(spReflectionEntryPoint_getName(self) as Option<&str>)
    }

    pub fn stage(&self) -> Stage {
        rcall!(spReflectionEntryPoint_getStage(self))
    }

    pub fn compute_thread_group_size(&self) -> [u64; 3] {
        let mut out = [0; 3];
        rcall!(spReflectionEntryPoint_getComputeThreadGroupSize(
            self,
            3,
            &mut out as *mut u64
        ));
        out
    }

    pub fn compute_wave_size(&self) -> u64 {
        let mut out = 0;
        rcall!(spReflectionEntryPoint_getComputeWaveSize(self, &mut out));
        out
    }

    pub fn uses_any_sample_rate_input(&self) -> bool {
        rcall!(spReflectionEntryPoint_usesAnySampleRateInput(self)) != 0
    }

    pub fn var_layout(&self) -> Option<&VariableLayout> {
        rcall!(spReflectionEntryPoint_getVarLayout(self) as Option<&VariableLayout>)
    }

    pub fn result_var_layout(&self) -> Option<&VariableLayout> {
        rcall!(spReflectionEntryPoint_getResultVarLayout(self) as Option<&VariableLayout>)
    }
}

#[repr(transparent)]
pub struct Variable(sys::SlangReflectionVariable);

impl Variable {
    pub fn name(&self) -> Option<&str> {
        rcall!(spReflectionVariable_GetName(self) as Option<&str>)
    }
}

#[repr(transparent)]
pub struct VariableLayout(sys::SlangReflectionVariableLayout);

impl VariableLayout {
    pub fn name(&self) -> Option<&str> {
        self.variable()?.name()
    }

    pub fn variable(&self) -> Option<&Variable> {
        rcall!(spReflectionVariableLayout_GetVariable(self) as Option<&Variable>)
    }

    pub fn type_layout(&self) -> Option<&TypeLayout> {
        rcall!(spReflectionVariableLayout_GetTypeLayout(self) as Option<&TypeLayout>)
    }

    pub fn kind(&self) -> Option<TypeLayoutKind<'_>> {
        self.type_layout().and_then(|tl| tl.kind())
    }

    pub fn category_count(&self) -> u32 {
        self.type_layout().map_or(0, |tl| tl.category_count())
    }

    pub fn category_by_index(&self, index: u32) -> ParameterCategory {
        self.type_layout()
            .map_or(ParameterCategory::None, |tl| tl.category_by_index(index))
    }

    pub fn categories(&self) -> impl ExactSizeIterator<Item = ParameterCategory> {
        (0..self.category_count()).map(|i| self.category_by_index(i))
    }

    pub fn offset(&self, category: ParameterCategory) -> usize {
        rcall!(spReflectionVariableLayout_GetOffset(self, category))
    }

    pub fn binding_space(&self) -> usize {
        rcall!(spReflectionParameter_GetBindingSpace(self)) as _
    }

    pub fn binding_space_with_category(&self, category: ParameterCategory) -> usize {
        rcall!(spReflectionVariableLayout_GetSpace(self, category))
    }

    pub fn semantic_name(&self) -> Option<&str> {
        rcall!(spReflectionVariableLayout_GetSemanticName(self) as Option<&str>)
    }

    pub fn semantic_index(&self) -> usize {
        rcall!(spReflectionVariableLayout_GetSemanticIndex(self))
    }
}

#[repr(transparent)]
pub struct Type(sys::SlangReflectionType);

use sys::spReflection_FindTypeByName;
pub use ty::*;
mod ty {
    use super::*;
    use crate::{ResourceAccess, ResourceShape};

    impl Type {
        pub(crate) unsafe fn as_raw(&self) -> *const sys::SlangReflectionType {
            self as *const _ as _
        }

        pub(super) unsafe fn scalar_type(&self) -> crate::ScalarType {
            rcall!(spReflectionType_GetScalarType(self))
        }

        pub(super) unsafe fn element_count(&self) -> usize {
            rcall!(spReflectionType_GetElementCount(self))
        }

        pub(super) unsafe fn element_type(&self) -> Option<&Type> {
            rcall!(spReflectionType_GetElementType(self) as Option<&Type>)
        }

        pub(super) unsafe fn row_count(&self) -> u32 {
            rcall!(spReflectionType_GetRowCount(self))
        }

        pub(super) unsafe fn column_count(&self) -> u32 {
            rcall!(spReflectionType_GetColumnCount(self))
        }

        pub(super) unsafe fn resource_shape(&self) -> ResourceShape {
            rcall!(spReflectionType_GetResourceShape(self))
        }

        pub(super) unsafe fn resource_access(&self) -> ResourceAccess {
            rcall!(spReflectionType_GetResourceAccess(self))
        }

        pub(super) unsafe fn resource_result_type(&self) -> Option<&Type> {
            rcall!(spReflectionType_GetResourceResultType(self) as Option<&Type>)
        }
    }

    #[derive(Clone, Copy)]
    pub struct ScalarType<'a>(pub(super) &'a Type);

    impl<'a> ScalarType<'a> {
        pub fn scalar_type(self) -> crate::ScalarType {
            unsafe { self.0.scalar_type() }
        }
    }

    #[derive(Clone, Copy)]
    pub struct StructType<'a>(pub(super) &'a Type);

    impl<'a> StructType<'a> {}

    #[derive(Clone, Copy)]
    pub struct ArrayType<'a>(pub(super) &'a Type);

    impl<'a> ArrayType<'a> {
        pub fn element_count(self) -> usize {
            unsafe { self.0.element_count() }
        }

        pub fn element_type(self) -> Option<&'a Type> {
            unsafe { self.0.element_type() }
        }
    }

    #[derive(Clone, Copy)]
    pub struct ResourceType<'a>(pub(super) &'a Type);

    impl<'a> ResourceType<'a> {
        pub fn shape(self) -> ResourceShape {
            unsafe { self.0.resource_shape() }
        }

        pub fn access(self) -> ResourceAccess {
            unsafe { self.0.resource_access() }
        }

        pub fn result_type(self) -> Option<&'a Type> {
            unsafe { self.0.resource_result_type() }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SingleElementContainerType {
    ConstantBuffer,
    ParameterBlock,
    TextureBuffer,
    ShaderStorageBuffer,
}

#[repr(transparent)]
pub struct TypeLayout(sys::SlangReflectionTypeLayout);

pub use type_layout::*;
mod type_layout {
    use super::*;
    use crate::{MatrixLayoutMode, ResourceAccess, ResourceShape};
    use core::fmt::Debug;

    impl TypeLayout {
        pub fn ty(&self) -> Option<&Type> {
            rcall!(spReflectionTypeLayout_GetType(self) as Option<&Type>)
        }

        pub fn kind(&self) -> Option<TypeLayoutKind<'_>> {
            use TypeLayoutKind as L;
            use sys::SlangTypeKind as S;
            let kind = rcall!(spReflectionTypeLayout_getKind(self));
            match kind {
                S::Struct => Some(L::Struct(StructTypeLayout(self))),
                S::Array => Some(L::Array(ArrayTypeLayout(self))),
                S::Matrix => Some(L::Matrix(MatrixTypeLayout(self))),
                S::Vector => Some(L::Vector(ArrayTypeLayout(self))),
                S::Scalar => Some(L::Scalar(ScalarTypeLayout(self))),
                S::Resource => Some(L::Resource(ResourceTypeLayout(self))),
                S::ConstantBuffer => Some(L::SingleElementContainer(
                    ContainerTypeLayout(self),
                    SingleElementContainerType::ConstantBuffer,
                )),
                S::ParameterBlock => Some(L::SingleElementContainer(
                    ContainerTypeLayout(self),
                    SingleElementContainerType::ParameterBlock,
                )),
                S::TextureBuffer => Some(L::SingleElementContainer(
                    ContainerTypeLayout(self),
                    SingleElementContainerType::TextureBuffer,
                )),
                S::ShaderStorageBuffer => Some(L::SingleElementContainer(
                    ContainerTypeLayout(self),
                    SingleElementContainerType::ShaderStorageBuffer,
                )),
                S::SamplerState => Some(L::SamplerState),
                S::GenericTypeParameter => Some(L::GenericTypeParameter),
                S::Interface => Some(L::Interface),
                S::OutputStream => Some(L::OutputStream),
                S::MeshOutput => Some(L::MeshOutput),
                S::Specialized => Some(L::Specialized),
                S::Feedback => Some(L::Feedback),
                S::Pointer => Some(L::Pointer),
                S::DynamicResource => Some(L::DynamicResource),
                _ => None,
            }
        }

        pub fn category_count(&self) -> u32 {
            rcall!(spReflectionTypeLayout_GetCategoryCount(self))
        }

        pub fn category_by_index(&self, index: u32) -> ParameterCategory {
            rcall!(spReflectionTypeLayout_GetCategoryByIndex(self, index))
        }

        pub fn categories(&self) -> impl ExactSizeIterator<Item = ParameterCategory> {
            (0..self.category_count()).map(|i| self.category_by_index(i))
        }

        unsafe fn element_count(&self) -> Option<usize> {
            self.ty().map(|ty| unsafe { ty.element_count() })
        }

        unsafe fn element_type_layout(&self) -> Option<&TypeLayout> {
            rcall!(spReflectionTypeLayout_GetElementTypeLayout(self) as Option<&TypeLayout>)
        }

        unsafe fn element_var_layout(&self) -> Option<&VariableLayout> {
            rcall!(spReflectionTypeLayout_GetElementVarLayout(self) as Option<&VariableLayout>)
        }

        unsafe fn container_var_layout(&self) -> Option<&VariableLayout> {
            rcall!(spReflectionTypeLayout_getContainerVarLayout(self) as Option<&VariableLayout>)
        }
    }

    #[derive(Clone, Copy)]
    pub struct ScalarTypeLayout<'a>(&'a TypeLayout);

    impl Debug for ScalarTypeLayout<'_> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("ScalarTypeLayout").finish()
        }
    }

    impl<'a> ScalarTypeLayout<'a> {
        pub fn ty(self) -> Option<ScalarType<'a>> {
            self.0.ty().map(ScalarType)
        }

        pub fn scalar_type(self) -> Option<crate::ScalarType> {
            self.ty().map(|ty| ty.scalar_type())
        }
    }

    #[derive(Clone, Copy)]
    pub struct StructTypeLayout<'a>(&'a TypeLayout);

    impl Debug for StructTypeLayout<'_> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("StructTypeLayout").finish()
        }
    }

    impl<'a> StructTypeLayout<'a> {
        pub fn field_count(self) -> usize {
            rcall!(spReflectionTypeLayout_GetFieldCount(self.0)) as _
        }

        pub fn field_by_index(self, index: usize) -> Option<&'a VariableLayout> {
            rcall!(spReflectionTypeLayout_GetFieldByIndex(self.0, index as _)
                as Option<&VariableLayout>)
        }

        pub fn fields(&self) -> impl ExactSizeIterator<Item = &'a VariableLayout> {
            (0..self.field_count()).map(|i| self.field_by_index(i).unwrap())
        }

        pub fn find_field_index_by_name(self, name: &str) -> usize {
            let (start, end) = (name.as_ptr(), unsafe { name.as_ptr().add(name.len()) });
            rcall!(spReflectionTypeLayout_findFieldIndexByName(
                self.0,
                start as *const _,
                end as *const _
            )) as _
        }
    }

    #[derive(Clone, Copy)]
    pub struct ArrayTypeLayout<'a>(&'a TypeLayout);

    impl Debug for ArrayTypeLayout<'_> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("ArrayTypeLayout").finish()
        }
    }

    impl<'a> ArrayTypeLayout<'a> {
        pub fn ty(self) -> Option<ArrayType<'a>> {
            self.0.ty().map(ArrayType)
        }

        pub fn element_count(self) -> Option<usize> {
            unsafe { self.0.element_count() }
        }

        pub fn element_type_layout(self) -> Option<&'a TypeLayout> {
            unsafe { self.0.element_type_layout() }
        }

        pub fn element_var_layout(self) -> Option<&'a VariableLayout> {
            unsafe { self.0.element_var_layout() }
        }
    }

    #[derive(Clone, Copy)]
    pub struct MatrixTypeLayout<'a>(&'a TypeLayout);

    impl Debug for MatrixTypeLayout<'_> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("MatrixTypeLayout").finish()
        }
    }

    impl<'a> MatrixTypeLayout<'a> {
        pub fn row_count(self) -> Option<u32> {
            self.0.ty().map(|t| unsafe { t.row_count() })
        }

        pub fn column_count(self) -> Option<u32> {
            self.0.ty().map(|t| unsafe { t.column_count() })
        }

        pub fn matrix_layout_mode(self) -> MatrixLayoutMode {
            rcall!(spReflectionTypeLayout_GetMatrixLayoutMode(self.0))
        }
    }

    #[derive(Clone, Copy)]
    pub struct ResourceTypeLayout<'a>(&'a TypeLayout);

    impl Debug for ResourceTypeLayout<'_> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("ResourceTypeLayout").finish()
        }
    }

    impl<'a> ResourceTypeLayout<'a> {
        pub fn ty(self) -> Option<ResourceType<'a>> {
            self.0.ty().map(ResourceType)
        }

        pub fn shape(self) -> Option<ResourceShape> {
            self.ty().map(ResourceType::shape)
        }

        pub fn access(self) -> Option<ResourceAccess> {
            self.ty().map(ResourceType::access)
        }

        pub fn result_type(self) -> Option<&'a Type> {
            self.ty().and_then(ResourceType::result_type)
        }
    }

    #[derive(Clone, Copy)]
    pub struct ContainerTypeLayout<'a>(&'a TypeLayout);

    impl Debug for ContainerTypeLayout<'_> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("ContainerTypeLayout").finish()
        }
    }

    impl<'a> ContainerTypeLayout<'a> {
        pub fn element_type_layout(self) -> Option<&'a TypeLayout> {
            unsafe { self.0.element_type_layout() }
        }

        pub fn element_var_layout(self) -> Option<&'a VariableLayout> {
            unsafe { self.0.element_var_layout() }
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub enum TypeLayoutKind<'a> {
        Struct(StructTypeLayout<'a>),
        Array(ArrayTypeLayout<'a>),
        Matrix(MatrixTypeLayout<'a>),
        Vector(ArrayTypeLayout<'a>),
        Scalar(ScalarTypeLayout<'a>),
        Resource(ResourceTypeLayout<'a>),
        SingleElementContainer(ContainerTypeLayout<'a>, SingleElementContainerType),
        SamplerState,
        GenericTypeParameter,
        Interface,
        OutputStream,
        MeshOutput,
        Specialized,
        Feedback,
        Pointer,
        DynamicResource,
    }
}

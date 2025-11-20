use slang::GlobalSession;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tracing::metadata::LevelFilter;

fn main() {
    struct Filesystem;

    impl slang::ISlangUnknown for Filesystem {
        fn is_interface_compatible(&self, uuid: &slang::Uuid) -> bool {
            slang::FileSystem::is_interface_compatible(uuid)
        }
    }

    impl slang::ISlangCastable for Filesystem {}

    impl slang::ISlangFileSystem for Filesystem {
        fn load_file(&self, path: &Path, buf: &mut Vec<u8>) -> slang::Result<usize> {
            println!("Trying to load {}", path.display());

            let mut file = File::open(path)?;
            let bytes = file.read_to_end(buf)?;

            println!("Loaded {}", path.display());
            Ok(bytes)
        }
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(LevelFilter::TRACE.into())
                .from_env_lossy(),
        )
        .init();

    let global_session = GlobalSession::new().unwrap();

    let session_options = slang::CompilerOptions::default()
        .optimization(slang::OptimizationLevel::High)
        .emit_spirv_directly(true)
        .floating_point_mode(slang::FloatingPointMode::Fast)
        .vulkan_use_entry_point_name(true);

    let targets = [slang::TargetDesc::default()
        .format(slang::CompileTarget::Spirv)
        .profile(global_session.find_profile("spirv_1_5"))];

    let filesystem = Filesystem;

    let session_desc = slang::SessionDesc::default()
        .targets(&targets)
        .options(&session_options)
        .file_system(filesystem);

    let session = global_session.create_session(&session_desc).unwrap();
    {
        // let _ = session
        //     .load_module_from_ir_blob(
        //         "common",
        //         "examples/utils/common",
        //         include_bytes!("common.slang-module").as_slice(),
        //     )
        //     .unwrap();

        // let common = session.load_module("examples/common").unwrap();

        // common
        //     .write_to_file("examples/common.slang-module")
        //     .unwrap();
        //

        let _prelude = session
            .load_module_from_source_string(
                "prelude",
                "examples/prelude.slang",
                include_str!("prelude.slang"),
            )
            .unwrap();

        let module = session
            .load_module_from_source_string(
                "test",
                "examples/test.slang",
                include_str!("test.slang"),
            )
            .unwrap();
        // let _module = session.load_module("examples/test2").unwrap();
        // let test_str = include_str!("test.slang");
        // let module = session.load_module_from_source_string("test", "examples/test", test_str).unwrap();

        module.write_to_file("examples/test.slang-module").unwrap();

        for dependency_file_path in module.dependency_file_paths() {
            let path = Path::new(dependency_file_path);
            println!("Dependency File Path: {}", path.display());
        }

        // let entry_point = module.find_entry_point_by_name("main").unwrap();

        // let program = session
        //     .create_composite_component_type(&[module.into(), entry_point.into()])
        //     .unwrap();

        // let linked_program = program.link().unwrap();

        // // let reflect = linked_program.layout(0).unwrap();
        // // let var = reflect.global_params_var_layout().unwrap();
        // // print_var_layout(var);
        // // validate_shader(reflect);

        // std::fs::create_dir_all("examples/output").unwrap();

        // let spv = linked_program.entry_point_code(0, 0).unwrap();
        // std::fs::write("examples/output/test.spv", spv.as_slice()).unwrap();

        // let glsl = linked_program.entry_point_code(0, 1).unwrap();
        // std::fs::write("examples/output/test.comp", glsl.as_slice()).unwrap();

        // let wgsl = linked_program.entry_point_code(0, 2).unwrap();
        // std::fs::write("examples/output/test.wgsl", wgsl.as_slice()).unwrap();
    }
}

#[derive(Debug)]
enum ValidateError {
    Unknown,
}

fn validate_shader(shader: &slang::reflect::Shader) {
    fn validate_var_layout(
        var_layout: &slang::reflect::VariableLayout,
        inside_struct: &mut bool,
    ) -> Result<(), ValidateError> {
        match var_layout.kind() {
            Some(slang::reflect::TypeLayoutKind::Struct(layout)) => {
                let mut has_handle = false;
                let mut has_pod = false;
                for field in layout.fields() {
                    match field.kind() {
                        Some(slang::reflect::TypeLayoutKind::Array(_))
                        | Some(slang::reflect::TypeLayoutKind::Matrix(_))
                        | Some(slang::reflect::TypeLayoutKind::Vector(_)) => {
                            has_pod = true;
                        }
                        Some(slang::reflect::TypeLayoutKind::SingleElementContainer(
                            _,
                            slang::reflect::SingleElementContainerType::ConstantBuffer,
                        ))
                        | Some(slang::reflect::TypeLayoutKind::Resource(_))
                        | Some(slang::reflect::TypeLayoutKind::SamplerState) => {
                            has_handle = true;
                        }
                        _ => validate_var_layout(field, inside_struct)?,
                    }
                }

                if has_pod && has_handle {
                    return Err(ValidateError::Unknown);
                }

                *inside_struct = true;
            }
            Some(slang::reflect::TypeLayoutKind::SingleElementContainer(layout, _)) => {
                validate_var_layout(layout.element_var_layout().unwrap(), inside_struct)?;
            }
            _ => {}
        }

        Ok(())
    }

    validate_var_layout(shader.global_params_var_layout().unwrap(), &mut false).unwrap();
}

fn print_var_layout(var: &slang::reflect::VariableLayout) {
    match var.kind() {
        Some(slang::reflect::TypeLayoutKind::Struct(layout)) => {
            println!("{:?}", var.name());

            for category in var.categories() {
                println!(
                    "{category:?} {} {}",
                    var.offset(category),
                    var.binding_space_with_category(category)
                );
            }

            println!();
            for field in layout.fields() {
                print_var_layout(field);
            }
        }
        Some(slang::reflect::TypeLayoutKind::SingleElementContainer(layout, ty)) => {
            println!(
                "{:?} {ty:?} {}",
                var.name(),
                var.offset(slang::ParameterCategory::SubElementRegisterSpace)
            );
            print_var_layout(layout.element_var_layout().unwrap());
        }
        kind => {
            println!("{:?} {:?}", var.name(), kind);

            for category in var.categories() {
                println!(
                    "{category:?} {} {}",
                    var.offset(category),
                    var.binding_space_with_category(category)
                );
            }

            println!();
        }
    }
}

use slang::{BindlessResourceMetadata, GlobalSession, SessionDesc};

fn main() {
    let global = GlobalSession::new().unwrap();

    let targets = [slang::TargetDesc::default()
        .format(slang::CompileTarget::Spirv)
        .profile(global.find_profile("spirv_1_5"))];

    let options = slang::CompilerOptions::default();

    let session = global
        .create_session(&SessionDesc::default().targets(&targets).options(&options))
        .unwrap();

    let module = session.load_module("examples/bindless").unwrap();

    let entry_point = module.find_entry_point_by_name("main").unwrap();

    let composed = session
        .create_composite_component_type(&[module.into(), entry_point.into()])
        .unwrap()
        .link()
        .unwrap();

    // let reflect = composed.layout(0).unwrap();

    // print_var_layout(
    //     reflect
    //         .entry_point_by_index(0)
    //         .unwrap()
    //         .var_layout()
    //         .unwrap(),
    // );

    let target_metadata = composed.target_metadata(0).unwrap();

    let bindless_metadata = target_metadata
        .cast_as::<BindlessResourceMetadata>()
        .unwrap();

    assert!(bindless_metadata.is_using_bindless_resource_heap());
    println!("{}", bindless_metadata.is_using_bindless_resource_heap())
}

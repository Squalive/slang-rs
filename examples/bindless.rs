use slang::{GlobalSession, SessionDesc};

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

    let code = composed.entry_point_code(0, 0).unwrap();
    let code = code.as_slice();
    std::fs::write("examples/bindless.spv", code).unwrap();

    let reflected = rspirv_reflect::Reflection::new_from_spirv(code).unwrap();
    for (set_index, set_bindings) in reflected.get_descriptor_sets().unwrap() {
        println!("set: {set_index}");
        for (binding_index, binding_info) in set_bindings {
            println!("\tbinding: {binding_index}, info: {binding_info:?}");
        }
    }
}

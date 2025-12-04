use slang::{GlobalSession, SessionDesc, SpecializeArgument};

fn main() {
    let global = GlobalSession::new().unwrap();

    let targets = [slang::TargetDesc::default()
        .format(slang::CompileTarget::Spirv)
        .profile(global.find_profile("spirv_1_5"))];

    let session = global
        .create_session(&SessionDesc::default().targets(&targets))
        .unwrap();

    let module = session.load_module("entry_point_specialization").unwrap();

    let f32_ty = module
        .layout(0)
        .unwrap()
        .find_type_by_name("float")
        .unwrap();

    let entry_point = module.find_entry_point_by_name("main").unwrap();

    assert_eq!(entry_point.specialization_param_count(), 2);

    let specialized_entry_point = entry_point
        .specialize(&[
            SpecializeArgument::Expr("true"),
            SpecializeArgument::Type(f32_ty),
        ])
        .expect("cannot specialize entry point");

    let composed = session
        .create_composite_component_type(&[module.into(), specialized_entry_point.into()])
        .unwrap()
        .link()
        .unwrap();

    let _code = composed.entry_point_code(0, 0).unwrap();
}

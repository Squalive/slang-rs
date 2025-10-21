use slang::GlobalSession;

fn main() {
	let global_session = GlobalSession::new().unwrap();

	let session_options = slang::CompilerOptions::default()
		.optimization(slang::OptimizationLevel::High);

	let target_desc = slang::TargetDesc::default()
		.format(slang::CompileTarget::Spirv);

	let targets = [target_desc];
	let search_paths = [c"examples/shaders".as_ptr()];

	let session_desc = slang::SessionDesc::default()
		.targets(&targets)
		.search_paths(&search_paths)
		.options(&session_options);

	let session = global_session.create_session(&session_desc).unwrap();
}

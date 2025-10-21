use std::env;

fn main() {
	println!("cargo:rerun-if-env-changed=VULKAN_SDK");
	println!("cargo:rerun-if-env-changed=SLANG_INCLUDE_DIR");

	let include_dir = if let Ok(dir) = env::var("SLANG_INCLUDE_DIR") {
		dir
	} else if let Ok(dir) = env::var("VULKAN_SDK") {
		format!("{dir}/include/slang")
	} else {
		panic!("The environment variable SLANG_INCLUDE_DIR or VULKAN_SDK must be set");
	};

	let lib_dir = if let Ok(dir) = env::var("SLANG_LIB_DIR") {
		dir
	} else if let Ok(dir) = env::var("VULKAN_SDK") {
		format!("{dir}/lib")
	} else {
		panic!("The environment variable SLANG_LIB_DIR or VULKAN_SDK must be set");
	};

	if !lib_dir.is_empty() {
		println!("cargo:rustc-link-search=native={lib_dir}");
	}

	println!("cargo:rustc-link-lib=dylib=slang");

	let out_dir = env::var("OUT_DIR").expect("Couldn't determine output directory.");

	bindgen::builder()
		.header(format!("{include_dir}/slang.h"))
		.clang_arg("-v")
		.clang_arg("-xc++")
		.clang_arg("-std=c++17")
		.vtable_generation(true)
		.layout_tests(false)
		.derive_copy(true)
		.generate()
		.expect("Couldn't generate bindings")
		.write_to_file(format!("{out_dir}/bindings.rs"))
		.expect("Couldn't write bindings");
}
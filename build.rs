use std::process::Command;
use std::path::Path;

fn main() {
    let build_dir = Path::new("c_build");
    std::fs::create_dir_all(build_dir).expect("创建c_build失败");

    // 生成Makefile/VS项目
    Command::new("cmake")
        .arg("..")
        .current_dir(build_dir)
        .status()
        .expect("CMake失败");

    // 编译C库
    #[cfg(unix)]
    Command::new("make")
        .current_dir(build_dir)
        .status()
        .expect("make失败");

    #[cfg(windows)]
    Command::new("msbuild")
        .arg("QuickSkySys.sln")
        .arg("/p:Configuration=Debug")
        .current_dir(build_dir)
        .status()
        .expect("msbuild失败");

    // 告诉Rust链接C库
    println!("cargo:rustc-link-lib=static=quickskysys");
    println!("cargo:rustc-link-search=native={}", build_dir.display());
}
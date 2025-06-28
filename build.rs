use std::fs;

fn main() {
    fs::create_dir_all("shaders/glsl").unwrap();
    fs::create_dir_all("shaders/spirv").unwrap();

    println!("cargo::rerun-if-changed=shaders/fs_cubes.sc");
    println!("cargo::rerun-if-changed=shaders/vs_cubes.sc");

    let mut ret = 0;

    ret |= bgfx_shaderc::shaderc(&[
        "-f",
        "shaders/fs_cubes.sc",
        "-o",
        "shaders/glsl/fs_cubes.bin",
        "--platform",
        "windows",
        "-p",
        "150",
        "--type",
        "fragment",
        "-O",
        "3",
    ]);
    ret |= bgfx_shaderc::shaderc(&[
        "-f",
        "shaders/vs_cubes.sc",
        "-o",
        "shaders/glsl/vs_cubes.bin",
        "--platform",
        "windows",
        "-p",
        "150",
        "--type",
        "vertex",
        "-O",
        "3",
    ]);

    /*ret |= bgfx_shaderc::shaderc(&[
        "-f",
        "shaders/fs_cubes.sc",
        "-o",
        "shaders/spirv/fs_cubes.bin",
        "--platform",
        "windows",
        "-p",
        "spirv",
        "--type",
        "fragment",
        "-O",
        "3",
    ]);
    ret |= bgfx_shaderc::shaderc(&[
        "-f",
        "shaders/vs_cubes.sc",
        "-o",
        "shaders/spirv/vs_cubes.bin",
        "--platform",
        "windows",
        "-p",
        "spirv",
        "--type",
        "vertex",
        "-O",
        "3",
    ]);*/

    if ret != 0 {
        bgfx_shaderc::shaderc(&["-h"]);
        panic!("shaderc failed");
    }
}

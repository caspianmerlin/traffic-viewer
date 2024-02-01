// fn main() {
//     println!("cargo:rustc-link-lib=dylib:+verbatim=res/resources.res");
// }

fn main() {
    embed_resource::compile("res/res.rc", embed_resource::NONE);
    println!("cargo:rustc-link-lib=static=FSUIPCuser64");
    println!("cargo:rustc-link-search=native=lib/");
}
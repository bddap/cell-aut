fn main() {
    println!("cargo:rerun-if-changed={}", "./shaders");
    println!("cargo:rerun-if-changed={}", "./compute_shaders");
}

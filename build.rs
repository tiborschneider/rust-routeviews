fn main() {
    println!("cargo:rustc-link-lib=rdkafka");
    println!("cargo:rustc-link-lib=parsebgp");
    println!("cargo:rustc-link-lib=bgpstream");
    println!("cargo:rustc-link-lib=wandio");
    println!("cargo:rustc-link-lib=bz2");
    println!("cargo:rustc-link-lib=z");
    println!("cargo:rustc-link-lib=lzo2");
    println!("cargo:rustc-link-lib=lzma");
    println!("cargo:rustc-link-lib=zstd");
    println!("cargo:rustc-link-lib=lz4");
    println!("cargo:rustc-link-lib=curl");
}

#[cfg(not(feature = "napi"))]
fn main() {}

#[cfg(feature = "napi")]
extern crate napi_build;
#[cfg(feature = "napi")]
fn main() {
    napi_build::setup();
}

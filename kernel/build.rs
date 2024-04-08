use bstr::ByteSlice;
use std::process::Command;

fn main() {
	let out = Command::new("fish").arg("build.fish").output().unwrap();
	println!("cargo:warning={}", out.stdout.as_bstr());
}

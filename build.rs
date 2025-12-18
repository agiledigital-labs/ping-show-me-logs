use std::fs;
use std::process::Stdio;

fn main() {
  std::process::Command::new("npm")
    .current_dir("ui")
    .args(["ci"])
    .stdout(Stdio::inherit())
    .spawn()
    .expect("failed :'(")
    .wait()
    .expect("TODO: panic message");

  std::process::Command::new("npm")
    .current_dir("ui")
    .args(["run", "build"])
    .stdout(Stdio::inherit())
    .spawn()
    .expect("failed :'(")
    .wait()
    .expect("TODO: panic message");

  println!(
    "cargo::rustc-env=JS={}",
    fs::read_dir("ui/dist/assets")
      .unwrap()
      .find_map(|file| {
        let path: String = file.unwrap().path().to_str().unwrap().into();

        if path.ends_with("js") {
          Some(path)
        } else {
          None
        }
      })
      .unwrap()
  );

  println!(
    "cargo::rustc-env=CSS={}",
    fs::read_dir("ui/dist/assets")
      .unwrap()
      .find_map(|file| {
        let path: String = file.unwrap().path().to_str().unwrap().into();

        if path.ends_with("css") {
          Some(path)
        } else {
          None
        }
      })
      .unwrap()
  );

  println!("cargo::rerun-if-changed=ui/src");
}

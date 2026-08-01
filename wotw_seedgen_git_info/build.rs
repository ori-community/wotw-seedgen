use std::process::Command;

pub fn main() {
    set_env_with_command("GIT_HEAD", "git", ["rev-parse", "HEAD"]);
    set_env_with_command("GIT_STATUS", "git", ["status", "--porcelain"]);
}

fn set_env_with_command<I>(env: &str, program: &str, args: I)
where
    I: IntoIterator<Item = &'static str>,
{
    let output = Command::new(program).args(args).output().unwrap();
    assert!(output.status.success());

    println!(
        "cargo::rustc-env={env}={}",
        str::from_utf8(&output.stdout).unwrap()
    );
}

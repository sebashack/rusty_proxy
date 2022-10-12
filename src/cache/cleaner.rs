use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

// Edit crontab in order to execute command every minute.
pub fn edit_crontab(user: &str, path: PathBuf) {
    let mut out = Command::new("crontab")
        .args(["-u", user, "-l"])
        .output()
        .unwrap();

    if out.status.success() {
        let mut current_crontab = String::from_utf8(out.stdout).unwrap();
        let new_cronrule = format!("* * * * * {}", path.as_os_str().to_str().unwrap());
        current_crontab.push_str(new_cronrule.as_str());

        let echo = Command::new("echo")
            .arg(current_crontab)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let outo = Command::new("crontab")
            .args(["-u", user, "-"])
            .stdin(echo.stdout.unwrap())
            .output()
            .unwrap();
    } else {
        panic!("Could not list crontab");
    }
}

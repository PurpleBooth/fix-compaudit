extern crate clap;

use std::error;
use std::process::Command;
use std::str;

use clap::crate_authors;
use clap::crate_version;
use clap::App;
use users::get_current_username;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

fn main() -> Result<()> {
    App::new(env!("CARGO_PKG_NAME"))
        .version(crate_version!())
        .author(crate_authors!())
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .get_matches();

    fix_comp_audit_problems(list_compuaudit_problems)
}

fn list_compuaudit_problems() -> Result<Vec<std::path::PathBuf>> {
    let compaudit_check_command = "autoload -U compaudit && compaudit";
    let command_output = Command::new("zsh")
        .arg("-c")
        .arg(&compaudit_check_command)
        .output()?;

    eprint!("{}", str::from_utf8(&command_output.stderr)?);

    match command_output.status.code() {
        None => {
            return Err(Box::from(format!(
                "Command failed, terminated without exit code (probably via signal): {}",
                &compaudit_check_command
            )))
        }
        Some(_) => {}
    }

    let output = str::from_utf8(&command_output.stdout)?;

    Ok(output
        .lines()
        .map(|x| std::path::PathBuf::from(x.trim()))
        .collect())
}

fn fix_comp_audit_problems(
    problem_lister: impl FnOnce() -> Result<Vec<std::path::PathBuf>>,
) -> Result<()> {
    let problems = problem_lister()?;
    let whoami: String = String::from(
        get_current_username()
            .ok_or("Couldn't get user")?
            .to_str()
            .ok_or("Couldn't convert username to string")?,
    );

    for problem in problems {
        println!("Fixing: {:?}", &problem);

        // We use sudo here, so we can gain permission to change the user group.
        // It's not great, but I don't have a better solution.
        Command::new("sudo")
            .arg("chown")
            .arg("-R")
            .arg(&whoami)
            .arg(&problem)
            .status()?;
        Command::new("chmod")
            .arg("-R")
            .arg("g-w")
            .arg(&problem)
            .status()?;
        Command::new("chmod")
            .arg("-R")
            .arg("o-w")
            .arg(&problem)
            .status()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::File;
    use std::os::unix::fs::MetadataExt;
    use std::os::unix::fs::PermissionsExt;

    use tempfile::tempdir;
    use users::get_current_uid;

    use super::*;

    const WRITABLE_GROUP_BITMASK: u32 = 0o20;
    const WRITABLE_OTHER_BITMASK: u32 = 0o2;

    #[test]
    fn it_does_nothing_on_no_errors() {
        fn mock_comp_audit() -> Result<Vec<std::path::PathBuf>> {
            Ok(Vec::new())
        }

        assert_eq!(true, fix_comp_audit_problems(mock_comp_audit).is_ok())
    }

    #[test]
    fn on_getting_the_list_failing_we_raise_that() {
        fn mock_comp_audit() -> Result<Vec<std::path::PathBuf>> {
            Err(Box::from("test"))
        }

        assert_eq!(true, fix_comp_audit_problems(mock_comp_audit).is_err())
    }

    #[test]
    fn files_belonging_to_others_change_to_belong_to_me() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        File::create(&file_path).unwrap();
        let command_output = Command::new("sudo")
            .arg("chown")
            .arg("root")
            .arg(&file_path)
            .status()
            .expect("Chown failed?");
        assert_eq!(command_output.code().unwrap(), 0);

        let mock_comp_audit = || -> Result<Vec<std::path::PathBuf>> {
            Ok(vec![std::path::PathBuf::from(&file_path.to_str().unwrap())])
        };

        assert_eq!(true, fix_comp_audit_problems(mock_comp_audit).is_ok());
        let metadata = fs::metadata(file_path).unwrap();
        assert_eq!(metadata.uid(), get_current_uid());
    }

    #[test]
    fn files_belonging_to_others_inside_that_directory_change_to_belong_to_me() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().join("demo");

        fs::create_dir_all(&dir_path).unwrap();

        let file_path = dir_path.join("test.txt");
        File::create(&file_path).unwrap();

        let command_output = Command::new("sudo")
            .arg("chown")
            .arg("root")
            .arg(&file_path)
            .status()
            .expect("Chown failed?");
        assert_eq!(command_output.code().unwrap(), 0);

        let mock_comp_audit = || -> Result<Vec<std::path::PathBuf>> {
            Ok(vec![std::path::PathBuf::from(&dir_path.to_str().unwrap())])
        };

        assert_eq!(true, fix_comp_audit_problems(mock_comp_audit).is_ok());
        let metadata = fs::metadata(file_path).unwrap();
        assert_eq!(metadata.uid(), get_current_uid());
    }

    #[test]
    fn files_that_are_writable_by_others_are_protected() {
        let file_path = tempdir().unwrap().into_path().join("test.txt");
        File::create(&file_path).unwrap();

        let command_output = Command::new("chmod")
            .arg("ag+w")
            .arg(&file_path)
            .status()
            .expect("Changing permissions failed?");
        assert_eq!(command_output.code().unwrap(), 0);

        let mock_comp_audit = || -> Result<Vec<std::path::PathBuf>> {
            Ok(vec![std::path::PathBuf::from(&file_path.to_str().unwrap())])
        };

        assert_eq!(true, fix_comp_audit_problems(mock_comp_audit).is_ok());
        let metadata = fs::metadata(file_path).unwrap();
        let actual = metadata.permissions().mode();

        assert_ne!(
            actual & WRITABLE_OTHER_BITMASK,
            WRITABLE_OTHER_BITMASK,
            "Other has write ({:o}, {:o})",
            actual & WRITABLE_OTHER_BITMASK,
            actual
        );
    }

    #[test]
    fn files_that_are_writable_by_group_are_protected() {
        let file_path = tempdir().unwrap().into_path().join("test.txt");
        File::create(&file_path).unwrap();

        let command_output = Command::new("chmod")
            .arg("ag+w")
            .arg(&file_path)
            .status()
            .expect("Changing permissions failed?");
        assert_eq!(command_output.code().unwrap(), 0);

        let mock_comp_audit = || -> Result<Vec<std::path::PathBuf>> {
            Ok(vec![std::path::PathBuf::from(&file_path.to_str().unwrap())])
        };

        assert_eq!(true, fix_comp_audit_problems(mock_comp_audit).is_ok());
        let metadata = fs::metadata(file_path).unwrap();
        let actual = metadata.permissions().mode();

        assert_ne!(
            actual & WRITABLE_GROUP_BITMASK,
            WRITABLE_GROUP_BITMASK,
            "Group has write ({:o}, {:o})",
            actual & WRITABLE_GROUP_BITMASK,
            actual
        );
    }

    #[test]
    fn files_that_are_in_a_directory_that_belongs_to_someone_else_are_protected() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().join("demo");

        fs::create_dir_all(&dir_path).unwrap();

        let file_path = dir_path.join("test.txt");
        File::create(&file_path).unwrap();

        let command_output = Command::new("chmod")
            .arg("ag+w")
            .arg(&file_path)
            .status()
            .expect("Changing permissions failed?");
        assert_eq!(command_output.code().unwrap(), 0);

        let command_output = Command::new("sudo")
            .arg("chown")
            .arg("root")
            .arg(&dir_path)
            .status()
            .expect("Chown failed?");
        assert_eq!(command_output.code().unwrap(), 0);

        let mock_comp_audit = || -> Result<Vec<std::path::PathBuf>> {
            Ok(vec![std::path::PathBuf::from(&dir_path.to_str().unwrap())])
        };

        assert_eq!(true, fix_comp_audit_problems(mock_comp_audit).is_ok());
        let metadata = fs::metadata(file_path).unwrap();
        let actual = metadata.permissions().mode();

        assert_eq!(metadata.uid(), get_current_uid());
        assert_ne!(
            actual & WRITABLE_GROUP_BITMASK,
            WRITABLE_GROUP_BITMASK,
            "Group has write ({:o}, {:o})",
            actual & WRITABLE_GROUP_BITMASK,
            actual
        );
        assert_ne!(
            actual & WRITABLE_OTHER_BITMASK,
            WRITABLE_OTHER_BITMASK,
            "Other has write ({:o}, {:o})",
            actual & WRITABLE_OTHER_BITMASK,
            actual
        );
    }
}

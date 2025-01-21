use std::env;
use std::error::Error;
use std::process::Command;

// Elevate the client if possible
#[cfg(target_os = "windows")]
pub fn run_as_admin() -> Result<(), Box<dyn Error>> {
    let exe_path = env::current_exe()?;

    // Collect arguments, skipping the program path
    let args: Vec<String> = env::args().skip(1).collect();

    // Format the command to execute using PowerShell's Start-Process cmdlet
    let command = format!(
        "Start-Process \"{}\" -ArgumentList \"{}\" -Verb runAs",
        exe_path.display(),
        args.join(" ")
    );
    // Use `powershell` to request elevation
    match Command::new("powershell")
        .arg("-Command")
        .arg(command)
        .spawn()
    {
        Ok(mut child) => {
            // Wait for the child process to finish
            match child.wait() {
                Ok(status) => {
                    if !status.success() {
                        return Err("Failed to launch the process as administrator.".into());
                    }
                }
                Err(e) => {
                    return Err(format!("Failed to wait on child process: {}", e).into());
                }
            }
        }
        Err(e) => {
            return Err(format!("Failed to execute process with runas: {}", e).into());
        }
    }

    Ok(())
}

#[cfg(target_os = "linux")]
pub fn run_as_admin() -> Result<(), Box<dyn Error>> {
    let exe_path = env::current_exe()?;

    // Collect arguments, skipping the program path
    let args: Vec<String> = env::args().skip(1).collect();

    // Use `sudo` to request root privileges on Linux
    match Command::new("sudo")
        .arg(exe_path)
        .args(&args)
        .spawn()
    {
        Ok(mut child) => {
            // Wait for the child process to finish
            match child.wait() {
                Ok(status) => {
                    if !status.success() {
                        return Err("Failed to launch the process as administrator.".into());
                    }
                }
                Err(e) => {
                    return Err(format!("Failed to wait on child process: {}", e).into());
                }
            }
        }
        Err(e) => {
            return Err(format!("Failed to execute process with runas: {}", e).into());
        }
    }

    Ok(())
}

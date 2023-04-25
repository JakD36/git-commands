use std::{env, process};

fn main()
{
    let current_branch = env::args().nth(1).expect("Need to provide the head ref for the current branch!");
    let remote_branch = env::args().nth(2).expect("Need to provide a branch to archive!");
    
    let i = remote_branch.find("/").expect("Failed to find separator in branch name");
    let remote = &remote_branch[0..i];
    let remote_branch_name = &remote_branch[i+1..];
    let archived_branch_name = String::from("archive/") + remote_branch_name;
    
    if remote_branch_name.to_lowercase().starts_with("archive/")
    {
        eprintln!("This branch is already archived");
        process::exit(1);
    }
    
    let git_path = if cfg!(target_os = "windows")
    {
        Err("Windows is not currently supported!")
    }
    else
    {
        Ok("git")
    };

    let git_path = git_path.expect("Not a supported platform");
    
    // Pull Original branch
    process::Command::new(git_path)
        .arg("checkout")
        .arg("-b")
        .arg(&archived_branch_name)
        .arg(&remote_branch)
        .output().expect("Failed to pull the original branch");

    // Push archived branch
    let output = process::Command::new(git_path)
        .arg("push")
        .arg(&remote)
        .arg(&archived_branch_name)
        .output().expect("Failed to push the archived branch");

    println!("{}",String::from_utf8_lossy(&output.stdout));

    // Set upstream to archived branch
    let output = process::Command::new(git_path)
        .arg("push")
        .arg(&remote)
        .arg("-u")
        .arg(&archived_branch_name)
        .output().expect("Failed to set upstream to archived branch");

    println!("{}",String::from_utf8_lossy(&output.stdout));

    // Delete original branch
    let output = process::Command::new(git_path)
        .arg("push")
        .arg(&remote)
        .arg("--delete")
        .arg(&remote_branch_name)
        .output().expect("Failed to delete the original branch");
    
    println!("{}",String::from_utf8_lossy(&output.stdout));

    // Change back to original branch
    process::Command::new(git_path)
        .arg("checkout")
        .arg(current_branch)
        .output().expect("Failed to change back to the original branch");

    // Delete the local archived branch
    let output = process::Command::new(git_path)
        .arg("branch")
        .arg("--delete")
        .arg(archived_branch_name)
        .output().expect("Failed to delete the local copy of the archived branch");

    println!("{}",String::from_utf8_lossy(&output.stdout));
}

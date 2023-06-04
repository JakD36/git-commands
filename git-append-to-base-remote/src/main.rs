use std::process;
use std::process::exit;
use clap::{Parser};

#[derive(Parser)]
struct CliInputs
{
    current_branch: String,
    target : String,
    base : String,
}

fn handle_process_output(output: process::Output) -> Result<Option<String>, Option<String>>
{
    if output.status.success() == false
    {
        if output.stderr.len() > 0
        {
            return Err(Some(String::from_utf8_lossy(&output.stderr).to_string()));
        }
        return Err(None);
    }
    if output.stdout.len() > 0
    {
        return Ok(Some(String::from_utf8_lossy(&output.stdout).to_string()));
    }
    return Ok(None);
}

fn main()
{
    let args = CliInputs::parse();
    
    let i = args.target.find("/").expect("Failed to find separator in branch name");
    let remote = &args.target[0..i];
    let remote_branch_name = &args.target[i+1..];

    let base = args.base + "/";
    let new_branch_name = base.clone() + remote_branch_name;
    
    if remote_branch_name.to_lowercase().starts_with(&base)
    {
        eprintln!("This branch already has this base");
        process::exit(1);
    }
    
    // Pull Original branch
    let output = process::Command::new("git")
        .arg("checkout")
        .arg("-b")
        .arg(&new_branch_name)
        .arg(&args.target)
        .output().expect("Failed to run git checkout on target branch");
    match handle_process_output(output) {
        Ok(Some(msg)) => println!("{}", msg),
        Ok(None) => {},
        Err(Some(msg)) => {eprintln!("{}", msg); exit(2)},
        Err(None) => {exit(2)}
    };
    
    // Push archived branch
    let output = process::Command::new("git")
        .arg("push")
        .arg(&remote)
        .arg(&new_branch_name)
        .output().expect(format!("Failed to run git push of the branch with the new prefix {}", base).as_str());
    match handle_process_output(output) {
        Ok(Some(msg)) => println!("{}", msg),
        Ok(None) => {},
        Err(Some(msg)) => {eprintln!("{}", msg); exit(2)},
        Err(None) => {exit(2)}
    };

    // Set upstream to archived branch
    let output = process::Command::new("git")
        .arg("push")
        .arg(&remote)
        .arg("-u")
        .arg(&new_branch_name)
        .output().expect("Failed run git push -u to set upstream to the new branch");
    match handle_process_output(output) {
        Ok(Some(msg)) => println!("{}", msg),
        Ok(None) => {},
        Err(Some(msg)) => {eprintln!("{}", msg); exit(2)},
        Err(None) => {exit(2)}
    };

    // Delete original branch
    let output = process::Command::new("git")
        .arg("push")
        .arg(&remote)
        .arg("--delete")
        .arg(&remote_branch_name)
        .output().expect("Failed to run git push --delete on the original branch");
    match handle_process_output(output) {
        Ok(Some(msg)) => println!("{}", msg),
        Ok(None) => {},
        Err(Some(msg)) => {eprintln!("{}", msg); exit(2)},
        Err(None) => {exit(2)}
    };

    // Change back to original branch
    let output = process::Command::new("git")
        .arg("checkout")
        .arg(args.current_branch)
        .output().expect("Failed to run git checkout to change back to the users branch");
    match handle_process_output(output) {
        Ok(Some(msg)) => println!("{}", msg),
        Ok(None) => {},
        Err(Some(msg)) => {eprintln!("{}", msg); exit(2)},
        Err(None) => {exit(2)}
    };

    // Delete the local archived branch
    let output = process::Command::new("git")
        .arg("branch")
        .arg("--delete")
        .arg(new_branch_name)
        .output().expect("Failed to run git branch --delete to delete the local copy of the new branch");
    match handle_process_output(output) {
        Ok(Some(msg)) => println!("{}", msg),
        Ok(None) => {},
        Err(Some(msg)) => {eprintln!("{}", msg); exit(2)},
        Err(None) => {exit(2)}
    };
}

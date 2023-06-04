use std::process;
use std::process::{exit};
use clap::{Parser};

#[derive(Parser)]
struct CliInputs
{
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
    
    let old_name = &args.target;
    let base = args.base + "/";
    let new_branch_name = base.clone() + &args.target;

    if old_name.to_lowercase().starts_with(&base)
    {
        eprintln!("The branch {} already has the base {}", args.target, base);
        exit(1);
    }
    
    let output = process::Command::new("git")
        .arg("show-ref")
        .arg("--verify")
        .arg("--quiet")
        .arg(&new_branch_name)
        .output().expect("Failed to run git command to check if a branch with the new name already exists");
    if output.status.success()
    {
        eprintln!("A branch already exists with the name {}", new_branch_name);
        exit(2)
    }

    let output = process::Command::new("git")
        .arg("branch")
        .arg("-m")
        .arg(old_name)
        .arg(new_branch_name)
        .output().expect("Failed to run the git command to rename the branch");
    match handle_process_output(output)
    {
        Ok(Some(msg)) => println!("{}", msg),
        Ok(None) => {},
        Err(Some(msg)) => {eprintln!("{}", msg); exit(2)},
        Err(None) => {exit(2)}
    }
}

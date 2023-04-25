use std::{env, process};

fn main() 
{
    let old_name = env::args().nth(1).expect("Need to provide a branch to archive!");
    if old_name.to_lowercase().starts_with("archive/")
    {
        eprintln!("This branch is already archived");
        process::exit(1);
    }
    
    let new_name = String::from("archive/") + &old_name;
    
    let cmd = if cfg!(target_os = "windows")
    {
        Err("Windows is not currently supported!")
    }
    else 
    {
        let mut command = process::Command::new("git");
            command.arg("branch")
            .arg("-m")
            .arg(old_name)
            .arg(new_name);
        Ok(command)
    };
    
    match cmd {
        Ok(mut cmd) => 
            {
                cmd.output().expect("Git Command Failed to execute!");
            }
        Err(err) => {eprintln!("{}",err);}
    };
}

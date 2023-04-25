use std::process;
use std::collections::HashMap;
use std::process::Stdio;
use clap::{Parser, ValueEnum};
use chrono::prelude::*;
use chrono::Duration;
use std::io::{self, Write};
use rayon::prelude::*;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode
{
    Merged,
    NoMerge
}

#[derive(Parser)]
struct CliInputs
{
    #[arg(short = 'b', long = "branch", required = true)]
    branch : String,
    #[arg(short = 'd', long = "days", required = true)]
    days : i64,
    #[arg(short = 'e', long = "exclude")]
    exclude : Option<Vec<String>>,

    #[arg(value_enum, default_value_t = Mode::Merged)]
    mode: Mode 
}

fn main()
{
    let git_path = if cfg!(target_os = "windows")
    {
        Err("Windows is not currently supported!")
    }
    else
    {
        Ok("git")
    };

    let git_path = git_path.expect("Not a supported platform");
    
    let args = CliInputs::parse();
    let remote_branches = get_remote_branches(git_path, &args.branch, args.mode);
    
    let remote_branches : Vec<&str> = if args.exclude.is_some() 
    { 
        let exclude_start_with = args.exclude.unwrap(); 
        remote_branches.lines()
            .filter_map(|x|
                {
                    let trimmed = x.trim();
                    for exclusion in &exclude_start_with
                    {
                        if trimmed.starts_with(exclusion) && trimmed.contains("HEAD") == false
                        {
                            return Some(trimmed);
                        }
                    }
                    return None;
                })
            .collect() 
    } 
    else 
    { 
        remote_branches.lines()
            .filter_map(|x| 
                { 
                    let trimmed = x.trim();
                    if trimmed.contains("HEAD") == false {Some(trimmed)} else { None }
                })
            .collect()
    };
    
    let earliest_time = Utc::now().checked_sub_signed(Duration::days(args.days)).expect(format!("Failed to calculate a date {} back", args.days).as_str());
    
    // Find the time of the last commit on each branch and the author of said commit
    let branch_vec = remote_branches.into_par_iter().filter_map(|branch|
        {
            let last_commit_time = get_branch_last_commit_time(git_path, branch);
            if last_commit_time < earliest_time
            {
                let author = get_author_of_last_commit(git_path, branch);
                Some((branch, author))
            }    
            else
            {
                None
            }
        });

    let mut author_to_branches = HashMap::<String, Vec<&str>>::new();
    for (branch, author) in branch_vec.collect::<Vec<(&str, String)>>().iter() 
    {
        author_to_branches.entry(author.to_string())
            .and_modify(|x| x.push(branch))
            .or_insert(vec![branch]);
    }
    
    if author_to_branches.len() == 0
    {
        println!("No Branches were found older than {} days with the given criteria.", args.days);
        return;
    }
    
    
    // Create our report
    let total_branches_found : usize = author_to_branches.iter().map(|(_, val)| val.len() ).sum();
    
    let stdout = io::stdout(); 
    let mut handle = io::BufWriter::new(stdout); 
    writeln!(handle, "Found a total of {} branches older than {} days with the given criteria\n", total_branches_found, args.days).expect("Failed to write to stdout");
    
    let mut v: Vec<(&String, &Vec<&str>)> = author_to_branches.iter().collect();
    v.par_sort_by(|a, b| b.1.len().cmp(&a.1.len()));
    let name_width = v.par_iter().map(|x| x.0.len()).max().expect("Failed to find the author name with the highest length to format output");
    
    for (k, v) in v
    {
        writeln!(handle, "{:name_width$}\t{} old branches\t{}", k, v.len(), v.join(", "))
            .expect("Failed to write to stdout");
    }
    handle.flush().expect("Failed to flush to stdout");
}

fn get_remote_branches(git_path: &str, target_branch: &str, mode: Mode) -> String
{
    let mode_str = match mode 
    {
        Mode::Merged => "--merged",
        Mode::NoMerge => "--no-merge"
    };
    
    let stdout = process::Command::new(git_path)
        .arg("branch")
        .arg(mode_str)
        .arg(target_branch)
        .arg("-r")
        .output().expect("Failed to get the remote branches")
        .stdout;

    return String::from_utf8_lossy(&stdout).to_string();
}

fn get_branch_last_commit_time(git_path: &str, branch: &str) -> DateTime<Utc>
{
    let mut process = process::Command::new(git_path)
        .arg("show")
        .arg("--format=\"%ci\"")
        .arg(branch)
        .stdout(Stdio::piped())
        .spawn().expect("Failed to spawn git command to find last commit time");
    
    let show_output = process.stdout.take().expect("Git show provided no output");
    
    let head_output = process::Command::new("head")
        .arg("-n")
        .arg("1")
        .stdin(show_output)
        .output().expect(format!("Failed to get last commit time for branch {}", branch).as_str()).stdout;

    let datetime = String::from_utf8_lossy(&head_output).to_string();
    let len = datetime.trim().len();
    
    let datetime = DateTime::parse_from_str(&datetime[1..len-1], "%Y-%m-%d %H:%M:%S %z").expect(format!("Failed to parse datetime {}", &datetime[1..len-1]).as_str());
    datetime.with_timezone(&Utc)
}

fn get_author_of_last_commit(git_path: &str, branch: &str) -> String
{
    let stdout = process::Command::new(git_path)
        .arg("log")
        .arg("-1")
        .arg("--pretty=format:%an")
        .arg(branch)
        .output().expect(format!("Failed to get the author of the last commit on branch: {}", branch).as_str())
        .stdout;
    String::from_utf8_lossy(&stdout).to_string()
}
use std::process;
use std::collections::HashMap;
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
    let args = CliInputs::parse();

    let mode_str = match args.mode
    {
        Mode::Merged => "--merged",
        Mode::NoMerge => "--no-merge"
    };
    
    let stdout = process::Command::new("git")
        .arg("for-each-ref")
        .arg("--format=%(refname:short)//%(authordate:iso)//%(authorname)")
        .arg("refs/remotes")
        .arg(mode_str)
        .arg(args.branch)
        .output().expect("Failed to run git for-each-ref")
        .stdout;
    let output = String::from_utf8_lossy(&stdout).to_string();
    let earliest_time = Utc::now().checked_sub_signed(Duration::days(args.days)).expect(format!("Failed to calculate a date {} back", args.days).as_str());

    let exclude_start_with = args.exclude;
    
    let commits = output.par_lines()
        .filter_map(|x|
        {
            let mut split = x.split("//");
            
            let branch = split.next().expect("Failed to split branch name from for-each-ref output");
            
            let excluded = if exclude_start_with.as_ref().is_some()
            {
                exclude_start_with.as_ref().unwrap().iter().any(|x| branch.starts_with(x))
            }
            else { false };
            
            if excluded == false
            {
                let datetime = split.next().expect("Failed to split datetime from for-each-ref output");
                let datetime = DateTime::parse_from_str(&datetime, "%Y-%m-%d %H:%M:%S %z")
                    .expect(format!("Failed to parse datetime {}", &datetime).as_str());
                let datetime = datetime.with_timezone(&Utc);

                if datetime < earliest_time
                {
                    let author = split.next().expect("Failed to split author name from for-each-ref output");
                    Some((branch, author))    
                }
                else 
                { 
                    None
                }
            }
            else 
            {
                None
            }
        });
    
    
    let mut author_to_branches = HashMap::<String, Vec<&str>>::new();
    for (branch, author) in commits.collect::<Vec<(&str, &str)>>().iter()
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
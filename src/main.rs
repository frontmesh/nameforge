use std::path::PathBuf;
use std::time::{Duration, Instant};
use clap::{Parser, Subcommand};
use colored::*;


#[derive(Parser, Debug)]
#[command(name = "nameforge", author, version, about = "Rename images by context", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to the input file (for default processing)
    #[arg(short, long, global = true)]
    input: Option<PathBuf>,

    /// Perform a dry run without making changes
    #[arg(short,long, default_value_t = false, global = true)]
    dry_run: bool,

    /// Organize photos into date-based folders (YYYY-MM-DD format)
    #[arg(short, long, default_value_t = false, global = true)]
    organize_by_date: bool,

    /// Enable AI content analysis
    #[arg(long, default_value_t = false, global = true)]
    ai_content: bool,

    /// AI model to use for content analysis
    #[arg(long, default_value = "llava-llama3:latest", global = true)]
    ai_model: String,

    /// Maximum characters for AI-generated filename
    #[arg(long, default_value_t = 20, global = true)]
    ai_max_chars: u32,

    /// Case format for AI-generated filename (snake_case, lowercase, uppercase, camelcase)
    #[arg(long, default_value = "snake_case", global = true)]
    ai_case: String,

    /// Language for AI-generated filename
    #[arg(long, default_value = "English", global = true)]
    ai_language: String,

    /// Use full timestamp (YYYY-MM-DD_HH-MM-SS) instead of date only
    #[arg(long, default_value_t = false, global = true)]
    full_timestamp: bool,

    /// Use file system date instead of EXIF date for filename
    #[arg(short = 'f', long, default_value_t = false, global = true)]
    use_file_date: bool,

    /// When using file date (-f), prefer modified time over creation time
    #[arg(short = 'M', long, default_value_t = false, global = true)]
    prefer_modified: bool,

    /// Skip date prefix in filename (use only AI-generated name)
    #[arg(short = 'n', long, default_value_t = false, global = true)]
    no_date: bool,
}

fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let millis = duration.subsec_millis();
    
    if total_secs >= 60 {
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        format!("{}m {}s", mins, secs)
    } else if total_secs > 0 {
        format!("{}.{:03}s", total_secs, millis)
    } else {
        format!("{}ms", millis)
    }
}

fn display_completion_time(start_time: Instant) {
    let duration = start_time.elapsed();
    println!("{}", "─".repeat(50).bright_black());
    println!("{}  {}{}", "⏱️".bright_cyan(), "Completed in: ".bright_cyan(), format_duration(duration).bright_white().bold());
    println!();
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Process images with AI content analysis (limited number of images)
    Prompt {
        /// Path to the input folder
        #[arg(short, long)]
        input: PathBuf,

        /// Maximum number of images to process (optional, processes all if not specified)
        #[arg(short = 'm', long)]
        max_images: Option<usize>,
    },
}

fn main() {
    let start_time = Instant::now();
    let args = Args::parse();

    match &args.command {
        Some(Commands::Prompt { input, max_images }) => {
            // For prompt command, force AI content analysis
            display_prompt_config(&args, input, *max_images);
            nameforge::process_folder(
                input,
                args.dry_run,
                args.organize_by_date,
                true, // Force AI content analysis for prompt command
                &args.ai_model,
                args.ai_max_chars,
                &args.ai_case,
                &args.ai_language,
                !args.full_timestamp,
                *max_images, // Pass the optional max_images limit
                args.use_file_date,
                args.prefer_modified,
                args.no_date,
            );
            
            display_completion_time(start_time);
        }
        None => {
            // Default processing - require input argument
            let input = args.input.as_ref().expect("Input path is required for default processing. Use --input or run 'nf prompt --input <path> --max-images <n>'");
            display_config(&args, input);
            nameforge::process_folder(
                input,
                args.dry_run,
                args.organize_by_date,
                args.ai_content,
                &args.ai_model,
                args.ai_max_chars,
                &args.ai_case,
                &args.ai_language,
                !args.full_timestamp,
                None, // No limit for default processing
                args.use_file_date,
                args.prefer_modified,
                args.no_date,
            );
            
            display_completion_time(start_time);
        }
    }
}

fn display_config(args: &Args, input: &std::path::Path) {
    println!("{}", "📸 NameForge Configuration".bright_cyan().bold());
    println!("{}", "─".repeat(50).bright_black());
    
    // Input settings
    println!("{}  {}", "📁 Input folder:".bright_green(), input.display().to_string().bright_white());
    println!("{}   {}", "🔧 Mode:".bright_green(), 
        if args.dry_run { "DRY RUN".bright_yellow().bold() } else { "LIVE".bright_red().bold() });
    
    // Organization settings
    println!("{}  {}", "📅 Date folders:".bright_green(), 
        if args.organize_by_date { "ENABLED".bright_green() } else { "DISABLED".bright_red() });
    println!("{}    {}", "📆 Date format:".bright_green(), 
        if args.full_timestamp { "FULL TIMESTAMP (YYYY-MM-DD_HH-MM-SS)".bright_cyan() } else { "DATE ONLY (YYYY-MM-DD)".bright_cyan().bold() });
    if args.no_date {
        println!("{}   {}", "📅 Date source:".bright_green(), "DISABLED".bright_red().bold());
    } else {
        println!("{}   {}", "📅 Date source:".bright_green(), 
            if args.use_file_date { 
                if args.prefer_modified { "FILE MODIFIED".bright_green().bold() } else { "FILE CREATION".bright_green().bold() }
            } else { "EXIF DATA".bright_cyan().bold() });
    }
    
    // AI settings
    if args.ai_content {
        println!("{}      {}", "🤖 AI Analysis:".bright_green(), "ENABLED".bright_green().bold());
        println!("{}        {}", "   Model:".bright_blue(), args.ai_model.bright_white());
        println!("{}    {}", "   Max chars:".bright_blue(), args.ai_max_chars.to_string().bright_white());
        println!("{}         {}", "   Case:".bright_blue(), args.ai_case.bright_white());
        println!("{}     {}", "   Language:".bright_blue(), args.ai_language.bright_white());
    } else {
        println!("{}      {}", "🤖 AI Analysis:".bright_green(), "DISABLED".bright_red());
        println!("{}     {}", "   Using GPS location data instead".bright_black(), "");
    }
    
    println!("{}", "─".repeat(50).bright_black());
    println!();
}

fn display_prompt_config(args: &Args, input: &std::path::Path, max_images: Option<usize>) {
    println!("{}", "🤖 NameForge - Prompt Mode".bright_magenta().bold());
    println!("{}", "─".repeat(50).bright_black());
    
    // Input settings
    println!("{}  {}", "📁 Input folder:".bright_green(), input.display().to_string().bright_white());
    println!("{}   {}", "🔧 Mode:".bright_green(), 
        if args.dry_run { "DRY RUN".bright_yellow().bold() } else { "LIVE".bright_red().bold() });
    println!("{}  {}", "🎯 Max images:".bright_green(), 
        match max_images {
            Some(n) => n.to_string().bright_cyan().bold(),
            None => "ALL".bright_green().bold()
        });
    
    // Organization settings
    println!("{}  {}", "📅 Date folders:".bright_green(), 
        if args.organize_by_date { "ENABLED".bright_green() } else { "DISABLED".bright_red() });
    println!("{}    {}", "📆 Date format:".bright_green(), 
        if args.full_timestamp { "FULL TIMESTAMP (YYYY-MM-DD_HH-MM-SS)".bright_cyan() } else { "DATE ONLY (YYYY-MM-DD)".bright_cyan().bold() });
    if args.no_date {
        println!("{}   {}", "📅 Date source:".bright_green(), "DISABLED".bright_red().bold());
    } else {
        println!("{}   {}", "📅 Date source:".bright_green(), 
            if args.use_file_date { 
                if args.prefer_modified { "FILE MODIFIED".bright_green().bold() } else { "FILE CREATION".bright_green().bold() }
            } else { "EXIF DATA".bright_cyan().bold() });
    }
    
    // AI settings (always enabled for prompt mode)
    println!("{} {}", "🤖 AI Analysis:".bright_green(), "ENABLED".bright_green().bold());
    println!("{} {}", "   Model:".bright_blue(), args.ai_model.bright_white());
    println!("{} {}", "   Max chars:".bright_blue(), args.ai_max_chars.to_string().bright_white());
    println!("{} {}", "   Case:".bright_blue(), args.ai_case.bright_white());
    println!("{} {}", "   Language:".bright_blue(), args.ai_language.bright_white());
    
    println!("{}", "─".repeat(50).bright_black());
    println!();
}

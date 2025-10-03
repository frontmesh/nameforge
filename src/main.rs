use std::path::PathBuf;
use clap::Parser;
use colored::*;


#[derive(Parser, Debug)]
#[command(name = "nameforge", author, version, about = "Rename images by context", long_about = None)]
struct Args {
    /// Path to the input file
    #[arg(short, long)]
    input: PathBuf,

    /// Perform a dry run without making changes
    #[arg(short,long, default_value_t = false)]
    dry_run: bool,

    /// Organize photos into date-based folders (YYYY-MM-DD format)
    #[arg(short, long, default_value_t = false)]
    organize_by_date: bool,

    /// Enable AI content analysis using Ollama
    #[arg(long, default_value_t = false)]
    ai_content: bool,

    /// Ollama model to use for content analysis
    #[arg(long, default_value = "llava:13b")]
    ai_model: String,

    /// Maximum characters for AI-generated filename
    #[arg(long, default_value_t = 20)]
    ai_max_chars: u32,

    /// Case format for AI-generated filename (lowercase, uppercase, camelcase)
    #[arg(long, default_value = "lowercase")]
    ai_case: String,

    /// Language for AI-generated filename
    #[arg(long, default_value = "English")]
    ai_language: String,
}

fn main() {
    let args = Args::parse();

    // Display current configuration
    display_config(&args);

    nameforge::process_folder(
        &args.input, 
        args.dry_run, 
        args.organize_by_date,
        args.ai_content,
        &args.ai_model,
        args.ai_max_chars,
        &args.ai_case,
        &args.ai_language,
    );
}

fn display_config(args: &Args) {
    println!("{}", "📸 NameForge Configuration".bright_cyan().bold());
    println!("{}", "─".repeat(50).bright_black());
    
    // Input settings
    println!("{}  {}", "📁 Input folder:".bright_green(), args.input.display().to_string().bright_white());
    println!("{}   {}", "🔧 Mode:".bright_green(), 
        if args.dry_run { "DRY RUN".bright_yellow().bold() } else { "LIVE".bright_red().bold() });
    
    // Organization settings
    println!("{}  {}", "📅 Date folders:".bright_green(), 
        if args.organize_by_date { "ENABLED".bright_green() } else { "DISABLED".bright_red() });
    
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

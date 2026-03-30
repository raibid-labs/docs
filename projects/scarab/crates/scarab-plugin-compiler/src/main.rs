use anyhow::{Context, Result, bail};
use clap::Parser;
use colored::*;
use std::fs;
use std::path::{Path, PathBuf};

use fusabi_frontend::{Lexer, Parser as FusabiParser, TypeInference, Compiler, TypeEnv};
use fusabi_vm::{Chunk, FZB_MAGIC, FZB_VERSION};

/// Scarab Fusabi Plugin Compiler
///
/// Compiles .fsx Fusabi source files to .fzb bytecode files
#[derive(Parser, Debug)]
#[command(
    name = "scarab-plugin-compiler",
    version,
    about,
    long_about = None
)]
struct Args {
    /// Input .fsx source file
    #[arg(value_name = "INPUT")]
    input: PathBuf,

    /// Output .fzb bytecode file (default: same name as input with .fzb extension)
    #[arg(short, long, value_name = "OUTPUT")]
    output: Option<PathBuf>,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Skip type checking (compile without type inference)
    #[arg(long)]
    skip_type_check: bool,

    /// Validate plugin metadata
    #[arg(long)]
    validate_metadata: bool,

    /// Print AST for debugging
    #[arg(long)]
    print_ast: bool,

    /// Print bytecode disassembly
    #[arg(long)]
    disassemble: bool,
}

/// Plugin metadata extracted from source comments
#[derive(Debug, Default)]
struct PluginMetadata {
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    author: Option<String>,
    api_version: Option<String>,
    min_scarab_version: Option<String>,
}

impl PluginMetadata {
    /// Extract metadata from source code comments
    fn parse_from_source(source: &str) -> Self {
        let mut metadata = PluginMetadata::default();

        for line in source.lines().take(50) {
            let trimmed = line.trim();
            if !trimmed.starts_with("//") {
                continue;
            }

            let content = trimmed.trim_start_matches("//").trim();

            if let Some(value) = content.strip_prefix("@name") {
                metadata.name = Some(value.trim().to_string());
            } else if let Some(value) = content.strip_prefix("@version") {
                metadata.version = Some(value.trim().to_string());
            } else if let Some(value) = content.strip_prefix("@description") {
                metadata.description = Some(value.trim().to_string());
            } else if let Some(value) = content.strip_prefix("@author") {
                metadata.author = Some(value.trim().to_string());
            } else if let Some(value) = content.strip_prefix("@api-version") {
                metadata.api_version = Some(value.trim().to_string());
            } else if let Some(value) = content.strip_prefix("@min-scarab-version") {
                metadata.min_scarab_version = Some(value.trim().to_string());
            }
        }

        metadata
    }

    /// Validate that required metadata is present
    fn validate(&self) -> Result<()> {
        let mut errors = Vec::new();

        if self.name.is_none() {
            errors.push("Missing @name metadata");
        }
        if self.version.is_none() {
            errors.push("Missing @version metadata");
        }
        if self.description.is_none() {
            errors.push("Missing @description metadata");
        }

        if !errors.is_empty() {
            bail!("Plugin metadata validation failed:\n  {}", errors.join("\n  "));
        }

        Ok(())
    }

    /// Print metadata summary
    fn print_summary(&self) {
        println!("{}", "Plugin Metadata:".bold());
        if let Some(name) = &self.name {
            println!("  {}: {}", "Name".cyan(), name);
        }
        if let Some(version) = &self.version {
            println!("  {}: {}", "Version".cyan(), version);
        }
        if let Some(desc) = &self.description {
            println!("  {}: {}", "Description".cyan(), desc);
        }
        if let Some(author) = &self.author {
            println!("  {}: {}", "Author".cyan(), author);
        }
        if let Some(api_version) = &self.api_version {
            println!("  {}: {}", "API Version".cyan(), api_version);
        }
        if let Some(min_version) = &self.min_scarab_version {
            println!("  {}: {}", "Min Scarab Version".cyan(), min_version);
        }
    }
}

/// Bytecode file header for .fzb files
#[derive(serde::Serialize, serde::Deserialize)]
struct BytecodeFileHeader {
    magic: Vec<u8>,
    version: u8,
    metadata: SerializedMetadata,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SerializedMetadata {
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    author: Option<String>,
}

impl From<&PluginMetadata> for SerializedMetadata {
    fn from(meta: &PluginMetadata) -> Self {
        SerializedMetadata {
            name: meta.name.clone(),
            version: meta.version.clone(),
            description: meta.description.clone(),
            author: meta.author.clone(),
        }
    }
}

/// Compile a Fusabi source file to bytecode
fn compile_plugin(
    source_path: &Path,
    output_path: &Path,
    args: &Args,
) -> Result<()> {
    // Read source file
    let source = fs::read_to_string(source_path)
        .with_context(|| format!("Failed to read source file: {}", source_path.display()))?;

    if args.verbose {
        println!("{} Reading source from: {}", "[1/6]".dimmed(), source_path.display());
    }

    // Extract and validate metadata
    let metadata = PluginMetadata::parse_from_source(&source);

    if args.verbose {
        println!("{} Extracting metadata...", "[2/6]".dimmed());
        metadata.print_summary();
    }

    if args.validate_metadata {
        metadata.validate()
            .context("Plugin metadata validation failed")?;
        println!("{} Metadata validation passed", "✓".green().bold());
    }

    // Tokenize
    if args.verbose {
        println!("{} Tokenizing source...", "[3/6]".dimmed());
    }

    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize()
        .map_err(|e| anyhow::anyhow!("Lexer error: {:?}", e))?;

    if args.verbose {
        println!("  {} tokens generated", tokens.len());
    }

    // Parse to AST
    if args.verbose {
        println!("{} Parsing to AST...", "[4/6]".dimmed());
    }

    let mut parser = FusabiParser::new(tokens);
    let ast = parser.parse()
        .map_err(|e| anyhow::anyhow!("Parser error: {:?}", e))?;

    if args.print_ast {
        println!("{}", "Abstract Syntax Tree:".bold());
        println!("{:#?}", ast);
    }

    // Type checking (optional)
    if !args.skip_type_check {
        if args.verbose {
            println!("{} Type checking...", "[5/6]".dimmed());
        }

        let mut type_inference = TypeInference::new();
        let env = TypeEnv::new();

        type_inference.infer_and_solve(&ast, &env)
            .map_err(|e| anyhow::anyhow!("Type inference error: {:?}", e))?;

        if args.verbose {
            println!("  {} Type checking passed", "✓".green());
        }
    } else if args.verbose {
        println!("{} Skipping type checking", "[5/6]".dimmed().yellow());
    }

    // Compile to bytecode
    if args.verbose {
        println!("{} Compiling to bytecode...", "[6/6]".dimmed());
    }

    let chunk = Compiler::compile(&ast)
        .map_err(|e| anyhow::anyhow!("Compilation error: {:?}", e))?;

    if args.verbose {
        println!("  {} bytecode instructions", chunk.instructions.len());
    }

    if args.disassemble {
        println!("{}", "Bytecode Disassembly:".bold());
        chunk.disassemble();
    }

    // Write bytecode to file
    write_bytecode_file(output_path, &chunk, &metadata, args.verbose)?;

    println!(
        "{} Compiled successfully: {}",
        "✓".green().bold(),
        output_path.display()
    );

    Ok(())
}

/// Write bytecode chunk to .fzb file with header
fn write_bytecode_file(
    path: &Path,
    chunk: &Chunk,
    metadata: &PluginMetadata,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("  Writing bytecode to: {}", path.display());
    }

    // Create output directory if needed
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create output directory: {}", parent.display()))?;
    }

    // Prepare file header
    let header = BytecodeFileHeader {
        magic: FZB_MAGIC.to_vec(),
        version: FZB_VERSION,
        metadata: metadata.into(),
    };

    // Serialize header
    let header_bytes = bincode::serialize(&header)
        .context("Failed to serialize bytecode header")?;

    // Serialize chunk
    let chunk_bytes = bincode::serialize(chunk)
        .context("Failed to serialize bytecode chunk")?;

    // Combine header + chunk
    let mut file_data = Vec::new();
    file_data.extend_from_slice(&(header_bytes.len() as u32).to_le_bytes());
    file_data.extend_from_slice(&header_bytes);
    file_data.extend_from_slice(&chunk_bytes);

    // Write to file
    fs::write(path, &file_data)
        .with_context(|| format!("Failed to write bytecode file: {}", path.display()))?;

    if verbose {
        println!("  {} bytes written", file_data.len());
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Print banner
    if args.verbose {
        println!("{}", "Scarab Fusabi Plugin Compiler".bold().cyan());
        println!("{}", "=".repeat(50).dimmed());
    }

    // Validate input file
    if !args.input.exists() {
        bail!("Input file does not exist: {}", args.input.display());
    }

    if args.input.extension().and_then(|s| s.to_str()) != Some("fsx") {
        eprintln!(
            "{} Input file should have .fsx extension: {}",
            "Warning:".yellow().bold(),
            args.input.display()
        );
    }

    // Determine output path
    let output_path = args.output.clone().unwrap_or_else(|| {
        let mut path = args.input.clone();
        path.set_extension("fzb");
        path
    });

    // Compile the plugin
    compile_plugin(&args.input, &output_path, &args)?;

    Ok(())
}

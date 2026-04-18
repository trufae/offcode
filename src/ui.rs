// ANSI color constants
pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";
pub const CYAN: &str = "\x1b[36m";
pub const YELLOW: &str = "\x1b[33m";
pub const RED: &str = "\x1b[31m";
pub const WHITE: &str = "\x1b[37m";
pub const BRIGHT_GREEN: &str = "\x1b[92m";
pub const BRIGHT_CYAN: &str = "\x1b[96m";
pub const BRIGHT_YELLOW: &str = "\x1b[93m";

pub fn print_mascot(model: &str) {
    let version = env!("CARGO_PKG_VERSION");
    let c = BRIGHT_CYAN;
    let b = BOLD;
    let r = RESET;
    let d = DIM;

    println!();
    println!("{c}{b}    ╔═══╗ ╔═══╗{r}");
    println!("{c}{b}    ║ ◉ ║ ║ ◉ ║{r}   {b}offcode{r} {c}v{version}{r}");
    println!("{c}{b}    ╚═══╝ ╚═══╝{r}   {d}offline coding assistant{r}");
    println!("{c}{b}      ╔═════╗{r}     {d}model : {model}{r}");
    println!("{c}{b}      ║ ~~~ ║{r}     {d}vendor: ollama (local){r}");
    println!("{c}{b}      ╚══╤══╝{r}");
    println!("{c}{b}      ╔══╧══╗{r}     {d}type /help for commands{r}");
    println!("{c}{b}      ║     ║{r}");
    println!("{c}{b}      ╚═════╝{r}");
    println!();
}


//! Command-line adapter for Aero1394.

#![forbid(unsafe_code)]

use aero1394::forensic::{
    FileOffset, Hexdump, HexdumpConfig, HexdumpConfigError, HexdumpError, HexdumpLine,
    MAX_BYTES_PER_LINE,
};
use std::env;
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::fs::File;
use std::io::{self, BufWriter, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::process::ExitCode;

const DEFAULT_LENGTH: u64 = 256;
const DEFAULT_WIDTH: usize = 16;

const GENERAL_HELP: &str = "\
Aero1394 forensic capture inspection

Usage:
  aero1394 <COMMAND>

Commands:
  hexdump    Display uninterpreted bytes with absolute file offsets

Options:
  -h, --help       Show this help
  -V, --version    Show the program version

Run 'aero1394 hexdump --help' for command options.
";

const HEXDUMP_HELP: &str = "\
Display uninterpreted file bytes with absolute offsets

Usage:
  aero1394 hexdump <FILE> [OPTIONS]

Arguments:
  <FILE>                 Capture file to inspect

Options:
  -o, --offset <BYTES>   Absolute starting offset [default: 0]
  -n, --length <BYTES>   Bytes to display, or 'all' [default: 256]
  -w, --width <BYTES>    Bytes per output line, from 1 through 256 [default: 16]
  -h, --help             Show this help

Byte counts accept decimal or a 0x-prefixed hexadecimal value. Underscores are
allowed as digit separators. Use '--length all' deliberately for an entire
file; output is otherwise bounded to 256 bytes by default.
";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ByteLimit {
    Bounded(u64),
    All,
}

impl ByteLimit {
    const fn as_option(self) -> Option<u64> {
        match self {
            Self::Bounded(value) => Some(value),
            Self::All => None,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct HexdumpArgs {
    path: PathBuf,
    offset: u64,
    length: ByteLimit,
    width: usize,
}

#[derive(Debug)]
enum AppError {
    Usage(String),
    Operation(String),
    Io { context: String, source: io::Error },
    Hexdump(HexdumpError),
}

impl AppError {
    fn io(context: impl Into<String>, source: io::Error) -> Self {
        Self::Io {
            context: context.into(),
            source,
        }
    }

    fn is_broken_pipe(&self) -> bool {
        matches!(
            self,
            Self::Io {
                source,
                ..
            } if source.kind() == io::ErrorKind::BrokenPipe
        )
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) | Self::Operation(message) => formatter.write_str(message),
            Self::Io { context, source } => write!(formatter, "{context}: {source}"),
            Self::Hexdump(error) => error.fmt(formatter),
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Hexdump(error) => Some(error),
            Self::Usage(_) | Self::Operation(_) => None,
        }
    }
}

impl From<HexdumpError> for AppError {
    fn from(error: HexdumpError) -> Self {
        Self::Hexdump(error)
    }
}

impl From<HexdumpConfigError> for AppError {
    fn from(error: HexdumpConfigError) -> Self {
        Self::Usage(error.to_string())
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) if error.is_broken_pipe() => ExitCode::SUCCESS,
        Err(AppError::Usage(message)) => {
            eprintln!("error: {message}\n\nTry 'aero1394 hexdump --help' for usage.");
            ExitCode::from(2)
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), AppError> {
    let mut arguments = env::args_os().skip(1);
    let Some(command) = arguments.next() else {
        return write_stdout(GENERAL_HELP);
    };

    if command == "-h" || command == "--help" {
        return write_stdout(GENERAL_HELP);
    }
    if command == "-V" || command == "--version" {
        return write_stdout(&format!("aero1394 {}\n", env!("CARGO_PKG_VERSION")));
    }
    if command != "hexdump" {
        return Err(AppError::Usage(format!(
            "unknown command '{}'",
            command.to_string_lossy()
        )));
    }

    let Some(arguments) = parse_hexdump_args(arguments)? else {
        return write_stdout(HEXDUMP_HELP);
    };
    execute_hexdump(&arguments)
}

fn parse_hexdump_args(
    arguments: impl IntoIterator<Item = OsString>,
) -> Result<Option<HexdumpArgs>, AppError> {
    let mut arguments = arguments.into_iter();
    let mut path = None;
    let mut offset = None;
    let mut length = None;
    let mut width = None;
    let mut positional_only = false;

    while let Some(argument) = arguments.next() {
        if !positional_only && (argument == "-h" || argument == "--help") {
            return Ok(None);
        }
        if !positional_only && argument == "--" {
            positional_only = true;
            continue;
        }

        if !positional_only && (argument == "-o" || argument == "--offset") {
            ensure_not_set(offset.is_some(), "--offset")?;
            let value = require_option_value(&mut arguments, "--offset")?;
            offset = Some(parse_byte_count(&value, "--offset")?);
            continue;
        }
        if !positional_only && (argument == "-n" || argument == "--length") {
            ensure_not_set(length.is_some(), "--length")?;
            let value = require_option_value(&mut arguments, "--length")?;
            length = Some(parse_byte_limit(&value)?);
            continue;
        }
        if !positional_only && (argument == "-w" || argument == "--width") {
            ensure_not_set(width.is_some(), "--width")?;
            let value = require_option_value(&mut arguments, "--width")?;
            width = Some(parse_width(&value)?);
            continue;
        }

        if !positional_only {
            if let Some(value) = option_assignment(&argument, "--offset") {
                ensure_not_set(offset.is_some(), "--offset")?;
                offset = Some(parse_byte_count(value, "--offset")?);
                continue;
            }
            if let Some(value) = option_assignment(&argument, "--length") {
                ensure_not_set(length.is_some(), "--length")?;
                length = Some(parse_byte_limit(value)?);
                continue;
            }
            if let Some(value) = option_assignment(&argument, "--width") {
                ensure_not_set(width.is_some(), "--width")?;
                width = Some(parse_width(value)?);
                continue;
            }

            if argument.to_string_lossy().starts_with('-') {
                return Err(AppError::Usage(format!(
                    "unknown hexdump option '{}'",
                    argument.to_string_lossy()
                )));
            }
        }

        if path.replace(PathBuf::from(&argument)).is_some() {
            return Err(AppError::Usage(format!(
                "unexpected extra file argument '{}'",
                argument.to_string_lossy()
            )));
        }
    }

    let path = path.ok_or_else(|| AppError::Usage("missing required <FILE> argument".into()))?;

    Ok(Some(HexdumpArgs {
        path,
        offset: offset.unwrap_or(0),
        length: length.unwrap_or(ByteLimit::Bounded(DEFAULT_LENGTH)),
        width: width.unwrap_or(DEFAULT_WIDTH),
    }))
}

fn ensure_not_set(is_set: bool, option: &str) -> Result<(), AppError> {
    if is_set {
        Err(AppError::Usage(format!(
            "option '{option}' may only be specified once"
        )))
    } else {
        Ok(())
    }
}

fn require_option_value(
    arguments: &mut impl Iterator<Item = OsString>,
    option: &str,
) -> Result<OsString, AppError> {
    arguments
        .next()
        .ok_or_else(|| AppError::Usage(format!("option '{option}' requires a value")))
}

fn option_assignment<'a>(argument: &'a OsStr, option: &str) -> Option<&'a OsStr> {
    let argument = argument.to_str()?;
    argument
        .strip_prefix(option)?
        .strip_prefix('=')
        .map(OsStr::new)
}

fn parse_byte_count(value: &OsStr, option: &str) -> Result<u64, AppError> {
    let value = value
        .to_str()
        .ok_or_else(|| AppError::Usage(format!("{option} requires a Unicode number")))?;
    let normalized = value.replace('_', "");
    if normalized.is_empty() {
        return Err(AppError::Usage(format!("{option} requires a byte count")));
    }

    let result = if let Some(hexadecimal) = normalized
        .strip_prefix("0x")
        .or_else(|| normalized.strip_prefix("0X"))
    {
        if hexadecimal.is_empty() {
            None
        } else {
            u64::from_str_radix(hexadecimal, 16).ok()
        }
    } else {
        normalized.parse::<u64>().ok()
    };

    result.ok_or_else(|| {
        AppError::Usage(format!(
            "invalid value '{value}' for {option}; use decimal or 0x-prefixed hexadecimal"
        ))
    })
}

fn parse_byte_limit(value: &OsStr) -> Result<ByteLimit, AppError> {
    if value
        .to_str()
        .is_some_and(|value| value.eq_ignore_ascii_case("all"))
    {
        Ok(ByteLimit::All)
    } else {
        parse_byte_count(value, "--length").map(ByteLimit::Bounded)
    }
}

fn parse_width(value: &OsStr) -> Result<usize, AppError> {
    let value = parse_byte_count(value, "--width")?;
    let width = usize::try_from(value)
        .map_err(|_| AppError::Usage(format!("--width must not exceed {MAX_BYTES_PER_LINE}")))?;
    HexdumpConfig::new(width, Some(0))?;
    Ok(width)
}

fn execute_hexdump(arguments: &HexdumpArgs) -> Result<(), AppError> {
    let mut file = File::open(&arguments.path).map_err(|error| {
        AppError::io(
            format!("could not open '{}'", arguments.path.display()),
            error,
        )
    })?;
    let file_size = file
        .metadata()
        .map_err(|error| AppError::io("could not read input metadata", error))?
        .len();

    if arguments.offset > file_size {
        return Err(AppError::Operation(format!(
            "offset {} (0x{:x}) is beyond the {}-byte file",
            arguments.offset, arguments.offset, file_size
        )));
    }

    file.seek(SeekFrom::Start(arguments.offset))
        .map_err(|error| AppError::io("could not seek to the requested offset", error))?;

    let config = HexdumpConfig::new(arguments.width, arguments.length.as_option())?;
    let dump = Hexdump::new(file, FileOffset::new(arguments.offset), config);
    let stdout = io::stdout();
    let mut output = BufWriter::new(stdout.lock());

    for line in dump {
        let line = line?;
        render_line(&mut output, &line, arguments.width)
            .map_err(|error| AppError::io("could not write hex dump", error))?;
    }
    output
        .flush()
        .map_err(|error| AppError::io("could not flush hex dump", error))
}

fn render_line(
    output: &mut impl Write,
    line: &HexdumpLine,
    bytes_per_line: usize,
) -> io::Result<()> {
    write!(output, "{:016x}  ", line.offset().get())?;

    for index in 0..bytes_per_line {
        if index > 0 {
            output.write_all(b" ")?;
        }
        if index > 0 && index % 8 == 0 {
            output.write_all(b" ")?;
        }
        if let Some(byte) = line.bytes().get(index) {
            write!(output, "{byte:02X}")?;
        } else {
            output.write_all(b"  ")?;
        }
    }

    output.write_all(b"  |")?;
    for byte in line.bytes() {
        let character = if (0x20..=0x7e).contains(byte) {
            char::from(*byte)
        } else {
            '.'
        };
        write!(output, "{character}")?;
    }
    output.write_all(b"|\n")
}

fn write_stdout(text: &str) -> Result<(), AppError> {
    let stdout = io::stdout();
    let mut output = stdout.lock();
    output
        .write_all(text.as_bytes())
        .map_err(|error| AppError::io("could not write output", error))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn os(value: &str) -> OsString {
        OsString::from(value)
    }

    #[test]
    fn parses_hexadecimal_offsets_and_explicit_unbounded_length() {
        let arguments = parse_hexdump_args([
            os("capture.bie"),
            os("--offset"),
            os("0x1_000"),
            os("--length=all"),
            os("--width"),
            os("32"),
        ])
        .expect("arguments are valid")
        .expect("help was not requested");

        assert_eq!(
            arguments,
            HexdumpArgs {
                path: PathBuf::from("capture.bie"),
                offset: 0x1000,
                length: ByteLimit::All,
                width: 32,
            }
        );
    }

    #[test]
    fn defaults_to_a_bounded_dump() {
        let arguments = parse_hexdump_args([os("capture.bie")])
            .expect("arguments are valid")
            .expect("help was not requested");

        assert_eq!(arguments.offset, 0);
        assert_eq!(arguments.length, ByteLimit::Bounded(256));
        assert_eq!(arguments.width, 16);
    }

    #[test]
    fn rejects_duplicate_options() {
        let error = parse_hexdump_args([os("capture.bie"), os("--offset=1"), os("--offset=2")])
            .expect_err("duplicate option must fail");

        assert!(error.to_string().contains("only be specified once"));
    }

    #[test]
    fn renders_absolute_offset_hex_and_ascii() {
        let config = HexdumpConfig::new(4, Some(3)).expect("valid configuration");
        let line = Hexdump::new(
            io::Cursor::new([0x00, b'A', 0xFF]),
            FileOffset::new(0x20),
            config,
        )
        .next()
        .expect("one line")
        .expect("read succeeds");
        let mut output = Vec::new();

        render_line(&mut output, &line, 4).expect("write succeeds");

        assert_eq!(
            String::from_utf8(output).expect("ASCII output"),
            "0000000000000020  00 41 FF     |.A.|\n"
        );
    }
}

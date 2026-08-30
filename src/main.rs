//! Command-line adapter for Aero1394.

#![forbid(unsafe_code)]

use aero1394::as5643::VpcValidationOutcome;
use aero1394::bie::{BieReadError, BieReadItem, BieReader, BieRecord};
use aero1394::bie_as5643::{BieAs5643MappingOutcome, map_bie_record_to_as5643};
use aero1394::forensic::{
    FileOffset, Hexdump, HexdumpConfig, HexdumpConfigError, HexdumpError, HexdumpLine,
    MAX_BYTES_PER_LINE,
};
use aero1394::payload::msfcs_storesmassdata_b::StoresMassData;
use aero1394::payload::{KnownPayload, PayloadContext, PayloadSelection, select_payload};
use std::env;
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const DEFAULT_LENGTH: u64 = 256;
const DEFAULT_WIDTH: usize = 16;
const WARNING_EXIT_CODE: u8 = 2;

const GENERAL_HELP: &str = "\
Aero1394 forensic capture inspection

Usage:
  aero1394 <COMMAND>

Commands:
  hexdump    Display uninterpreted bytes with absolute file offsets
  records    List structurally parsed BIE records
  as5643     Decode supported AS5643 envelopes from BIE records

Options:
  -h, --help       Show this help
  -V, --version    Show the program version

Run 'aero1394 <COMMAND> --help' for command options.
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

const RECORDS_HELP: &str = "\
List structurally parsed BIE records

Usage:
  aero1394 records <FILE>

Arguments:
  <FILE>                 Complete BIE file to inspect

Options:
  -h, --help             Show this help

The file must end at its four-byte zero terminator. Output preserves BIE
container values and does not imply IEEE-1394, AS5643, or payload decoding.
";

const AS5643_HELP: &str = "\
Decode supported AS5643 envelopes from BIE records

Usage:
  aero1394 as5643 <FILE>

Arguments:
  <FILE>                 Complete BIE file to inspect

Options:
  -h, --help             Show this help

The file must end at its four-byte zero terminator. Only the explicit
0x00005D04 plus 116-byte mapping is decoded. Other data-item identities and
stored-data lengths remain successful, inspectable records labeled unsupported.
Mapped application bytes are checked against the built-in payload registry;
registered payloads expose retained raw fields plus explicitly labeled provisional semantics.

Exit status:
  0    Successful decode with no warnings
  1    Usage, I/O, framing, or decoding error
  2    Successful decode with one or more payload warnings
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

#[derive(Debug, Eq, PartialEq)]
struct RecordsArgs {
    path: PathBuf,
}

#[derive(Debug, Eq, PartialEq)]
struct As5643Args {
    path: PathBuf,
}

#[derive(Debug)]
enum AppError {
    Usage(String),
    Operation(String),
    Io { context: String, source: io::Error },
    Hexdump(HexdumpError),
    Bie(BieReadError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AppOutcome {
    Success,
    Warnings { count: usize },
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
            Self::Bie(error) => error.fmt(formatter),
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Hexdump(error) => Some(error),
            Self::Bie(error) => Some(error),
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

impl From<BieReadError> for AppError {
    fn from(error: BieReadError) -> Self {
        Self::Bie(error)
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(AppOutcome::Success) => ExitCode::SUCCESS,
        Ok(AppOutcome::Warnings { count }) => {
            eprintln!("warning: decoded successfully with {count} payload warning(s)");
            ExitCode::from(WARNING_EXIT_CODE)
        }
        Err(error) if error.is_broken_pipe() => ExitCode::SUCCESS,
        Err(AppError::Usage(message)) => {
            eprintln!("error: {message}\n\nTry 'aero1394 --help' for usage.");
            ExitCode::FAILURE
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<AppOutcome, AppError> {
    let mut arguments = env::args_os().skip(1);
    let Some(command) = arguments.next() else {
        return write_stdout(GENERAL_HELP).map(|()| AppOutcome::Success);
    };

    if command == "-h" || command == "--help" {
        return write_stdout(GENERAL_HELP).map(|()| AppOutcome::Success);
    }
    if command == "-V" || command == "--version" {
        return write_stdout(&format!("aero1394 {}\n", env!("CARGO_PKG_VERSION")))
            .map(|()| AppOutcome::Success);
    }
    if command == "hexdump" {
        let Some(arguments) = parse_hexdump_args(arguments)? else {
            return write_stdout(HEXDUMP_HELP).map(|()| AppOutcome::Success);
        };
        return execute_hexdump(&arguments).map(|()| AppOutcome::Success);
    }

    if command == "records" {
        let Some(arguments) = parse_records_args(arguments)? else {
            return write_stdout(RECORDS_HELP).map(|()| AppOutcome::Success);
        };
        return execute_records(&arguments).map(|()| AppOutcome::Success);
    }

    if command == "as5643" {
        let Some(arguments) = parse_as5643_args(arguments)? else {
            return write_stdout(AS5643_HELP).map(|()| AppOutcome::Success);
        };
        return execute_as5643(&arguments);
    }

    Err(AppError::Usage(format!(
        "unknown command '{}'",
        command.to_string_lossy()
    )))
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

fn parse_records_args(
    arguments: impl IntoIterator<Item = OsString>,
) -> Result<Option<RecordsArgs>, AppError> {
    let mut path = None;
    let mut positional_only = false;

    for argument in arguments {
        if !positional_only && (argument == "-h" || argument == "--help") {
            return Ok(None);
        }
        if !positional_only && argument == "--" {
            positional_only = true;
            continue;
        }
        if !positional_only && argument.to_string_lossy().starts_with('-') {
            return Err(AppError::Usage(format!(
                "unknown records option '{}'",
                argument.to_string_lossy()
            )));
        }
        if path.replace(PathBuf::from(&argument)).is_some() {
            return Err(AppError::Usage(format!(
                "unexpected extra file argument '{}'",
                argument.to_string_lossy()
            )));
        }
    }

    let path = path.ok_or_else(|| AppError::Usage("missing required <FILE> argument".into()))?;
    Ok(Some(RecordsArgs { path }))
}

fn parse_as5643_args(
    arguments: impl IntoIterator<Item = OsString>,
) -> Result<Option<As5643Args>, AppError> {
    let mut path = None;
    let mut positional_only = false;

    for argument in arguments {
        if !positional_only && (argument == "-h" || argument == "--help") {
            return Ok(None);
        }
        if !positional_only && argument == "--" {
            positional_only = true;
            continue;
        }
        if !positional_only && argument.to_string_lossy().starts_with('-') {
            return Err(AppError::Usage(format!(
                "unknown as5643 option '{}'",
                argument.to_string_lossy()
            )));
        }
        if path.replace(PathBuf::from(&argument)).is_some() {
            return Err(AppError::Usage(format!(
                "unexpected extra file argument '{}'",
                argument.to_string_lossy()
            )));
        }
    }

    let path = path.ok_or_else(|| AppError::Usage("missing required <FILE> argument".into()))?;
    Ok(Some(As5643Args { path }))
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

fn execute_records(arguments: &RecordsArgs) -> Result<(), AppError> {
    let mut record_reader = validated_bie_reader(&arguments.path)?;
    let stdout = io::stdout();
    let mut output = BufWriter::new(stdout.lock());
    let mut record_count = 0_usize;

    loop {
        match record_reader.next_item()? {
            Some(BieReadItem::Record(record)) => {
                render_record(&mut output, record_count, &record)
                    .map_err(|error| AppError::io("could not write BIE record inventory", error))?;
                record_count = checked_record_count(record_count)?;
            }
            Some(BieReadItem::Terminator { offset }) => {
                render_terminator(&mut output, offset, record_count)
                    .map_err(|error| AppError::io("could not write BIE record inventory", error))?;
                break;
            }
            None => return Err(validated_terminator_missing()),
        }
    }

    output
        .flush()
        .map_err(|error| AppError::io("could not flush BIE record inventory", error))
}

fn execute_as5643(arguments: &As5643Args) -> Result<AppOutcome, AppError> {
    let mut record_reader = validated_bie_reader(&arguments.path)?;
    let stdout = io::stdout();
    let mut output = BufWriter::new(stdout.lock());
    let mut record_count = 0_usize;
    let mut warning_count = 0_usize;

    loop {
        match record_reader.next_item()? {
            Some(BieReadItem::Record(record)) => {
                let record_warnings = render_as5643_record(&mut output, record_count, &record)
                    .map_err(|error| {
                        AppError::io("could not write AS5643 record inventory", error)
                    })?;
                warning_count = warning_count.checked_add(record_warnings).ok_or_else(|| {
                    AppError::Operation(
                        "payload warning count cannot be represented by usize".into(),
                    )
                })?;
                record_count = checked_record_count(record_count)?;
            }
            Some(BieReadItem::Terminator { offset }) => {
                render_terminator(&mut output, offset, record_count).map_err(|error| {
                    AppError::io("could not write AS5643 record inventory", error)
                })?;
                break;
            }
            None => return Err(validated_terminator_missing()),
        }
    }

    output
        .flush()
        .map_err(|error| AppError::io("could not flush AS5643 record inventory", error))?;

    if warning_count == 0 {
        Ok(AppOutcome::Success)
    } else {
        Ok(AppOutcome::Warnings {
            count: warning_count,
        })
    }
}

fn validated_bie_reader(path: &Path) -> Result<BieReader<BufReader<File>>, AppError> {
    let source = File::open(path)
        .map_err(|error| AppError::io(format!("could not open '{}'", path.display()), error))?;
    let mut validation_reader = BieReader::new(BufReader::new(source), FileOffset::new(0));
    validate_records(&mut validation_reader)?;

    let mut source = validation_reader.into_inner().into_inner();
    source
        .seek(SeekFrom::Start(0))
        .map_err(|error| AppError::io("could not rewind validated BIE input", error))?;
    Ok(BieReader::new(BufReader::new(source), FileOffset::new(0)))
}

fn checked_record_count(record_count: usize) -> Result<usize, AppError> {
    record_count.checked_add(1).ok_or_else(|| {
        AppError::Operation("BIE record count cannot be represented by usize".into())
    })
}

fn validated_terminator_missing() -> AppError {
    AppError::Operation("BIE reader ended without returning its validated terminator".into())
}

fn validate_records(reader: &mut BieReader<impl Read>) -> Result<(), AppError> {
    loop {
        match reader.next_item()? {
            Some(BieReadItem::Record(_)) => {}
            Some(BieReadItem::Terminator { .. }) => return Ok(()),
            None => {
                return Err(AppError::Operation(
                    "BIE reader ended before returning a terminator".into(),
                ));
            }
        }
    }
}

fn render_record(output: &mut impl Write, index: usize, record: &BieRecord<'_>) -> io::Result<()> {
    writeln!(
        output,
        "record={index} offset=0x{:016X} data_item_id=0x{:08X} recorder_seconds={} recorder_microseconds={} status_and_length=0x{:08X} unresolved_flags=0x{:08X} data_length={}",
        record.file_offset().get(),
        record.data_item_id().get(),
        record.recorder_time().seconds(),
        record.recorder_time().microseconds(),
        record.status_and_length().raw(),
        record.status_and_length().unresolved_flags(),
        record.status_and_length().data_length(),
    )
}

fn render_as5643_record(
    output: &mut impl Write,
    index: usize,
    record: &BieRecord<'_>,
) -> io::Result<usize> {
    write!(
        output,
        "record={index} offset=0x{:016X} data_item_id=0x{:08X} recorder_seconds={} recorder_microseconds={} status_and_length=0x{:08X} unresolved_flags=0x{:08X} data_length={} as5643=",
        record.file_offset().get(),
        record.data_item_id().get(),
        record.recorder_time().seconds(),
        record.recorder_time().microseconds(),
        record.status_and_length().raw(),
        record.status_and_length().unresolved_flags(),
        record.status_and_length().data_length(),
    )?;

    match map_bie_record_to_as5643(*record).outcome() {
        BieAs5643MappingOutcome::AssumedAs5643bV1(message) => {
            let validation = message.vpc_validation();
            write!(
                output,
                "mapped profile={} assumption_dependent={} message_id=0x{:08X} reserved_security=0x{:08X} node_id=0x{:08X} priority_and_payload_length=0x{:08X} health_status=0x{:08X} heartbeat=0x{:08X} application_length={} stof_transmit_offset={} stof_receive_offset={} stof_datapump_offset={} stored_vpc=0x{:08X} calculated_vpc=",
                message.profile_id(),
                message.assumption_dependent(),
                message.message_id().get(),
                message.reserved_security(),
                message.node_id(),
                message.priority_and_payload_length(),
                message.health_status(),
                message.heartbeat(),
                message.application_data().len(),
                message.stof_transmit_offset(),
                message.stof_receive_offset(),
                message.stof_datapump_offset(),
                message.stored_vpc(),
            )?;
            if let Some(calculated_vpc) = validation.calculated_vpc() {
                write!(output, "0x{calculated_vpc:08X}")?;
            } else {
                output.write_all(b"none")?;
            }
            write!(
                output,
                " vpc={}",
                vpc_validation_label(validation.outcome())
            )?;
            let warning_count = render_payload_selection(
                output,
                select_payload(
                    PayloadContext::new(record.data_item_id().get()),
                    message.application_data(),
                ),
            )?;
            writeln!(output)?;
            Ok(warning_count)
        }
        BieAs5643MappingOutcome::UnsupportedDataItem => {
            writeln!(output, "unsupported reason=data_item_id")?;
            Ok(0)
        }
        BieAs5643MappingOutcome::UnsupportedStoredDataLength { expected, actual } => {
            writeln!(
                output,
                "unsupported reason=stored_data_length expected={expected} actual={actual}"
            )?;
            Ok(0)
        }
    }
}

fn render_payload_selection(
    output: &mut impl Write,
    selection: PayloadSelection<'_, '_>,
) -> io::Result<usize> {
    match selection {
        PayloadSelection::Matched(matched) => {
            let definition = matched.definition();
            write!(
                output,
                " payload=matched payload_name={} payload_definition={} payload_size={} payload_byte_order={}",
                definition.name(),
                definition.version(),
                matched.raw().size(),
                definition.byte_order().label(),
            )?;
            match matched.decode() {
                Ok(KnownPayload::MsfcsStoresMassDataB(payload)) => {
                    let warnings = payload.warnings();
                    write!(
                        output,
                        " payload_decode=raw_fields system_ticks={} system_elapsed_seconds_provisional={:.12} system_tick_rate_hz={} system_time_epoch=system_startup_unconfirmed message_valid=0x{:02X} message_valid_interpreted={} payload_values_valid={} eots_present=0x{:02X} eots_present_interpreted={} spare_byte=0x{:02X} cm_present=0x{:02X} cm_present_interpreted={} payload_warning_count={}",
                        payload.system_ticks().get(),
                        payload.system_ticks().provisional_elapsed_seconds(),
                        aero1394::payload::msfcs_storesmassdata_b::NOMINAL_SYSTEM_TICK_RATE_HZ,
                        payload.message_valid().get(),
                        boolean_label(payload.message_valid().as_bool()),
                        boolean_label(payload.message_valid().as_bool()),
                        payload.eots_present().get(),
                        boolean_label(payload.eots_present().as_bool()),
                        payload.spare_byte().get(),
                        payload.cm_present().get(),
                        boolean_label(payload.cm_present().as_bool()),
                        warnings.len(),
                    )?;
                    if !warnings.is_empty() {
                        output.write_all(b" payload_warnings=")?;
                        for (index, warning) in warnings.iter().enumerate() {
                            if index > 0 {
                                output.write_all(b",")?;
                            }
                            write!(output, "{warning}")?;
                        }
                    }
                    render_stores_mass_data(output, "current", payload.current_stores_mass_data())?;
                    render_stores_mass_data(output, "post_ej", payload.post_ej_stores_mass_data())?;
                    Ok(warnings.len())
                }
                Err(_) => {
                    output.write_all(b" payload_decode=unavailable")?;
                    Ok(0)
                }
            }
        }
        PayloadSelection::Unknown(raw) => {
            write!(output, " payload=unknown payload_size={}", raw.size())?;
            Ok(0)
        }
        PayloadSelection::Ambiguous(ambiguous) => {
            write!(
                output,
                " payload=ambiguous payload_size={} candidates=",
                ambiguous.raw().size()
            )?;
            for (index, definition) in ambiguous.definitions().iter().enumerate() {
                if index > 0 {
                    output.write_all(b",")?;
                }
                write!(output, "{}@{}", definition.name(), definition.version())?;
            }
            Ok(0)
        }
    }
}

const fn boolean_label(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "true",
        Some(false) => "false",
        None => "unknown",
    }
}

fn render_stores_mass_data(
    output: &mut impl Write,
    prefix: &str,
    fields: StoresMassData,
) -> io::Result<()> {
    write!(
        output,
        " {prefix}_weight={} {prefix}_cg_fs={} {prefix}_cg_bl={} {prefix}_cg_wl={} {prefix}_ixx={} {prefix}_iyy={} {prefix}_izz={} {prefix}_ixy={} {prefix}_iyz={} {prefix}_ixz={}",
        fields.weight().value(),
        fields.cg_fs().value(),
        fields.cg_bl().value(),
        fields.cg_wl().value(),
        fields.ixx().value(),
        fields.iyy().value(),
        fields.izz().value(),
        fields.ixy().value(),
        fields.iyz().value(),
        fields.ixz().value(),
    )
}

const fn vpc_validation_label(outcome: VpcValidationOutcome) -> &'static str {
    match outcome {
        VpcValidationOutcome::Valid => "valid",
        VpcValidationOutcome::Invalid => "invalid",
        VpcValidationOutcome::NotPresent => "not_present",
        VpcValidationOutcome::NotChecked => "not_checked",
    }
}

fn render_terminator(
    output: &mut impl Write,
    offset: FileOffset,
    record_count: usize,
) -> io::Result<()> {
    writeln!(
        output,
        "terminator_offset=0x{:016X} records={record_count}",
        offset.get()
    )
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
    fn parses_a_records_file_path_after_the_option_terminator() {
        let arguments = parse_records_args([os("--"), os("-capture.bie")])
            .expect("arguments are valid")
            .expect("help was not requested");

        assert_eq!(
            arguments,
            RecordsArgs {
                path: PathBuf::from("-capture.bie"),
            }
        );
    }

    #[test]
    fn rejects_records_options() {
        let error = parse_records_args([os("--offset"), os("4"), os("capture.bie")])
            .expect_err("records has no offset option");

        assert!(error.to_string().contains("unknown records option"));
    }

    #[test]
    fn parses_an_as5643_file_path_after_the_option_terminator() {
        let arguments = parse_as5643_args([os("--"), os("-capture.bie")])
            .expect("arguments are valid")
            .expect("help was not requested");

        assert_eq!(
            arguments,
            As5643Args {
                path: PathBuf::from("-capture.bie"),
            }
        );
    }

    #[test]
    fn rejects_as5643_options() {
        let error = parse_as5643_args([os("--profile"), os("other"), os("capture.bie")])
            .expect_err("as5643 profile selection is not implicit or user-defined");

        assert!(error.to_string().contains("unknown as5643 option"));
    }

    /// Requirements: L3-OUT-001, L3-OUT-002, L3-OUT-006
    #[test]
    fn renders_raw_bie_record_inventory_values() {
        let bytes = [
            0xDE, 0xAD, 0xBE, 0xEF, 0, 0, 0, 10, 0, 0, 0, 20, 0x40, 0, 0, 1, 0xAA, 0, 0, 0, 0,
        ];
        let (record, consumed) = aero1394::bie::parse_record(&bytes, FileOffset::new(0x100))
            .expect("test BIE record parses");
        let mut output = Vec::new();

        render_record(&mut output, 0, &record).expect("record write succeeds");
        render_terminator(&mut output, FileOffset::new(0x100 + consumed as u64), 1)
            .expect("terminator write succeeds");

        assert_eq!(
            String::from_utf8(output).expect("ASCII output"),
            "record=0 offset=0x0000000000000100 data_item_id=0xDEADBEEF recorder_seconds=10 recorder_microseconds=20 status_and_length=0x40000001 unresolved_flags=0x40000000 data_length=1\nterminator_offset=0x0000000000000111 records=1\n"
        );
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

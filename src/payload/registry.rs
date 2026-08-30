use super::PayloadFieldDefinition;
use super::msfcs_storesmassdata_b;
use std::error::Error;
use std::fmt;

/// Byte order declared by a payload definition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PayloadByteOrder {
    /// Most-significant byte is stored first.
    BigEndian,
    /// Least-significant byte is stored first.
    LittleEndian,
}

impl PayloadByteOrder {
    /// Returns the stable human-readable label for this byte order.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::BigEndian => "big-endian",
            Self::LittleEndian => "little-endian",
        }
    }
}

/// Available selectors for one application payload.
///
/// Data code and configuration are optional because they are not present in
/// every input representation. A definition that requires either selector
/// cannot match when the caller does not supply it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PayloadContext<'a> {
    data_item_id: u32,
    data_code: Option<&'a str>,
    configuration: Option<&'a str>,
}

impl<'a> PayloadContext<'a> {
    /// Creates context with the minimum required data-item identity.
    #[must_use]
    pub const fn new(data_item_id: u32) -> Self {
        Self {
            data_item_id,
            data_code: None,
            configuration: None,
        }
    }

    /// Adds an available recorder or bus data-code selector.
    #[must_use]
    pub const fn with_data_code(mut self, data_code: &'a str) -> Self {
        self.data_code = Some(data_code);
        self
    }

    /// Adds an available configuration selector.
    #[must_use]
    pub const fn with_configuration(mut self, configuration: &'a str) -> Self {
        self.configuration = Some(configuration);
        self
    }

    /// Returns the raw data-item identity supplied by the caller.
    #[must_use]
    pub const fn data_item_id(self) -> u32 {
        self.data_item_id
    }

    /// Returns the optional data-code selector.
    #[must_use]
    pub const fn data_code(self) -> Option<&'a str> {
        self.data_code
    }

    /// Returns the optional configuration selector.
    #[must_use]
    pub const fn configuration(self) -> Option<&'a str> {
        self.configuration
    }
}

/// Registry metadata for one built-in payload definition.
///
/// Data-item identity and exact payload size are always match criteria.
/// Optional constraints are applied only by definitions that declare them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PayloadDefinition {
    name: &'static str,
    version: &'static str,
    data_item_id: u32,
    payload_size: usize,
    byte_order: PayloadByteOrder,
    fields: &'static [PayloadFieldDefinition],
    data_code: Option<&'static str>,
    configuration: Option<&'static str>,
}

impl PayloadDefinition {
    /// Creates a definition with the required identity and size criteria.
    #[must_use]
    pub const fn new(
        name: &'static str,
        version: &'static str,
        data_item_id: u32,
        payload_size: usize,
        byte_order: PayloadByteOrder,
    ) -> Self {
        Self {
            name,
            version,
            data_item_id,
            payload_size,
            byte_order,
            fields: &[],
            data_code: None,
            configuration: None,
        }
    }

    /// Declares the explicit fields owned by this definition.
    #[must_use]
    pub const fn with_fields(mut self, fields: &'static [PayloadFieldDefinition]) -> Self {
        self.fields = fields;
        self
    }

    /// Requires an exact data code in addition to identity and size.
    #[must_use]
    pub const fn with_data_code(mut self, data_code: &'static str) -> Self {
        self.data_code = Some(data_code);
        self
    }

    /// Requires an exact configuration in addition to identity and size.
    #[must_use]
    pub const fn with_configuration(mut self, configuration: &'static str) -> Self {
        self.configuration = Some(configuration);
        self
    }

    /// Returns the stable definition name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        self.name
    }

    /// Returns the stable Aero1394 definition version.
    #[must_use]
    pub const fn version(self) -> &'static str {
        self.version
    }

    /// Returns the required data-item identity.
    #[must_use]
    pub const fn data_item_id(self) -> u32 {
        self.data_item_id
    }

    /// Returns the required exact application size.
    #[must_use]
    pub const fn payload_size(self) -> usize {
        self.payload_size
    }

    /// Returns the declared multi-byte field order.
    #[must_use]
    pub const fn byte_order(self) -> PayloadByteOrder {
        self.byte_order
    }

    /// Returns every field in authoritative definition order.
    #[must_use]
    pub const fn fields(self) -> &'static [PayloadFieldDefinition] {
        self.fields
    }

    /// Returns the optional required data code.
    #[must_use]
    pub const fn data_code(self) -> Option<&'static str> {
        self.data_code
    }

    /// Returns the optional required configuration.
    #[must_use]
    pub const fn configuration(self) -> Option<&'static str> {
        self.configuration
    }

    fn matches(self, context: PayloadContext<'_>, payload_size: usize) -> bool {
        self.data_item_id == context.data_item_id()
            && self.payload_size == payload_size
            && optional_constraint_matches(self.data_code, context.data_code())
            && optional_constraint_matches(self.configuration, context.configuration())
    }
}

fn optional_constraint_matches(required: Option<&str>, actual: Option<&str>) -> bool {
    required.is_none_or(|required| actual == Some(required))
}

/// Exact undecoded application bytes together with their selection context.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RawPayload<'a> {
    context: PayloadContext<'a>,
    bytes: &'a [u8],
}

impl<'a> RawPayload<'a> {
    /// Returns all available identity context.
    #[must_use]
    pub const fn context(self) -> PayloadContext<'a> {
        self.context
    }

    /// Returns the exact application bytes supplied to the registry.
    #[must_use]
    pub const fn bytes(self) -> &'a [u8] {
        self.bytes
    }

    /// Returns the observed application size.
    #[must_use]
    pub const fn size(self) -> usize {
        self.bytes.len()
    }
}

/// One unambiguous registry match and its undecoded bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MatchedPayload<'definitions, 'payload> {
    definition: &'definitions PayloadDefinition,
    raw: RawPayload<'payload>,
}

impl<'definitions, 'payload> MatchedPayload<'definitions, 'payload> {
    /// Returns the selected built-in definition.
    #[must_use]
    pub const fn definition(self) -> &'definitions PayloadDefinition {
        self.definition
    }

    /// Returns the preserved application input.
    #[must_use]
    pub const fn raw(self) -> RawPayload<'payload> {
        self.raw
    }

    /// Decodes the selected built-in definition into its typed raw fields.
    pub fn decode(self) -> Result<KnownPayload<'payload>, MatchedPayloadDecodeError> {
        if *self.definition == msfcs_storesmassdata_b::DEFINITION {
            return msfcs_storesmassdata_b::decode(self.raw.bytes())
                .map(KnownPayload::MsfcsStoresMassDataB)
                .map_err(MatchedPayloadDecodeError::MsfcsStoresMassDataB);
        }

        Err(MatchedPayloadDecodeError::UnsupportedDefinition {
            name: self.definition.name(),
            version: self.definition.version(),
        })
    }
}

/// Typed raw payloads supported by the built-in registry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KnownPayload<'a> {
    /// Raw fields from the supplied `msfcs_storesmassdata_b` layout.
    MsfcsStoresMassDataB(msfcs_storesmassdata_b::MsfcsStoresMassDataB<'a>),
}

/// Failure to decode a definition after an unambiguous registry match.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MatchedPayloadDecodeError {
    /// A caller-created registry definition has no built-in typed decoder.
    UnsupportedDefinition {
        /// Stable definition name.
        name: &'static str,
        /// Stable definition version.
        version: &'static str,
    },
    /// The Stores Mass raw decoder rejected its definition or bytes.
    MsfcsStoresMassDataB(msfcs_storesmassdata_b::DecodeError),
}

impl fmt::Display for MatchedPayloadDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedDefinition { name, version } => {
                write!(
                    formatter,
                    "payload definition {name}@{version} has no built-in decoder"
                )
            }
            Self::MsfcsStoresMassDataB(error) => error.fmt(formatter),
        }
    }
}

impl Error for MatchedPayloadDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::UnsupportedDefinition { .. } => None,
            Self::MsfcsStoresMassDataB(error) => Some(error),
        }
    }
}

/// Multiple compatible definitions and the preserved undecoded payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AmbiguousPayload<'definitions, 'payload> {
    definitions: Vec<&'definitions PayloadDefinition>,
    raw: RawPayload<'payload>,
}

impl<'definitions, 'payload> AmbiguousPayload<'definitions, 'payload> {
    /// Returns every compatible definition in stable registry order.
    #[must_use]
    pub fn definitions(&self) -> &[&'definitions PayloadDefinition] {
        &self.definitions
    }

    /// Returns the preserved application input.
    #[must_use]
    pub const fn raw(&self) -> RawPayload<'payload> {
        self.raw
    }
}

/// Deterministic result of applying a payload registry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PayloadSelection<'definitions, 'payload> {
    /// Exactly one definition matched every available selector.
    Matched(MatchedPayload<'definitions, 'payload>),
    /// No definition matched; the complete payload remains available.
    Unknown(RawPayload<'payload>),
    /// More than one definition matched; no definition was selected.
    Ambiguous(AmbiguousPayload<'definitions, 'payload>),
}

/// An ordered set of compiled-in payload definitions.
#[derive(Clone, Copy, Debug)]
pub struct PayloadRegistry<'definitions> {
    definitions: &'definitions [PayloadDefinition],
}

impl<'definitions> PayloadRegistry<'definitions> {
    /// Creates a registry whose definition order remains stable.
    #[must_use]
    pub const fn new(definitions: &'definitions [PayloadDefinition]) -> Self {
        Self { definitions }
    }

    /// Returns the registered definitions in deterministic selection order.
    #[must_use]
    pub const fn definitions(self) -> &'definitions [PayloadDefinition] {
        self.definitions
    }

    /// Selects one definition without consuming or copying application bytes.
    #[must_use]
    pub fn select<'payload>(
        self,
        context: PayloadContext<'payload>,
        bytes: &'payload [u8],
    ) -> PayloadSelection<'definitions, 'payload> {
        let raw = RawPayload { context, bytes };
        let mut matches = self
            .definitions
            .iter()
            .filter(|definition| definition.matches(context, bytes.len()));

        let Some(first) = matches.next() else {
            return PayloadSelection::Unknown(raw);
        };
        let Some(second) = matches.next() else {
            return PayloadSelection::Matched(MatchedPayload {
                definition: first,
                raw,
            });
        };

        let mut definitions = vec![first, second];
        definitions.extend(matches);
        PayloadSelection::Ambiguous(AmbiguousPayload { definitions, raw })
    }
}

const BUILT_IN_DEFINITIONS: [PayloadDefinition; 1] = [msfcs_storesmassdata_b::DEFINITION];
const BUILT_IN_REGISTRY: PayloadRegistry<'static> = PayloadRegistry::new(&BUILT_IN_DEFINITIONS);

/// Selects a payload using Aero1394's built-in Rust definitions.
#[must_use]
pub fn select_payload<'payload>(
    context: PayloadContext<'payload>,
    bytes: &'payload [u8],
) -> PayloadSelection<'static, 'payload> {
    BUILT_IN_REGISTRY.select(context, bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_BYTES: [u8; 4] = [0xDE, 0xAD, 0xBE, 0xEF];
    const FIRST: PayloadDefinition = PayloadDefinition::new(
        "first",
        "1",
        0x1234,
        TEST_BYTES.len(),
        PayloadByteOrder::BigEndian,
    );
    const SECOND: PayloadDefinition = PayloadDefinition::new(
        "second",
        "1",
        0x1234,
        TEST_BYTES.len(),
        PayloadByteOrder::LittleEndian,
    );

    /// Requirements: L3-PAY-004, L3-PAY-005
    #[test]
    fn selects_exactly_one_identity_and_size_match() {
        let definitions = [FIRST];
        let registry = PayloadRegistry::new(&definitions);

        let PayloadSelection::Matched(matched) =
            registry.select(PayloadContext::new(0x1234), &TEST_BYTES)
        else {
            panic!("identity and exact size must select the definition");
        };

        assert_eq!(*matched.definition(), FIRST);
        assert_eq!(matched.raw().bytes(), TEST_BYTES);
    }

    /// Requirements: L3-PAY-004, L3-PAY-005, L3-PAY-006, L3-TST-005
    #[test]
    fn preserves_unknown_identity_context_size_and_bytes() {
        let definitions = [FIRST];
        let registry = PayloadRegistry::new(&definitions);
        let context = PayloadContext::new(0x5678)
            .with_data_code("bus-b")
            .with_configuration("ground-test");

        let PayloadSelection::Unknown(raw) = registry.select(context, &TEST_BYTES) else {
            panic!("unknown identity must remain raw");
        };

        assert_eq!(raw.context(), context);
        assert_eq!(raw.size(), TEST_BYTES.len());
        assert_eq!(raw.bytes(), TEST_BYTES);
    }

    /// Requirements: L3-PAY-004, L3-PAY-005, L3-TST-005
    #[test]
    fn reports_every_ambiguous_match_in_registry_order() {
        let definitions = [FIRST, SECOND];
        let registry = PayloadRegistry::new(&definitions);

        let PayloadSelection::Ambiguous(ambiguous) =
            registry.select(PayloadContext::new(0x1234), &TEST_BYTES)
        else {
            panic!("overlapping definitions must remain ambiguous");
        };

        assert_eq!(ambiguous.definitions(), [&FIRST, &SECOND]);
        assert_eq!(ambiguous.raw().bytes(), TEST_BYTES);
    }

    /// Requirements: L3-PAY-004, L3-PAY-005
    #[test]
    fn uses_available_data_code_and_configuration_constraints() {
        let definitions = [
            FIRST.with_data_code("bus-a").with_configuration("flight"),
            SECOND.with_data_code("bus-b").with_configuration("flight"),
        ];
        let registry = PayloadRegistry::new(&definitions);
        let context = PayloadContext::new(0x1234)
            .with_data_code("bus-b")
            .with_configuration("flight");

        let PayloadSelection::Matched(matched) = registry.select(context, &TEST_BYTES) else {
            panic!("available constraints must disambiguate definitions");
        };

        assert_eq!(matched.definition().name(), "second");
    }

    /// Requirements: L3-PAY-004, L3-PAY-005
    #[test]
    fn requires_exact_size_and_any_declared_context() {
        let definitions = [FIRST.with_data_code("bus-a")];
        let registry = PayloadRegistry::new(&definitions);

        assert!(matches!(
            registry.select(PayloadContext::new(0x1234), &TEST_BYTES),
            PayloadSelection::Unknown(_)
        ));
        assert!(matches!(
            registry.select(PayloadContext::new(0x1234).with_data_code("bus-a"), &[0; 3]),
            PayloadSelection::Unknown(_)
        ));
    }
}

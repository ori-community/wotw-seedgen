pub mod tokenizer;
use tokenizer::TokenStream;
pub(crate) mod parser;
mod emitter;
mod v;
mod tools;
mod code;

pub use emitter::{HeaderBuild, ItemDetails};
pub use v::{VResolve, V, VString};
pub(crate) use v::vdisplay;
pub use tools::validate_headers;
pub use code::CodeDisplay;
use std::{fmt, str::FromStr};

use crate::{util::{Icon, UberStateTrigger, VUberStateTrigger, UberIdentifier}, VItem, Item};

use rustc_hash::FxHashMap;
use rand::Rng;

use parser::{parse_header_contents};
use super::{ParseError, parser::ParseErrorCollection};

use wotw_seedgen_derive::{FromStr, VVariant};

/// An item placed at a location trigger
#[derive(Debug, Clone, VVariant)]
pub struct Pickup {
    /// [`UberStateTrigger`] that should grant the [`Item`]
    #[VType]
    pub trigger: UberStateTrigger,
    /// [`Item`] to be granted
    #[VType]
    pub item: Item,
    /// Whether this pickup should be ignored for any logic the seed generator applies based on header
    pub ignore: bool,
    /// Whether this pickup should be ignored during header validation
    pub skip_validation: bool,
}
impl Pickup {
    pub fn code(&self) -> CodeDisplay<Pickup> {
        CodeDisplay::new(self, |s, f| { write!(f, "{}|{}", s.trigger.code(), s.item.code())})
    }
}

#[derive(Debug, Clone)]
/// Abstract representation of a header file
pub struct Header {
    /// Contents of the header
    pub contents: Vec<HeaderContent>,
}

impl Header {
    /// Parse complete header syntax
    /// 
    /// All `!!pool`, `!!flush` and `!!take` syntax will be evaluated at this time, using the provided rng
    pub fn parse(mut input: String, rng: &mut impl Rng) -> Result<Header, ParseErrorCollection> {
        // TODO not actually parsing pool means anything using pool gets wrong errors
        parser::preprocess(&mut input, rng).map_err(|err| vec![ParseError::new(format!("Error preprocessing: {err}"), "", 0..0)])?;
        let mut parser = parser::new(&input);
        let contents = parse_header_contents(&mut parser)?;
        Ok(Header { contents })
    }

    /// Evaluates the header based on the provided parameters and returns the desired changes to seed generation
    /// 
    /// Returns an error if the parameters lead to invalid syntax
    /// See [`HeaderBuild`] for more information
    pub fn build(self, mut parameters: FxHashMap<String, String>) -> Result<HeaderBuild, String> {
        self.fill_parameters(&mut parameters)?;
        emitter::build(self.contents, &parameters)
    }

    fn fill_parameters(&self, parameters: &mut FxHashMap<String, String>) -> Result<(), String> {
        let own_parameters = self.parameters();

        // Reject any unknown parameters
        if let Some(unknown) = parameters.keys().find(|&identifier| !own_parameters.iter().any(|own| &own.identifier == identifier)) {
            return Err(format!("Unknown parameter {unknown}"));
        }

        // Validate custom parameters
        for ParameterInfo { identifier, default, .. } in own_parameters {
            if let Some(custom) = parameters.get(&identifier) {
                match default.kind() {
                    ParameterType::Bool => { custom.parse::<bool>().map_err(|_| format!("invalid value for parameter {identifier}"))?; },
                    ParameterType::Int => { custom.parse::<i32>().map_err(|_| format!("invalid value for parameter {identifier}"))?; },
                    ParameterType::Float => { custom.parse::<f32>().map_err(|_| format!("invalid value for parameter {identifier}"))?; },
                    ParameterType::String => {},
                }
            } else {
                parameters.insert(identifier, default.to_string());
            }
        }

        Ok(())
    }

    /// Returns the annotations of this header
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use wotw_seedgen::Header;
    /// use wotw_seedgen::header::Annotation;
    /// 
    /// let input = "#hide\n9|0|8|9|0|int|0".to_string();
    /// 
    /// let header = Header::parse(input, &mut rand::thread_rng()).unwrap();
    /// let annotations = header.annotations();
    /// 
    /// assert_eq!(annotations, vec![&Annotation::Hide]);
    /// ```
    pub fn annotations(&self) -> Vec<&Annotation> {
        self.contents.iter().filter_map(|content|
            if let HeaderContent::Annotation(annotation) = content {
                Some(annotation)
            } else { None }
        ).collect()
    }

    /// Returns the configuration parameters for this header
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use wotw_seedgen::Header;
    /// use wotw_seedgen::header::ParameterDefault;
    /// use wotw_seedgen::header::ParameterInfo;
    /// 
    /// let input = "!!parameter fun int:69".to_string();
    /// let header = Header::parse(input, &mut rand::thread_rng()).unwrap();
    /// 
    /// let parameters = header.parameters();
    /// 
    /// assert_eq!(parameters, vec![ParameterInfo {
    ///     identifier: "fun".to_string(),
    ///     default: ParameterDefault::Int(69),
    ///     documentation: None,
    /// }]);
    /// ```
    pub fn parameters(&self) -> Vec<ParameterInfo> {
        let mut last_documentation = None;
        self.contents.iter().filter_map(|content| {
            let documentation = last_documentation.take();
            match content {
                HeaderContent::InnerDocumentation(documentation) => {
                    last_documentation = Some(documentation.clone());
                    None
                },
                HeaderContent::Command(HeaderCommand::Parameter { identifier, default }) =>
                    Some(ParameterInfo { identifier: identifier.clone(), default: default.clone(), documentation }),
                _ => None,
            }
        }).collect()
    }

    /// Returns the documentation for this header
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use wotw_seedgen::Header;
    /// use wotw_seedgen::header::Annotation;
    /// 
    /// let input = "#hide\n/// My first header\n///\n/// Someday I'll have this header do something!".to_string();
    /// let header = Header::parse(input, &mut rand::thread_rng()).unwrap();
    /// 
    /// let documentation = header.documentation();
    /// 
    /// assert_eq!(documentation.name, Some("My first header".to_string()));
    /// assert_eq!(documentation.description, Some("Someday I'll have this header do something!".to_string()));
    /// ```
    pub fn documentation(&self) -> HeaderDocumentation {
        let mut name = None;
        let mut description: Option<String> = None;
        for content in &self.contents {
            if let HeaderContent::OuterDocumentation(documentation) = content {
                if documentation.is_empty() { continue }
                if name.is_none() {
                    name = Some(documentation.clone());
                } else if let Some(prior) = &mut description {
                    prior.push('\n');
                    prior.push_str(documentation);
                } else {
                    description = Some(documentation.clone());
                }
            }
        }

        HeaderDocumentation { name, description }
    }

    /// Returns the annotations of a given header syntax
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use wotw_seedgen::Header;
    /// use wotw_seedgen::header::Annotation;
    /// 
    /// let input = "#hide\n9|0|8|9|0|int|0";
    /// 
    /// let annotations = Header::parse_annotations(input).unwrap();
    /// 
    /// assert_eq!(annotations, vec![Annotation::Hide]);
    /// ```
    /// 
    /// This will only parse the minimum amount required to know the annotations
    /// 
    /// ```
    /// # use wotw_seedgen::Header;
    /// # 
    /// let input = "#hide\n3|6|This isn't even valid header syntax!";
    /// 
    /// assert!(Header::parse_annotations(input).is_ok());
    /// ```
    pub fn parse_annotations(input: &str) -> Result<Vec<Annotation>, String> {
        let mut annotations = vec![];

        for line in input.lines() {
            if let Some(mut annotation) = line.strip_prefix('#') {
                let end = annotation.find('\n').unwrap_or(annotation.len());
                annotation = &annotation[..end];
                if let Some(end) = annotation.find("//") { annotation = &annotation[..end] }
                if let Ok(annotation) = Annotation::from_str(annotation) {
                    annotations.push(annotation);
                }
            } else if !line.is_empty() {
                break;
            }
        }

        Ok(annotations)
    }

    /// Returns the name and description of a given header syntax
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use wotw_seedgen::Header;
    /// use wotw_seedgen::header::Annotation;
    /// 
    /// let input = "#hide\n/// My first header\n///\n/// Someday I'll have this header do something!";
    /// 
    /// let documentation = Header::parse_documentation(input);
    /// 
    /// assert_eq!(documentation.name, Some("My first header".to_string()));
    /// assert_eq!(documentation.description, Some("Someday I'll have this header do something!".to_string()));
    /// ```
    /// 
    /// This will only parse the minimum amount required to know the documentation
    /// 
    /// ```
    /// # use wotw_seedgen::Header;
    /// # 
    /// let input = "/// A very bad header\n3|6|This isn't even valid header syntax!";
    /// 
    /// let documentation = Header::parse_documentation(input);
    /// 
    /// assert_eq!(documentation.name, Some("A very bad header".to_string()));
    /// assert_eq!(documentation.description, None);
    /// ```
    pub fn parse_documentation(input: &str) -> HeaderDocumentation {
        let mut name = None;
        let mut description: Option<String> = None;

        for line in input.lines() {
            if line.is_empty() || line.starts_with('#') { continue }
            if let Some(documentation) = line.trim_start().strip_prefix("///") {
                if documentation.starts_with('/') { break }
                let documentation = documentation.trim();
                if documentation.is_empty() { continue }
                if name.is_none() {
                    name = Some(documentation.to_string());
                } else if let Some(prior) = &mut description {
                    prior.push('\n');
                    prior.push_str(documentation);
                } else {
                    description = Some(documentation.to_string());
                }
            } else { break }
        }

        HeaderDocumentation { name, description }
    }

    /// Returns the parameters present in the header, including their names and default values
    /// 
    /// This will parse any parameter lines to read their relevant values, but skip parsing anything else
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use wotw_seedgen::Header;
    /// use wotw_seedgen::header::ParameterDefault;
    /// use wotw_seedgen::header::ParameterInfo;
    /// 
    /// let input = "3|0|6|Good luck have fun!\n//// Some ad\n!!parameter extra_text string:Hier könnte ihre Werbung stehen!\n3|0|6|$PARAM(extra_text)";
    /// 
    /// let parameters = Header::parse_parameters(input);
    /// 
    /// assert_eq!(parameters, vec![ParameterInfo {
    ///     identifier: "extra_text".to_string(),
    ///     default: ParameterDefault::String("Hier könnte ihre Werbung stehen!".to_string()),
    ///     documentation: Some("Some ad".to_string()),
    /// }]);
    /// ```
    pub fn parse_parameters(input: &str) -> Vec<ParameterInfo> {
        let mut last_documentation = None;
        input.lines().filter_map(|line| {
            let documentation = if let Some(documentation) = line.strip_prefix("////") {
                if !documentation.starts_with('/') {
                    last_documentation = Some(documentation.trim().to_owned());
                }
                return None;
            } else { last_documentation.take() };
            line.strip_prefix("!!").and_then(|command|
                if command.starts_with("parameter ") {
                    HeaderCommand::from_str(command).ok().map(|command|
                        if let HeaderCommand::Parameter { identifier, default } = command {
                            ParameterInfo { identifier, default, documentation }
                        } else { unreachable!() }
                    )
                } else { None }
            )
        }).collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParameterInfo {
    pub identifier: String,
    pub default: ParameterDefault,
    pub documentation: Option<String>,
}

/// Annotations providing meta information about how to treat the header
#[derive(Debug, Clone, PartialEq)]
pub enum Annotation {
    /// Hide this header from the user, it is only to be used internally through includes
    Hide,
    /// Put this header into a category with other, similar headers
    Category(String),
}

#[derive(Debug, Clone)]
pub struct HeaderDocumentation {
    /// Brief name, this may never exceed one line
    /// 
    /// [`None`] if not provided by the header
    pub name: Option<String>,
    /// Extended description
    /// 
    /// [`None`] if not provided by the header
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
/// One statement in a header
pub enum HeaderContent {
    /// Documentation for the Header
    OuterDocumentation(String),
    /// Documentation for contained configuration parameters
    InnerDocumentation(String),
    /// Meta annotations
    Annotation(Annotation),
    /// A List of Flags to add to the resulting seed
    Flags(Vec<String>),
    /// A timer definition to add to the resulting seed
    Timer(TimerDefinition),
    /// A header command to be applied at generation time
    Command(HeaderCommand),
    /// A pickup to add to the resulting seed
    Pickup(VPickup),
}

#[derive(Debug, Clone)]
pub struct TimerDefinition {
    pub switch: UberIdentifier,
    pub counter: UberIdentifier,
}
impl TimerDefinition {
    pub fn code(&self) -> CodeDisplay<TimerDefinition> {
        CodeDisplay::new(self, |s, f| write!(f, "{}|{}", s.switch.code(), s.counter.code()))
    }
}

#[derive(Debug, Clone)]
/// Header-specific commands that influence seed generation, but won't be added to the resulting seed
pub enum HeaderCommand {
    Include { name: String },
    Exclude { name: String },
    Add { item: VItem, amount: V<i32> },
    Remove { item: VItem, amount: V<i32> },
    Name { item: VItem, name: VString },
    Display { item: VItem, name: VString },
    Description { item: VItem, description: VString },
    Price { item: VItem, price: V<u32> },
    Icon { item: VItem, icon: Icon },
    Parameter { identifier: String, default: ParameterDefault },
    Set { state: String },
    If { parameter: String, value: String },
    EndIf,
    GoalmodeHack(GoalmodeHack),
}

#[derive(Debug, Clone)]
pub enum GoalmodeHack {
    Trees,
    Wisps,
    Quests,
    Relics { chance: V<f64>, amount: V<usize> },
}

/// Type and value of a parameter's default
#[derive(Debug, Clone, PartialEq, FromStr)]
#[ParseFromIdentifier]
pub enum ParameterType {
    Bool,
    Int,
    Float,
    String,
}

/// Type and value of a parameter's default
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterDefault {
    Bool(bool),
    Int(i32),
    Float(f32),
    String(String),
}

impl ParameterDefault {
    pub fn kind(&self) -> ParameterType {
        match self {
            ParameterDefault::Bool(_) => ParameterType::Bool,
            ParameterDefault::Int(_) => ParameterType::Int,
            ParameterDefault::Float(_) => ParameterType::Float,
            ParameterDefault::String(_) => ParameterType::String,
        }
    }
}

impl FromStr for ParameterDefault {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut default_parts = s.splitn(2, ':');
        let first_part = default_parts.next().unwrap();
        let (parameter_type, default) = if let Some(default) = default_parts.next() {
            (first_part, default)
        } else {
            ("string", first_part)
        };

        let default = match parameter_type {
            "bool" => ParameterDefault::Bool(default.parse().map_err(|_| format!("invalid value boolean {default}"))?),
            "int" => ParameterDefault::Int(default.parse().map_err(|_| format!("invalid value integer {default}"))?),
            "float" => ParameterDefault::Float(default.parse().map_err(|_| format!("invalid value float {default}"))?),
            "string" => ParameterDefault::String(default.to_string()),
            _ => return Err(format!("invalid parameter type {parameter_type}")),
        };

        Ok(default)
    }
}

impl fmt::Display for ParameterDefault {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParameterDefault::Bool(bool) => write!(f, "{bool}"),
            ParameterDefault::Int(i32) => write!(f, "{i32}"),
            ParameterDefault::Float(f32) => write!(f, "{f32}"),
            ParameterDefault::String(string) => write!(f, "{string}"),
        }
    }
}

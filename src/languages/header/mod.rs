pub mod parser;
mod emitter;
mod v;
mod tools;

pub use emitter::{HeaderBuild, ItemDetails};
pub use v::{VResolve, V, VString};
pub use tools::{list, inspect, validate};

use std::{fmt, str::FromStr};

use crate::{util::{Icon, extensions::StrExtension}, VItem, seed::{TimerDefinition, VPickup}};

use rustc_hash::FxHashMap;
use rand::Rng;

#[derive(Debug, Clone)]
/// Abstract representation of a header file
pub struct Header {
    /// Top level annotations such as `#hide`
    pub annotations: Vec<Annotation>,
    /// Top level documentation
    pub documentation: HeaderDocumentation,
    /// Documentation for contained parameters
    pub parameter_documentation: FxHashMap<String, String>,
    /// Contents of the header
    pub contents: Vec<HeaderContent>,
}

impl Header {
    /// Parse complete header syntax
    /// 
    /// All `!!pool`, `!!flush` and `!!take` syntax will be evaluated at this time, using the provided rng
    pub fn parse<R: Rng>(mut input: String, rng: &mut R) -> Result<Header, String> {
        let annotations = parser::parse_annotations(&mut input)?;
        let documentation = parser::parse_documentation(&input);
        parser::preprocess(&mut input, rng)?;
        let parser::HeaderContents { contents, parameter_documentation } = parser::parse_contents(input)?;
        Ok(Header { annotations, documentation, parameter_documentation, contents })
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
        if let Some(unknown) = parameters.keys().find(|&identifier| !own_parameters.iter().any(|own| own.0 == identifier)) {
            return Err(format!("Unknown parameter {unknown}"));
        }

        // Validate custom parameters
        for (identifier, default_value) in own_parameters {
            if let Some(custom) = parameters.get(identifier) {
                match default_value.kind() {
                    ParameterDefaultKind::Bool => { custom.parse::<bool>().map_err(|_| format!("invalid value for parameter {identifier}"))?; },
                    ParameterDefaultKind::Int => { custom.parse::<i32>().map_err(|_| format!("invalid value for parameter {identifier}"))?; },
                    ParameterDefaultKind::Float => { custom.parse::<f32>().map_err(|_| format!("invalid value for parameter {identifier}"))?; },
                    ParameterDefaultKind::String => {},
                }
            } else {
                parameters.insert(identifier.clone(), default_value.to_string());
            }
        }

        Ok(())
    }

    /// Returns the configuration parameters for this header
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use seedgen::Header;
    /// use seedgen::header::ParameterDefault;
    /// 
    /// let input = "!!parameter fun int:69".to_string();
    /// let header = Header::parse(input, &mut rand::thread_rng()).unwrap();
    /// 
    /// let parameters = header.parameters();
    /// 
    /// assert_eq!(parameters, vec![
    ///     (&"fun".to_string(), &ParameterDefault::Int(69))
    /// ]);
    /// ```
    pub fn parameters(&self) -> Vec<(&String, &ParameterDefault)> {
        self.contents.iter().filter_map(|content|
            if let HeaderContent::Command(HeaderCommand::Parameter { identifier, default }) = content {
                Some((identifier, default))
            } else { None }
        ).collect()
    }

    /// Returns the annotations of a given header syntax
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use seedgen::Header;
    /// use seedgen::header::Annotation;
    /// 
    /// let input = "#hide\n9|0|8|9|0|int|0".to_string();
    /// 
    /// let annotations = Header::parse_annotations(input).unwrap();
    /// 
    /// assert_eq!(annotations, vec![Annotation::Hide]);
    /// ```
    /// 
    /// This will only parse the minimum amount required to know the annotations
    /// 
    /// ```
    /// # use seedgen::Header;
    /// # 
    /// let input = "#hide\n3|6|This isn't even valid header syntax!".to_string();
    /// 
    /// assert!(Header::parse_annotations(input).is_ok());
    /// ```
    pub fn parse_annotations(mut input: String) -> Result<Vec<Annotation>, String> {
        parser::parse_annotations(&mut input)
    }

    /// Returns the name and description of a given header syntax
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use seedgen::Header;
    /// use seedgen::header::Annotation;
    /// 
    /// let input = "/// My first header\n///\n/// Someday I'll have this header do something!";
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
    /// # use seedgen::Header;
    /// # 
    /// let input = "/// A very bad header\n3|6|This isn't even valid header syntax!";
    /// 
    /// let documentation = Header::parse_documentation(input);
    /// 
    /// assert_eq!(documentation.name, Some("A very bad header".to_string()));
    /// assert_eq!(documentation.description, None);
    /// ```
    pub fn parse_documentation(input: &str) -> HeaderDocumentation {
        let mut after_annotations = input.len();
        for range in input.line_ranges() {
            let range_start = range.start;
            let line = &input[range];
            if line.starts_with('#') || line.is_empty() { continue }
            else {
                after_annotations = range_start;
                break;
            }
        }
        parser::parse_documentation(&input[after_annotations..])
    }

    /// Returns the parameters present in the header, including their names and default values
    /// 
    /// This will parse any parameter lines to read their relevant values, but skip parsing anything else
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use seedgen::Header;
    /// use seedgen::header::ParameterDefault;
    /// 
    /// let input = "3|0|6|Good luck have fun!\n!!parameter extra_text Hier könnte ihre Werbung stehen!\n3|0|6|$PARAM(extra_text)";
    /// 
    /// let parameters = Header::parse_parameters(input);
    /// 
    /// assert_eq!(parameters, vec![(
    ///     "extra_text".to_string(),
    ///     ParameterDefault::String("Hier könnte ihre Werbung stehen!".to_string())
    /// )]);
    /// ```
    pub fn parse_parameters(input: &str) -> Vec<(String, ParameterDefault)> {
        input.lines().filter_map(|line|
            line.strip_prefix("!!").and_then(|command|
                if command.starts_with("parameter ") {
                    HeaderCommand::parse(command).ok().map(|command|
                        if let HeaderCommand::Parameter { identifier, default } = command {
                            (identifier, default)
                        } else { unreachable!() }
                    )
                } else { None }
            )
        ).collect()
    }
}

/// Annotations providing meta information about how to treat the header
#[derive(Debug, Clone, Copy, PartialEq, seedgen_derive::FromStr)]
#[ParseFromIdentifier]
pub enum Annotation {
    /// Hide this header from the user, it is only to be used internally through includes
    Hide,
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
    /// A List of Flags to add to the resulting seed
    Flags(Vec<V<String>>),
    /// A header command to be applied at generation time
    Command(HeaderCommand),
    /// A timer definition to add to the resulting seed
    Timer(TimerDefinition),
    /// A pickup to add to the resulting seed
    Pickup(VPickup),
}

#[derive(Debug, Clone)]
/// Header-specific commands that influence seed generation, but won't be added to the resulting seed
pub enum HeaderCommand {
    Include { name: String },
    Exclude { name: String },
    Add { item: VItem, amount: V<i32> },
    Remove { item: VItem, amount: V<i32> },
    Name { item: VItem, name: V<String> },
    Display { item: VItem, name: V<String> },
    Price { item: VItem, price: V<u32> },
    Icon { item: VItem, icon: Icon },
    Parameter { identifier: String, default: ParameterDefault },
    Set { state: String },
    If { parameter: String, value: String },
    EndIf,
}

/// Type and value of a parameter's default
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterDefaultKind {
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
    pub fn kind(&self) -> ParameterDefaultKind {
        match self {
            ParameterDefault::Bool(_) => ParameterDefaultKind::Bool,
            ParameterDefault::Int(_) => ParameterDefaultKind::Int,
            ParameterDefault::Float(_) => ParameterDefaultKind::Float,
            ParameterDefault::String(_) => ParameterDefaultKind::String,
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

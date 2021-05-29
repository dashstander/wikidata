use crate::ids::{consts, Fid, Lid, Pid, Qid, Sid};
use crate::text::{Lang, Text};
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A Wikibase entity: this could be an entity, property, or lexeme.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// All of the claims on the entity.
    pub claims: Vec<(Pid, ClaimValue)>,
    /// The type of the entity.
    pub entity_type: EntityType,
    /// All of the descriptions in all known languages.
    pub description: Vec<Text>,
    /// All of the labels in all known languages.
    pub labels: Vec<Text>,
}

/// The type of entity: normal entity with a Qid, a property with a Pid, or a lexeme with a Lid.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EntityType {
    /// An entity with a Qid.
    Entity,
    /// An entity with a Pid.
    Property,
    /// An entity with a Lid.
    Lexeme,
}

/// Data relating to a claim value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClaimValueData {
    /// The ID of a file on Wikimedia Commons.
    CommonsMedia(String),
    /// Coordinates on some globe.
    GlobeCoordinate {
        /// Latitude.
        lat: f64,
        /// Longitude.
        lon: f64,
        /// How many degrees of distance of precision there are.
        precision: f64,
        /// The globe the coordnaties are on, usually usually Q2 for Earth.
        globe: Qid,
    },
    /// A Wikidata item.
    Item(Qid),
    /// A Wikidata property.
    Property(Pid),
    /// A language-less string of text.
    String(String),
    /// Text with a language.
    MonolingualText(Text),
    /// The same text, translated across multiple languages.
    MultilingualText(Vec<Text>),
    /// An external identifier.
    ExternalID(String),
    /// Some numeric quantity of something.
    Quantity {
        /// How much.
        amount: f64, // technically it could exceed the bound, but meh
        /// The lowest possible value. If this isn't present then it is exactly the amount.
        lower_bound: Option<f64>,
        /// The highest possible value. If this isn't present then it is exactly the amount.
        upper_bound: Option<f64>,
        /// The units used.
        unit: Option<Qid>, // *could* be any IRI but in practice almost all are Wikidata entity IRIs
    },
    /// A point in time time.
    DateTime {
        /// The time as a Chrono DateTime.
        date_time: DateTime<chrono::offset::Utc>,
        /// The precision of the date:
        ///
        /// | precision | time |
        /// | --------- | ---- |
        /// | `0` | 1 billion years |
        /// | `1` | 100 million years |
        /// | `2` | 10 million years |
        /// | `3` | 1 million years |
        /// | `4` | 100k years |
        /// | `5` | 10k years |
        /// | `6` | 1000 years |
        /// | `7` | 100 years |
        /// | `8` | decade |
        /// | `9` | year |
        /// | `10` | month |
        /// | `11` | day |
        /// | `12` | hour (deprecated) |
        /// | `13` | minute (deprecated) |
        /// | `14` | second (deprecated) |
        precision: u8,
    },
    /// A URL.
    Url(String),
    /// A LaTeX math expression.
    MathExpr(String),
    /// A geometric shape. The value of the string is currently unspecified.
    GeoShape(String),
    /// LilyPond musical notation.
    MusicNotation(String),
    /// ID of a file with tabular data on Wikimedia commons.
    TabularData(String),
    /// A lexeme ID on Wikidata.
    Lexeme(Lid),
    /// A form ID on Wikidata.
    Form(Fid),
    /// A sense ID on Wikidata.
    Sense(Sid),
    /// No value.
    NoValue,
    /// Unknown value.
    UnknownValue,
}

impl Default for ClaimValueData {
    fn default() -> Self {
        ClaimValueData::NoValue
    }
}

/// A statement rank.
#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Rank {
    /// The deprecated rank, indicating outdated/wrong info.
    Deprecated,
    /// Normal rank, the default.
    Normal,
    /// Preferred rank, indicates the claim is most recent or accurate.
    Preferred,
}

impl Default for Rank {
    fn default() -> Self {
        Rank::Normal
    }
}

/// A group of claims that make up a single reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceGroup {
    /// All of the claims.
    pub claims: Vec<(Pid, ClaimValueData)>,
}

/// A claim value.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClaimValue {
    /// The data of the claim.
    pub data: ClaimValueData,
    /// The rank of this claim.
    pub rank: Rank,
    /// The globally unique claim ID.
    pub id: String,
    /// All of the qualifiers for this claim.
    pub qualifiers: Vec<(Pid, ClaimValueData)>,
    /// All of the groups of references for this claim.
    pub references: Vec<ReferenceGroup>,
}

impl Entity {
    /// All of the values of "instance of" on the entity.
    #[must_use]
    pub fn instances(&self) -> Vec<Qid> {
        let mut instances = Vec::with_capacity(1);
        for (pid, claim) in &self.claims {
            if *pid == consts::INSTANCE_OF {
                if let ClaimValueData::Item(qid) = claim.data {
                    instances.push(qid);
                };
            };
        }
        instances.shrink_to_fit();
        instances
    }

    /// When the entity started existing.
    #[must_use]
    pub fn start_time(&self) -> Option<DateTime<chrono::offset::Utc>> {
        for (pid, claim) in &self.claims {
            if *pid == consts::DATE_OF_BIRTH {
                if let ClaimValueData::DateTime { date_time, .. } = claim.data {
                    return Some(date_time);
                };
            };
        }
        None
    }

    /// When the entity stopped existing.
    #[must_use]
    pub fn end_time(&self) -> Option<DateTime<chrono::offset::Utc>> {
        for (pid, claim) in &self.claims {
            if *pid == consts::DATE_OF_DEATH {
                if let ClaimValueData::DateTime { date_time, .. } = claim.data {
                    return Some(date_time);
                };
            };
        }
        None
    }
}

/// An error related to entity parsing/creation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EntityError {
    /// A float couldn't be parsed.
    FloatParse,
    /// A string was expected but not found
    ExpectedString,
    /// Expected string repersenting number
    ExpectedNumberString,
    /// Expected string repersenting URI
    ExpectedUriString,
    /// A valid Qid URI was expected but not found
    ExpectedQidString,
    /// Expected a string because the datatype is string
    ExpectedStringDatatype,
    /// A time string was empty
    TimeEmpty,
    /// An ID was invalid
    BadId,
    /// A date didn't have a year
    NoDateYear,
    /// No date matched the day/month/year
    NoDateMatched,
    /// An ambiguous date was specified
    DateAmbiguous,
    /// The datatype was invalid
    InvalidDatatype,
    /// The datatype was invalid or unknown
    UnknownDatatype,
    /// The time was missing an hour
    MissingHour,
    /// The time was missing an minute
    MissingMinute,
    /// The time was missing an second
    MissingSecond,
    /// The snaktype was invalid
    InvalidSnaktype,
    /// The precision level was invalid
    InvalidPrecision,
    /// No rank was specified
    NoRank,
    /// A number was out of bounds
    NumberOutOfBounds,
    /// No ID was found
    NoId,
}

fn get_json_string(json: Value) -> Result<String, EntityError> {
    json.as_str()
        .map(ToString::to_string)
        .ok_or(EntityError::ExpectedString)
}

fn parse_wb_number(num: &Value) -> Result<f64, EntityError> {
    match num {
        Value::Number(num) => num.as_f64().ok_or(EntityError::NumberOutOfBounds),
        Value::String(s) => match s.parse() {
            Ok(x) => Ok(x),
            Err(_) => Err(EntityError::FloatParse),
        },
        _ => Err(EntityError::ExpectedNumberString),
    }
}

fn try_get_as_qid(datavalue: &Value) -> Result<Qid, EntityError> {
    match datavalue
        .as_str()
        .ok_or(EntityError::ExpectedUriString)?
        .split("http://www.wikidata.org/entity/Q")
        .nth(1)
        .ok_or(EntityError::ExpectedQidString)?
        .parse()
    {
        Ok(x) => Ok(Qid(x)),
        Err(_) => Err(EntityError::FloatParse),
    }
}

fn take_prop(key: &'static str, claim: &mut Value) -> Value {
    match claim.as_object_mut() {
        Some(obj) => obj.remove(key).unwrap_or(Value::Null),
        None => Value::Null,
    }
}

fn parse_wb_time(time: &str) -> Result<chrono::DateTime<chrono::offset::Utc>, EntityError> {
    if time.is_empty() {
        return Err(EntityError::TimeEmpty);
    }

    // "Negative years are allowed in formatting but not in parsing.", so we
    // set the era ourselves, after parsing
    let is_ce = time.chars().next().ok_or(EntityError::TimeEmpty)? == '+';
    let time = &time[1..];

    let time_parts: Vec<&str> = time.split('T').collect();
    let dash_parts: Vec<&str> = time_parts[0].split('-').collect();
    // could be wrong maybe if the percision is more than a year, meh
    let year: i32 = match dash_parts[0].parse() {
        Ok(x) => x,
        Err(_) => return Err(EntityError::NoDateYear),
    };
    let year: i32 = year * (if is_ce { 1 } else { -1 });
    let month: Option<u32> = match dash_parts.get(1) {
        Some(month_str) => match month_str.parse() {
            Ok(0) | Err(_) => None,
            Ok(x) => Some(x),
        },
        None => None,
    };
    let day: Option<u32> = match dash_parts.get(2) {
        Some(day_str) => match day_str.parse() {
            Ok(0) | Err(_) => None,
            Ok(x) => Some(x),
        },
        None => None,
    };
    let maybe_date = Utc.ymd_opt(year, month.unwrap_or(1), day.unwrap_or(1));
    let date = match maybe_date {
        chrono::offset::LocalResult::Single(date) => date,
        chrono::offset::LocalResult::None => return Err(EntityError::NoDateMatched),
        chrono::offset::LocalResult::Ambiguous(_, _) => return Err(EntityError::DateAmbiguous),
    };
    let (hour, min, sec) = if time_parts.len() == 2 {
        let colon_parts: Vec<&str> = time_parts[1].split(':').collect();
        let hour = match colon_parts.get(0).ok_or(EntityError::MissingHour)?.parse() {
            Ok(x) => x,
            Err(_) => return Err(EntityError::FloatParse),
        };
        let minute = match colon_parts
            .get(1)
            .ok_or(EntityError::MissingMinute)?
            .parse()
        {
            Ok(x) => x,
            Err(_) => return Err(EntityError::FloatParse),
        };
        let sec = match colon_parts.get(2).ok_or(EntityError::MissingSecond)?[0..2].parse() {
            Ok(x) => x,
            Err(_) => return Err(EntityError::FloatParse),
        };
        (hour, minute, sec)
    } else {
        (0, 0, 0)
    };
    Ok(date.and_hms(hour, min, sec))
}

impl ClaimValueData {
    /// Parses a snak.
    ///
    /// # Errors
    /// If the `snak` does not correspond to a valid snak, then an error will be returned.
    pub fn parse_snak(mut snak: Value) -> Result<Self, EntityError> {
        let mut datavalue: Value = take_prop("datavalue", &mut snak);
        let datatype: &str = &get_json_string(take_prop("datatype", &mut snak))?;
        let snaktype: &str = &get_json_string(take_prop("snaktype", &mut snak))?;
        match snaktype {
            "value" => {}
            "somevalue" => return Ok(ClaimValueData::UnknownValue),
            "novalue" => return Ok(ClaimValueData::NoValue),
            _ => return Err(EntityError::InvalidSnaktype),
        };
        let type_str = take_prop("type", &mut datavalue)
            .as_str()
            .ok_or(EntityError::InvalidSnaktype)?
            .to_string();
        let mut value = take_prop("value", &mut datavalue);
        match &type_str[..] {
            "string" => {
                let s = value
                    .as_str()
                    .ok_or(EntityError::ExpectedStringDatatype)?
                    .to_string();
                match datatype {
                    "string" => Ok(ClaimValueData::String(s)),
                    "commonsMedia" => Ok(ClaimValueData::CommonsMedia(s)),
                    "external-id" => Ok(ClaimValueData::ExternalID(s)),
                    "math" => Ok(ClaimValueData::MathExpr(s)),
                    "geo-shape" => Ok(ClaimValueData::GeoShape(s)),
                    "musical-notation" => Ok(ClaimValueData::MusicNotation(s)),
                    "tabular-data" => Ok(ClaimValueData::TabularData(s)),
                    "url" => Ok(ClaimValueData::Url(s)),
                    _ => Err(EntityError::InvalidDatatype),
                }
            }
            "wikibase-entityid" => {
                // the ID could be a entity, lexeme, property, form, or sense
                let id = get_json_string(take_prop("id", &mut value))?;
                match id.chars().next().ok_or(EntityError::BadId)? {
                    'Q' => Ok(ClaimValueData::Item(Qid(id[1..]
                        .parse()
                        .map_err(|_| EntityError::BadId)?))),
                    'P' => Ok(ClaimValueData::Property(Pid(id[1..]
                        .parse()
                        .map_err(|_| EntityError::BadId)?))),
                    'L' => {
                        // sense: "L1-S2", form: "L1-F2", lexeme: "L2"
                        let parts: Vec<&str> = id.split('-').collect();
                        match parts.len() {
                            1 => Ok(ClaimValueData::Lexeme(Lid(id[1..]
                                .parse()
                                .map_err(|_| EntityError::BadId)?))),
                            2 => match parts[1].chars().next().ok_or(EntityError::BadId)? {
                                'F' => Ok(ClaimValueData::Form(Fid(
                                    Lid(parts[0][1..].parse().map_err(|_| EntityError::BadId)?),
                                    parts[1][1..].parse().map_err(|_| EntityError::BadId)?,
                                ))),
                                'S' => Ok(ClaimValueData::Sense(Sid(
                                    Lid(parts[0][1..].parse().map_err(|_| EntityError::BadId)?),
                                    parts[1][1..].parse().map_err(|_| EntityError::BadId)?,
                                ))),
                                _ => Err(EntityError::BadId),
                            },
                            _ => Err(EntityError::BadId),
                        }
                    }
                    _ => Err(EntityError::BadId),
                }
            }
            "globecoordinate" => {
                Ok(ClaimValueData::GlobeCoordinate {
                    lat: parse_wb_number(&take_prop("latitude", &mut value))?,
                    lon: parse_wb_number(&take_prop("longitude", &mut value))?,
                    // altitude field is deprecated and we ignore it
                    precision: parse_wb_number(&take_prop("precision", &mut value))?,
                    // globe *can* be any IRI, but it practice it's almost always an entity URI
                    // so we return None if it doesn't match our expectations
                    globe: try_get_as_qid(&take_prop("globe", &mut value))?,
                })
            }
            "quantity" => Ok(ClaimValueData::Quantity {
                amount: parse_wb_number(&take_prop("amount", &mut value))?,
                upper_bound: parse_wb_number(&take_prop("upperBound", &mut value)).ok(),
                lower_bound: parse_wb_number(&take_prop("lowerBound", &mut value)).ok(),
                unit: try_get_as_qid(&take_prop("unit", &mut value)).ok(),
            }),
            "time" => Ok(ClaimValueData::DateTime {
                // our time parsing code can't handle a few edge cases (really old years), so we
                // just give up on parsing the snak if parse_wb_time returns None
                date_time: parse_wb_time(&get_json_string(take_prop("time", &mut value))?)?,
                precision: parse_wb_number(&take_prop("precision", &mut value))
                    .map_err(|_| EntityError::InvalidPrecision)? as u8,
            }),
            "monolingualtext" => Ok(ClaimValueData::MonolingualText(Text {
                text: get_json_string(take_prop("text", &mut value))?,
                lang: Lang(get_json_string(take_prop("language", &mut value))?),
            })),
            _ => Err(EntityError::UnknownDatatype),
        }
    }
}

impl ClaimValue {
    /// Try to parse a JSON claim to a claim value.
    #[must_use]
    pub fn get_prop_from_snak(mut claim: Value, skip_id: bool) -> Option<ClaimValue> {
        let rank = match take_prop("rank", &mut claim).as_str()? {
            "deprecated" => {
                return None;
            }
            "normal" => Rank::Normal,
            "preferred" => Rank::Preferred,
            _ => return None,
        };
        let mainsnak = take_prop("mainsnak", &mut claim);
        let data = ClaimValueData::parse_snak(mainsnak).ok()?;
        let references = if let Some(arr) = take_prop("references", &mut claim).as_array() {
            let mut v: Vec<ReferenceGroup> = Vec::with_capacity(arr.len());
            for reference_group in arr {
                let reference_group = reference_group.as_object()?;
                let mut claims = Vec::with_capacity(reference_group["snaks"].as_array()?.len());
                let snaks = reference_group["snaks"].as_object()?;
                for (pid, snak_group) in snaks.iter() {
                    for snak in snak_group.as_array()?.iter() {
                        // clone, meh
                        let owned_snak = snak.clone().take();
                        if let Ok(x) = ClaimValueData::parse_snak(owned_snak) {
                            claims.push((Pid(pid[1..].parse().ok()?), x))
                        }
                    }
                }
                v.push(ReferenceGroup { claims });
            }
            v
        } else {
            Vec::new()
        };
        let qualifiers_json = take_prop("qualifiers", &mut claim);
        let qualifiers = if qualifiers_json.is_object() {
            let mut v: Vec<(Pid, ClaimValueData)> = vec![];
            for (pid, claim_array_json) in qualifiers_json.as_object()?.iter() {
                // yep it's a clone, meh
                let mut claim_array = if let Value::Array(x) = claim_array_json.clone().take() {
                    x
                } else {
                    return None;
                };
                for claim in claim_array.drain(..) {
                    if let Ok(x) = ClaimValueData::parse_snak(claim) {
                        v.push((Pid(pid[1..].parse().ok()?), x))
                    }
                }
            }
            v
        } else {
            vec![]
        };
        Some(ClaimValue {
            rank,
            id: if skip_id {
                String::new()
            } else {
                take_prop("id", &mut claim).as_str()?.to_string()
            },
            data,
            references,
            qualifiers,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn time_parsing() {
        let valid_times = vec![
            "+2001-12-31T00:00:00Z",
            "+12346-12-31T00:00:00Z",
            "+311-12-31T00:00:00Z",
            "+1979-00-00T00:00:00Z",
            "-1979-00-00T00:00:00Z",
            "+2001-12-31T00:00:00Z",
            "+2001-12-31",
            "+2001-12",
            "-12561",
            "+311-12-31T12:34:56Z",
            "+311-12-31T23:45:42Z",
            // below are times that *should* work, but chrono doesn't accept
            // "-410000000-00-00T00:00:00Z",
        ];
        for time in valid_times {
            println!("Trying \"{}\"", time);
            assert!(match parse_wb_time(time) {
                Ok(val) => {
                    println!("Got {:#?}", val);
                    true
                }
                Err(_) => false,
            });
        }
    }

    #[test]
    fn as_qid_test() {
        let qid = try_get_as_qid(
            &serde_json::from_str(r#""http://www.wikidata.org/entity/Q1234567""#).unwrap(),
        );
        assert_eq!(qid, Ok(Qid(1234567)));
    }
}

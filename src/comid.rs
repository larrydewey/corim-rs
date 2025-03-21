// SPDX-License-Identifier: MIT

//! Concise Module Identifier (CoMID) Implementation
//!
//! This module implements the CoMID (Concise Module Identifier) data structure as defined in the
//! IETF CoRIM specification. CoMID provides a structured way to identify and describe software
//! modules using CBOR-encoded tags.
//!
//! # Key Components
//!
//! * [`ConciseMidTag`] - The main CoMID structure, tagged with CBOR tag 506
//! * [`TagIdentityMap`] - Identification information for a tag
//! * [`ComidEntityMap`] - Information about entities associated with the tag
//! * [`TriplesMap`] - Collection of triples describing module characteristics
//!
//! # Example
//!
//! Creating a basic CoMID tag with entity and identity information:
//!
//! ```rust
//! use corim_rs::{
//!     comid::{
//!         ConciseMidTag, TagIdentityMap, ComidEntityMap, TriplesMap,
//!         TagIdTypeChoice, ComidRoleTypeChoice
//!     },
//!     core::{Text, Tstr},
//! };
//!
//! // Create a tag identity
//! let tag_identity = TagIdentityMap {
//!     tag_id: TagIdTypeChoice::Tstr(Tstr::from("example-tag-id")),
//!     tag_version: Some(1u32),
//! };
//!
//! // Create an entity
//! let entity = ComidEntityMap {
//!     entity_name: Text::from("Example Corp"),
//!     reg_id: None,
//!     role: vec![ComidRoleTypeChoice::TagCreator],
//!     extension: None,
//! };
//!
//! // Create an empty triples map
//! let triples = TriplesMap {
//!     reference_triples: None,
//!     endorsed_triples: None,
//!     identity_triples: None,
//!     attest_key_triples: None,
//!     dependency_triples: None,
//!     membership_triples: None,
//!     coswid_triples: None,
//!     conditional_endorsement_series_triples: None,
//!     conditional_endorsement_triples: None,
//!     extension: None,
//! };
//!
//! // Create the CoMID tag
//! let comid = ConciseMidTag {
//!     language: None,
//!     tag_identity,
//!     entities: Some(vec![entity].into()),
//!     linked_tags: None,
//!     triples,
//!     extension: None,
//! };
//! ```
//!
//! # Features
//!
//! * CBOR-based serialization using tag 506
//! * Support for multiple entity roles
//! * Extensible triple records for various module characteristics
//! * Optional language support
//! * Linking between related tags
//!
//! # Architecture
//!
//! The module is structured around the main [`ConciseMidTag`] type, which contains:
//!
//! 1. Tag identity information via [`TagIdentityMap`]
//! 2. Associated entities via [`ComidEntityMap`]
//! 3. Optional links to related tags via [`LinkedTagMap`]
//! 4. Triple records describing the module via [`TriplesMap`]
//!
//! All components support optional extensions through [`ExtensionMap`] for future expandability.

use crate::{
    core::{NonEmptyVec, RawValueType, TaggedBytes},
    generate_tagged,
    triples::{EnvironmentMap, MeasuredElementTypeChoice, MeasurementMap, MeasurementValuesMap},
    AttestKeyTripleRecord, ComidError, ConditionalEndorsementSeriesTripleRecord,
    ConditionalEndorsementTripleRecord, CoswidTripleRecord, DomainDependencyTripleRecord,
    DomainMembershipTripleRecord, EndorsedTripleRecord, ExtensionMap, IdentityTripleRecord,
    ReferenceTripleRecord, Result, Text, Tstr, Uint, Uri, UuidType,
};
use derive_more::{Constructor, From, TryFrom};
use serde::{Deserialize, Serialize};

/// A tag version number represented as an unsigned integer
pub type TagVersionType = Uint;

generate_tagged!((
    506,
    TaggedConciseMidTag,
    ConciseMidTag<'a>,
    'a,
    "A Concise Module Identifier (CoMID) structured tag"
),);
/// A Concise Module Identifier (CoMID) tag structure tagged with CBOR tag 506
#[derive(
    Debug, Serialize, Deserialize, From, Constructor, PartialEq, Eq, PartialOrd, Ord, Clone,
)]
#[repr(C)]
pub struct ConciseMidTag<'a> {
    /// Optional language identifier for the tag content
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "0")]
    pub language: Option<Text<'a>>,
    /// Identity information for this tag
    #[serde(rename = "1")]
    pub tag_identity: TagIdentityMap<'a>,
    /// List of entities associated with this tag
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "2")]
    pub entities: Option<NonEmptyVec<ComidEntityMap<'a>>>,
    /// Optional references to other related tags
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "3")]
    pub linked_tags: Option<NonEmptyVec<LinkedTagMap<'a>>>,
    /// Collection of triples describing the module
    #[serde(rename = "4")]
    pub triples: TriplesMap<'a>,
    /// Optional extensible attributes
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub extension: Option<ExtensionMap<'a>>,
}

impl<'a> ConciseMidTag<'a> {
    /// Adds a reference value to the CoMID tag's reference triples.
    ///
    /// This method serializes the provided value to CBOR bytes and adds it as a raw measurement value
    /// within a reference triple. If a reference triple with the same environment already exists,
    /// the measurement is added to that triple. Otherwise, a new reference triple is created.
    ///
    /// # Arguments
    ///
    /// * `environment` - The environment map that describes the context for this reference value
    /// * `mkey` - Optional measurement element type that identifies what is being measured
    /// * `value` - The value to serialize and store as the reference value
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if successful, or an `std::io::Error` if serialization fails.
    ///
    /// # Example
    ///
    /// ``` ignore
    /// use corim_rs::{
    ///     comid::{ConciseMidTag, TagIdentityMap, ComidEntityMap, TriplesMap, TagIdTypeChoice},
    ///     core::{OneOrMore, Text, Tstr},
    ///     triples::{EnvironmentMap, MeasuredElementTypeChoice},
    /// };
    ///
    /// let mut comid = ConciseMidTag {
    ///     language: None,
    ///     tag_identity: TagIdentityMap {
    ///         tag_id: TagIdTypeChoice::Tstr(Tstr::from("example-id")),
    ///         tag_version: Some(1),
    ///     },
    ///     entities: OneOrMore::One(ComidEntityMap {
    ///         entity_name: Text::from("Example Corp"),
    ///         reg_id: None,
    ///         role: OneOrMore::One(corim_rs::comid::ComidRoleTypeChoice::TagCreator),
    ///         extension: None,
    ///     }),
    ///     linked_tags: None,
    ///     triples: TriplesMap::default(),
    ///     extension: None,
    /// };
    ///
    /// // Add a reference value
    /// let env = EnvironmentMap::default();
    /// let reference_data = "example reference value";
    /// comid.add_reference_value(env, None, &reference_data).expect("Failed to add reference value");
    /// ```
    ///
    /// # How It Works
    ///
    /// 1. The value is serialized to CBOR bytes using the `ciborium` library
    /// 2. The bytes are wrapped in a `TaggedBytes` structure
    /// 3. A `MeasurementMap` is created with the provided measurement key and the raw value
    /// 4. The method then updates the CoMID's reference triples based on existing data:
    ///    - If no reference triples exist, a new one is created
    ///    - If a reference triple with the matching environment exists, the measurement is added to it
    ///    - If reference triples exist but none match the environment, a new triple is added
    pub fn add_reference_raw_value<T>(
        &mut self,
        environment: &EnvironmentMap<'a>,
        mkey: MeasuredElementTypeChoice<'a>,
        value: &T,
    ) -> std::io::Result<()>
    where
        T: ?Sized + Serialize,
    {
        let mut raw_bytes = vec![];
        ciborium::into_writer(value, &mut raw_bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        let raw_value = TaggedBytes::new(raw_bytes.into());

        let measurement = MeasurementMap {
            mkey: Some(mkey),
            mval: MeasurementValuesMap {
                raw: Some(RawValueType {
                    raw_value: raw_value.into(),
                    raw_value_mask: None,
                }),
                ..Default::default()
            },
            authorized_by: None,
        };

        match &mut self.triples.reference_triples {
            None => {
                let new_record = ReferenceTripleRecord {
                    ref_env: environment.clone(),
                    ref_claims: vec![measurement].into(),
                };
                self.triples.reference_triples = Some(vec![new_record].into());
            }
            Some(vec) => {
                if let Some(record) = vec.iter_mut().find(|r| r.ref_env == *environment) {
                    record.ref_claims.push(measurement);
                } else {
                    let new_record = ReferenceTripleRecord {
                        ref_env: environment.clone(),
                        ref_claims: vec![measurement].into(),
                    };
                    vec.push(new_record);
                }
            }
        }
        Ok(())
    }
    pub fn add_endorsement_raw_value<T>(
        &mut self,
        environment: &EnvironmentMap<'a>,
        mkey: MeasuredElementTypeChoice<'a>,
        value: &T,
    ) -> std::io::Result<()>
    where
        T: ?Sized + Serialize,
    {
        let mut raw_bytes = vec![];
        ciborium::into_writer(value, &mut raw_bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        let raw_value = TaggedBytes::new(raw_bytes.into());

        let measurement = MeasurementMap {
            mkey: Some(mkey),
            mval: MeasurementValuesMap {
                raw: Some(RawValueType {
                    raw_value: raw_value.into(),
                    raw_value_mask: None,
                }),
                ..Default::default()
            },
            authorized_by: None,
        };

        match &mut self.triples.endorsed_triples {
            None => {
                let new_record = EndorsedTripleRecord {
                    condition: environment.clone(),
                    endorsement: vec![measurement].into(),
                };
                self.triples.endorsed_triples = Some(vec![new_record].into());
            }

            Some(vec) => {
                if let Some(record) = vec.iter_mut().find(|r| r.condition == *environment) {
                    record.endorsement.push(measurement);
                } else {
                    let new_record = EndorsedTripleRecord {
                        condition: environment.clone(),
                        endorsement: vec![measurement].into(),
                    };
                    vec.push(new_record);
                }
            }
        }
        Ok(())
    }
}

/// Identification information for a tag
#[derive(
    Debug, Serialize, Deserialize, From, Constructor, PartialEq, Eq, PartialOrd, Ord, Clone,
)]
#[repr(C)]
pub struct TagIdentityMap<'a> {
    /// Unique identifier for the tag
    #[serde(rename = "0")]
    pub tag_id: TagIdTypeChoice<'a>,
    /// Optional version number for the tag
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "1")]
    pub tag_version: Option<TagVersionType>,
}

/// Represents either a string or UUID tag identifier
#[derive(Debug, Serialize, Deserialize, From, TryFrom, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[repr(C)]
#[serde(untagged)]
pub enum TagIdTypeChoice<'a> {
    /// Text string identifier
    Tstr(Tstr<'a>),
    /// UUID identifier
    Uuid(UuidType),
}


impl<'a> From<&'a str> for TagIdTypeChoice<'a> {
    fn from(value: &'a str) -> Self {
        TagIdTypeChoice::Tstr(Tstr::from(value))
    }
}

/// Information about an entity associated with the tag
#[derive(
    Debug, Serialize, Deserialize, From, Constructor, PartialEq, Eq, PartialOrd, Ord, Clone,
)]
#[repr(C)]
pub struct ComidEntityMap<'a> {
    /// Name of the entity
    #[serde(rename = "31")]
    pub entity_name: Text<'a>,
    /// Optional registration identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "32")]
    pub reg_id: Option<Uri<'a>>,
    /// One or more roles this entity fulfills
    #[serde(rename = "33")]
    pub role: Vec<ComidRoleTypeChoice>,
    /// Optional extensible attributes
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub extension: Option<ExtensionMap<'a>>,
}

/// Role types that can be assigned to entities
#[derive(Debug, Serialize, Deserialize, From, TryFrom, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[repr(C)]
#[serde(untagged)]
pub enum ComidRoleTypeChoice {
    /// Entity that created the tag (value: 0)
    TagCreator = 0,
    /// Entity that created the module (value: 1)
    Creator = 1,
    /// Entity that maintains the module (value: 2)
    Maintainer = 2,
}

/// Reference to another tag and its relationship to this one
#[derive(
    Debug, Serialize, Deserialize, From, Constructor, PartialEq, Eq, PartialOrd, Ord, Clone,
)]
#[repr(C)]
pub struct LinkedTagMap<'a> {
    /// Identifier of the linked tag
    #[serde(rename = "0")]
    pub linked_tag_id: TagIdTypeChoice<'a>,
    /// Relationship type between the tags
    #[serde(rename = "1")]
    pub tag_rel: TagRelTypeChoice,
}

/// Types of relationships between tags
#[derive(Debug, Serialize, Deserialize, From, TryFrom, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[repr(C)]
#[serde(untagged)]
pub enum TagRelTypeChoice {
    /// This tag supplements the linked tag by providing additional information
    /// without replacing or invalidating the linked tag's content
    Supplements,
    /// This tag completely replaces the linked tag, indicating that the linked
    /// tag should no longer be considered valid or current
    Replaces,
}

/// Collection of different types of triples describing the module characteristics. It is
/// **HIGHLY** recommended to use the TriplesMapBuilder, to ensure the CDDL enforcement of
/// at least one field being present.
#[derive(Default, Debug, Serialize, Deserialize, From, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[repr(C)]
pub struct TriplesMap<'a> {
    /// Optional reference triples that link to external references
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "0")]
    pub reference_triples: Option<NonEmptyVec<ReferenceTripleRecord<'a>>>,

    /// Optional endorsement triples that contain verification information
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "1")]
    pub endorsed_triples: Option<NonEmptyVec<EndorsedTripleRecord<'a>>>,

    /// Optional identity triples that provide identity information
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "2")]
    pub identity_triples: Option<NonEmptyVec<IdentityTripleRecord<'a>>>,

    /// Optional attestation key triples containing cryptographic keys
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "3")]
    pub attest_key_triples: Option<NonEmptyVec<AttestKeyTripleRecord<'a>>>,

    /// Optional domain dependency triples describing relationships between domains
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "4")]
    pub dependency_triples: Option<NonEmptyVec<DomainDependencyTripleRecord<'a>>>,

    /// Optional domain membership triples describing domain associations
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "5")]
    pub membership_triples: Option<NonEmptyVec<DomainMembershipTripleRecord<'a>>>,

    /// Optional SWID triples containing software identification data
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "6")]
    pub coswid_triples: Option<NonEmptyVec<CoswidTripleRecord<'a>>>,

    /// Optional conditional endorsement series triples for complex endorsement chains
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "8")]
    pub conditional_endorsement_series_triples:
        Option<NonEmptyVec<ConditionalEndorsementSeriesTripleRecord<'a>>>,

    /// Optional conditional endorsement triples for conditional verification
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "10")]
    pub conditional_endorsement_triples:
        Option<NonEmptyVec<ConditionalEndorsementTripleRecord<'a>>>,

    /// Optional extensible attributes for future expansion
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub extension: Option<ExtensionMap<'a>>,
}

#[derive(Default)]
pub struct TriplesMapBuilder<'a> {
    reference_triples: Option<NonEmptyVec<ReferenceTripleRecord<'a>>>,
    endorsed_triples: Option<NonEmptyVec<EndorsedTripleRecord<'a>>>,
    identity_triples: Option<NonEmptyVec<IdentityTripleRecord<'a>>>,
    attest_key_triples: Option<NonEmptyVec<AttestKeyTripleRecord<'a>>>,
    dependency_triples: Option<NonEmptyVec<DomainDependencyTripleRecord<'a>>>,
    membership_triples: Option<NonEmptyVec<DomainMembershipTripleRecord<'a>>>,
    coswid_triples: Option<NonEmptyVec<CoswidTripleRecord<'a>>>,
    conditional_endorsement_series_triples:
        Option<NonEmptyVec<ConditionalEndorsementSeriesTripleRecord<'a>>>,
    conditional_endorsement_triples: Option<NonEmptyVec<ConditionalEndorsementTripleRecord<'a>>>,
    extension: Option<ExtensionMap<'a>>,
}

impl<'a> TriplesMapBuilder<'a> {
    // Setter methods for each field
    pub fn reference_triples(mut self, value: NonEmptyVec<ReferenceTripleRecord<'a>>) -> Self {
        self.reference_triples = Some(value);
        self
    }

    pub fn endorsed_triples(mut self, value: NonEmptyVec<EndorsedTripleRecord<'a>>) -> Self {
        self.endorsed_triples = Some(value);
        self
    }

    pub fn identity_triples(mut self, value: NonEmptyVec<IdentityTripleRecord<'a>>) -> Self {
        self.identity_triples = Some(value);
        self
    }

    pub fn attest_key_triples(mut self, value: NonEmptyVec<AttestKeyTripleRecord<'a>>) -> Self {
        self.attest_key_triples = Some(value);
        self
    }

    pub fn dependency_triples(
        mut self,
        value: NonEmptyVec<DomainDependencyTripleRecord<'a>>,
    ) -> Self {
        self.dependency_triples = Some(value);
        self
    }

    pub fn membership_triples(
        mut self,
        value: NonEmptyVec<DomainMembershipTripleRecord<'a>>,
    ) -> Self {
        self.membership_triples = Some(value);
        self
    }

    pub fn coswid_triples(mut self, value: NonEmptyVec<CoswidTripleRecord<'a>>) -> Self {
        self.coswid_triples = Some(value);
        self
    }

    pub fn conditional_endorsement_series_triples(
        mut self,
        value: NonEmptyVec<ConditionalEndorsementSeriesTripleRecord<'a>>,
    ) -> Self {
        self.conditional_endorsement_series_triples = Some(value);
        self
    }

    pub fn conditional_endorsement_triples(
        mut self,
        value: NonEmptyVec<ConditionalEndorsementTripleRecord<'a>>,
    ) -> Self {
        self.conditional_endorsement_triples = Some(value);
        self
    }

    pub fn extension(mut self, value: ExtensionMap<'a>) -> Self {
        self.extension = Some(value);
        self
    }

    /// Builds the TriplesMap, returning an error if no fields are set
    pub fn build(self) -> Result<TriplesMap<'a>> {
        if self.reference_triples.is_none()
            && self.endorsed_triples.is_none()
            && self.identity_triples.is_none()
            && self.attest_key_triples.is_none()
            && self.dependency_triples.is_none()
            && self.membership_triples.is_none()
            && self.coswid_triples.is_none()
            && self.conditional_endorsement_series_triples.is_none()
            && self.conditional_endorsement_triples.is_none()
            && self.extension.is_none()
        {
            return Err(ComidError::EmptyTriplesMap)?;
        }

        Ok(TriplesMap {
            reference_triples: self.reference_triples,
            endorsed_triples: self.endorsed_triples,
            identity_triples: self.identity_triples,
            attest_key_triples: self.attest_key_triples,
            dependency_triples: self.dependency_triples,
            membership_triples: self.membership_triples,
            coswid_triples: self.coswid_triples,
            conditional_endorsement_series_triples: self.conditional_endorsement_series_triples,
            conditional_endorsement_triples: self.conditional_endorsement_triples,
            extension: self.extension,
        })
    }
}

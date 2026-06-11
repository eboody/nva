use bon::Builder;
use serde::{Deserialize, Deserializer, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("storage codec error")]
    Codec(#[from] CodecError),
    #[error("{record:?} storage shape mismatch: {reason:?}")]
    StorageShapeMismatch {
        record: RecordKind,
        reason: ShapeMismatchReason,
    },
    #[error("domain value rejected storage field {field:?}: {reason}")]
    InvalidDomainValue { field: StorageField, reason: String },
}

#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error("failed to decode json: {source}")]
    JsonDecode { source: serde_json::Error },
    #[error("failed to encode json: {source}")]
    JsonEncode { source: serde_json::Error },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordKind {
    PetResortPortfolio,
    ServiceOffering,
    CoreServiceContracts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeMismatchReason {
    RequiredFieldMissing,
    FieldBelongsToDifferentVariant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageField {
    ResortCount,
    BrandName,
    GroomingCadenceWeeks,
    TrainingProgramDurationWeeks,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct PetResortPortfolioRecord {
    pub operator: OperatorCode,
    pub resort_count: StoredResortCount,
    pub structure: PortfolioStructureCode,
    pub business_lines: Vec<BusinessLineCode>,
    pub brands: Vec<PetResortBrandRecord>,
}

impl PetResortPortfolioRecord {
    pub fn decode_json(raw: &str) -> Result<Self> {
        serde_json::from_str(raw).map_err(|source| CodecError::JsonDecode { source }.into())
    }

    pub fn encode_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|source| CodecError::JsonEncode { source }.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperatorCode {
    #[serde(rename = "nva")]
    NationalVeterinaryAssociates,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PortfolioStructureCode {
    FederatedMultiBrand,
    SingleBrand,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BusinessLineCode {
    GeneralPracticeVeterinaryHospitals,
    PetResorts,
    Equine,
    SpecialtyEmergencyHospitals,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PetResortBrandRecord {
    Known { code: PetResortBrandCode },
    Other { name: StoredBrandName },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PetResortBrandCode {
    NvaPetResorts,
    PetSuites,
    PoochHotel,
    EliteSuites,
    TheBarkSide,
    WoofdorfAstoria,
    DoggieDistrict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct StoredResortCount(u16);

impl StoredResortCount {
    pub const fn try_new(value: u16) -> std::result::Result<Self, StoredResortCountError> {
        if value == 0 {
            return Err(StoredResortCountError::ZeroResorts);
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> u16 {
        self.0
    }
}

impl<'de> Deserialize<'de> for StoredResortCount {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(u16::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum StoredResortCountError {
    #[error("stored pet resort portfolios require at least one resort")]
    ZeroResorts,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct StoredBrandName(String);

impl StoredBrandName {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        let value = value.into().trim().to_owned();
        if value.is_empty() {
            return Err(Error::InvalidDomainValue {
                field: StorageField::BrandName,
                reason: "brand name cannot be empty".to_owned(),
            });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<PetResortPortfolioRecord> for domain::operations::PetResortPortfolio {
    type Error = Error;

    fn try_from(record: PetResortPortfolioRecord) -> Result<Self> {
        Ok(Self::builder()
            .operator(record.operator.into())
            .resort_count(record.resort_count.try_into()?)
            .structure(record.structure.into())
            .business_lines(record.business_lines.into_iter().map(Into::into).collect())
            .brands(
                record
                    .brands
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>>>()?,
            )
            .build())
    }
}

impl TryFrom<domain::operations::PetResortPortfolio> for PetResortPortfolioRecord {
    type Error = Error;

    fn try_from(domain_portfolio: domain::operations::PetResortPortfolio) -> Result<Self> {
        Ok(Self::builder()
            .operator(domain_portfolio.operator.into())
            .resort_count(domain_portfolio.resort_count.try_into()?)
            .structure(domain_portfolio.structure.into())
            .business_lines(
                domain_portfolio
                    .business_lines
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            )
            .brands(
                domain_portfolio
                    .brands
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>>>()?,
            )
            .build())
    }
}

impl From<OperatorCode> for domain::operations::Operator {
    fn from(value: OperatorCode) -> Self {
        match value {
            OperatorCode::NationalVeterinaryAssociates => Self::NationalVeterinaryAssociates,
        }
    }
}

impl From<domain::operations::Operator> for OperatorCode {
    fn from(value: domain::operations::Operator) -> Self {
        match value {
            domain::operations::Operator::NationalVeterinaryAssociates => {
                Self::NationalVeterinaryAssociates
            }
        }
    }
}

impl From<PortfolioStructureCode> for domain::operations::PortfolioStructure {
    fn from(value: PortfolioStructureCode) -> Self {
        match value {
            PortfolioStructureCode::FederatedMultiBrand => Self::FederatedMultiBrand,
            PortfolioStructureCode::SingleBrand => Self::SingleBrand,
            PortfolioStructureCode::Unknown => Self::Unknown,
        }
    }
}

impl From<domain::operations::PortfolioStructure> for PortfolioStructureCode {
    fn from(value: domain::operations::PortfolioStructure) -> Self {
        match value {
            domain::operations::PortfolioStructure::FederatedMultiBrand => {
                Self::FederatedMultiBrand
            }
            domain::operations::PortfolioStructure::SingleBrand => Self::SingleBrand,
            domain::operations::PortfolioStructure::Unknown => Self::Unknown,
        }
    }
}

impl From<BusinessLineCode> for domain::operations::BusinessLine {
    fn from(value: BusinessLineCode) -> Self {
        match value {
            BusinessLineCode::GeneralPracticeVeterinaryHospitals => {
                Self::GeneralPracticeVeterinaryHospitals
            }
            BusinessLineCode::PetResorts => Self::PetResorts,
            BusinessLineCode::Equine => Self::Equine,
            BusinessLineCode::SpecialtyEmergencyHospitals => Self::SpecialtyEmergencyHospitals,
        }
    }
}

impl From<domain::operations::BusinessLine> for BusinessLineCode {
    fn from(value: domain::operations::BusinessLine) -> Self {
        match value {
            domain::operations::BusinessLine::GeneralPracticeVeterinaryHospitals => {
                Self::GeneralPracticeVeterinaryHospitals
            }
            domain::operations::BusinessLine::PetResorts => Self::PetResorts,
            domain::operations::BusinessLine::Equine => Self::Equine,
            domain::operations::BusinessLine::SpecialtyEmergencyHospitals => {
                Self::SpecialtyEmergencyHospitals
            }
        }
    }
}

impl TryFrom<StoredResortCount> for domain::operations::ResortCount {
    type Error = Error;

    fn try_from(value: StoredResortCount) -> Result<Self> {
        domain::operations::ResortCount::try_new(value.get()).map_err(|err| {
            Error::InvalidDomainValue {
                field: StorageField::ResortCount,
                reason: err.to_string(),
            }
        })
    }
}

impl TryFrom<domain::operations::ResortCount> for StoredResortCount {
    type Error = Error;

    fn try_from(value: domain::operations::ResortCount) -> Result<Self> {
        Self::try_new(value.get()).map_err(|err| Error::InvalidDomainValue {
            field: StorageField::ResortCount,
            reason: err.to_string(),
        })
    }
}

impl TryFrom<PetResortBrandRecord> for domain::operations::PetResortBrand {
    type Error = Error;

    fn try_from(value: PetResortBrandRecord) -> Result<Self> {
        Ok(match value {
            PetResortBrandRecord::Known { code } => code.into(),
            PetResortBrandRecord::Other { name } => Self::Other {
                name: ::domain::location::Name::try_new(name.as_str()).map_err(|err| {
                    Error::InvalidDomainValue {
                        field: StorageField::BrandName,
                        reason: err.to_string(),
                    }
                })?,
            },
        })
    }
}

impl TryFrom<domain::operations::PetResortBrand> for PetResortBrandRecord {
    type Error = Error;

    fn try_from(value: domain::operations::PetResortBrand) -> Result<Self> {
        Ok(match value {
            domain::operations::PetResortBrand::NvaPetResorts => Self::Known {
                code: PetResortBrandCode::NvaPetResorts,
            },
            domain::operations::PetResortBrand::PetSuites => Self::Known {
                code: PetResortBrandCode::PetSuites,
            },
            domain::operations::PetResortBrand::PoochHotel => Self::Known {
                code: PetResortBrandCode::PoochHotel,
            },
            domain::operations::PetResortBrand::EliteSuites => Self::Known {
                code: PetResortBrandCode::EliteSuites,
            },
            domain::operations::PetResortBrand::TheBarkSide => Self::Known {
                code: PetResortBrandCode::TheBarkSide,
            },
            domain::operations::PetResortBrand::WoofdorfAstoria => Self::Known {
                code: PetResortBrandCode::WoofdorfAstoria,
            },
            domain::operations::PetResortBrand::DoggieDistrict => Self::Known {
                code: PetResortBrandCode::DoggieDistrict,
            },
            domain::operations::PetResortBrand::Other { name } => Self::Other {
                name: StoredBrandName::try_new(name.into_inner())?,
            },
        })
    }
}

impl From<PetResortBrandCode> for domain::operations::PetResortBrand {
    fn from(value: PetResortBrandCode) -> Self {
        match value {
            PetResortBrandCode::NvaPetResorts => Self::NvaPetResorts,
            PetResortBrandCode::PetSuites => Self::PetSuites,
            PetResortBrandCode::PoochHotel => Self::PoochHotel,
            PetResortBrandCode::EliteSuites => Self::EliteSuites,
            PetResortBrandCode::TheBarkSide => Self::TheBarkSide,
            PetResortBrandCode::WoofdorfAstoria => Self::WoofdorfAstoria,
            PetResortBrandCode::DoggieDistrict => Self::DoggieDistrict,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct ServiceOfferingRecord {
    pub service_kind: ServiceOfferingKindCode,
    pub boarding_accommodation: Option<BoardingAccommodationCode>,
    #[builder(default)]
    pub boarding_included_care: Vec<BoardingCareFeatureCode>,
    #[builder(default)]
    pub boarding_add_ons: Vec<BoardingAddOnCode>,
    pub daycare_format: Option<DaycareFormatCode>,
    #[builder(default)]
    pub daycare_eligibility_rules: Vec<DaycareEligibilityRuleCode>,
    pub grooming_service: Option<GroomingServiceCode>,
    pub grooming_cadence_weeks: Option<StoredCadenceWeeks>,
    pub training_program: Option<TrainingProgramRecord>,
    pub retail_partner: Option<RetailPartnerCode>,
    pub retail_product_category: Option<RetailProductCategoryCode>,
}

impl ServiceOfferingRecord {
    pub fn decode_json(raw: &str) -> Result<Self> {
        serde_json::from_str(raw).map_err(|source| CodecError::JsonDecode { source }.into())
    }

    pub fn encode_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|source| CodecError::JsonEncode { source }.into())
    }

    fn mismatch(reason: ShapeMismatchReason) -> Error {
        Error::StorageShapeMismatch {
            record: RecordKind::ServiceOffering,
            reason,
        }
    }

    fn ensure_empty_cross_variant_fields(&self, allowed: ServiceOfferingKindCode) -> Result<()> {
        let invalid = match allowed {
            ServiceOfferingKindCode::Boarding => {
                self.daycare_format.is_some()
                    || !self.daycare_eligibility_rules.is_empty()
                    || self.grooming_service.is_some()
                    || self.grooming_cadence_weeks.is_some()
                    || self.training_program.is_some()
                    || self.retail_partner.is_some()
                    || self.retail_product_category.is_some()
            }
            ServiceOfferingKindCode::Daycare => {
                self.boarding_accommodation.is_some()
                    || !self.boarding_included_care.is_empty()
                    || !self.boarding_add_ons.is_empty()
                    || self.grooming_service.is_some()
                    || self.grooming_cadence_weeks.is_some()
                    || self.training_program.is_some()
                    || self.retail_partner.is_some()
                    || self.retail_product_category.is_some()
            }
            ServiceOfferingKindCode::Grooming => {
                self.boarding_accommodation.is_some()
                    || !self.boarding_included_care.is_empty()
                    || !self.boarding_add_ons.is_empty()
                    || self.daycare_format.is_some()
                    || !self.daycare_eligibility_rules.is_empty()
                    || self.training_program.is_some()
                    || self.retail_partner.is_some()
                    || self.retail_product_category.is_some()
            }
            ServiceOfferingKindCode::Training => {
                self.boarding_accommodation.is_some()
                    || !self.boarding_included_care.is_empty()
                    || !self.boarding_add_ons.is_empty()
                    || self.daycare_format.is_some()
                    || !self.daycare_eligibility_rules.is_empty()
                    || self.grooming_service.is_some()
                    || self.grooming_cadence_weeks.is_some()
                    || self.retail_partner.is_some()
                    || self.retail_product_category.is_some()
            }
            ServiceOfferingKindCode::RetailPartnerProduct => {
                self.boarding_accommodation.is_some()
                    || !self.boarding_included_care.is_empty()
                    || !self.boarding_add_ons.is_empty()
                    || self.daycare_format.is_some()
                    || !self.daycare_eligibility_rules.is_empty()
                    || self.grooming_service.is_some()
                    || self.grooming_cadence_weeks.is_some()
                    || self.training_program.is_some()
            }
        };

        if invalid {
            Err(Self::mismatch(
                ShapeMismatchReason::FieldBelongsToDifferentVariant,
            ))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceOfferingKindCode {
    Boarding,
    Daycare,
    Grooming,
    Training,
    RetailPartnerProduct,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoardingAccommodationCode {
    ClassicSuite,
    LuxurySuite,
    CatCondo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoardingCareFeatureCode {
    DailyHousekeeping,
    PottyWalks,
    Bedding,
    PawgressReport,
    FeedingSupport,
    MedicationSupport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoardingAddOnCode {
    Playtime,
    ExitBath,
    PremiumSuite,
    Grooming,
    TrainingSession,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DaycareFormatCode {
    AllDayPlay,
    HalfDayPlay,
    DayBoarding,
    DayPlayPlusRoom,
    CatIndividualPlaytime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DaycareEligibilityRuleCode {
    TemperamentReviewRequired,
    SpayNeuterRequiredForGroupPlay,
    VaccineProofRequired,
    StaffToPetRatioRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroomingServiceCode {
    MiniGroom,
    FullGroom,
    ExitBath,
    FullBath,
    PremiumBath,
    NailTrim,
    NailDremel,
    EarCleaning,
    CoatSkinSpecificProduct,
    FirstTimeGroomingOffer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrainingProgramRecord {
    StayAndStudy {
        duration_weeks: StoredTrainingProgramDurationWeeks,
    },
    TutorSession,
    GroupClass,
    PuppyKindergarten,
    PrivateLesson,
    AkcCanineGoodCitizenPrep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetailPartnerCode {
    VirbacCalmCare,
    PurinaProPlanVeterinarySupplements,
    PurinaEnBoardingDiet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetailProductCategoryCode {
    Supplement,
    InHouseDiet,
    PersonalizedUpsell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct StoredCadenceWeeks(u8);

impl StoredCadenceWeeks {
    pub const fn try_new(value: u8) -> std::result::Result<Self, StoredCadenceWeeksError> {
        if value == 0 {
            return Err(StoredCadenceWeeksError::ZeroWeeks);
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> u8 {
        self.0
    }
}

impl<'de> Deserialize<'de> for StoredCadenceWeeks {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(u8::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum StoredCadenceWeeksError {
    #[error("stored grooming cadence requires at least one week")]
    ZeroWeeks,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct StoredTrainingProgramDurationWeeks(u8);

impl StoredTrainingProgramDurationWeeks {
    pub const fn try_new(
        value: u8,
    ) -> std::result::Result<Self, StoredTrainingProgramDurationWeeksError> {
        if value == 0 {
            return Err(StoredTrainingProgramDurationWeeksError::ZeroWeeks);
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> u8 {
        self.0
    }
}

impl<'de> Deserialize<'de> for StoredTrainingProgramDurationWeeks {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(u8::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum StoredTrainingProgramDurationWeeksError {
    #[error("stored training program duration requires at least one week")]
    ZeroWeeks,
}

impl TryFrom<domain::operations::ServiceOffering> for ServiceOfferingRecord {
    type Error = Error;

    fn try_from(value: domain::operations::ServiceOffering) -> Result<Self> {
        Ok(match value {
            domain::operations::ServiceOffering::Boarding {
                accommodation,
                included_care,
                add_ons,
            } => Self::builder()
                .service_kind(ServiceOfferingKindCode::Boarding)
                .boarding_accommodation(accommodation.into())
                .boarding_included_care(included_care.into_iter().map(Into::into).collect())
                .boarding_add_ons(add_ons.into_iter().map(Into::into).collect())
                .build(),
            domain::operations::ServiceOffering::Daycare {
                format,
                eligibility_rules,
            } => Self::builder()
                .service_kind(ServiceOfferingKindCode::Daycare)
                .daycare_format(format.into())
                .daycare_eligibility_rules(eligibility_rules.into_iter().map(Into::into).collect())
                .build(),
            domain::operations::ServiceOffering::Grooming { service, cadence } => {
                let cadence_weeks = match cadence {
                    domain::operations::GroomingCadence::EveryWeeks(weeks) => {
                        Some(weeks.try_into()?)
                    }
                    domain::operations::GroomingCadence::AsNeeded
                    | domain::operations::GroomingCadence::Unknown => None,
                };
                let builder = Self::builder()
                    .service_kind(ServiceOfferingKindCode::Grooming)
                    .grooming_service(service.into());
                match cadence_weeks {
                    Some(weeks) => builder.grooming_cadence_weeks(weeks).build(),
                    None => builder.build(),
                }
            }
            domain::operations::ServiceOffering::Training { program } => Self::builder()
                .service_kind(ServiceOfferingKindCode::Training)
                .training_program(program.try_into()?)
                .build(),
            domain::operations::ServiceOffering::RetailPartnerProduct { partner, category } => {
                Self::builder()
                    .service_kind(ServiceOfferingKindCode::RetailPartnerProduct)
                    .retail_partner(partner.into())
                    .retail_product_category(category.into())
                    .build()
            }
        })
    }
}

impl TryFrom<ServiceOfferingRecord> for domain::operations::ServiceOffering {
    type Error = Error;

    fn try_from(record: ServiceOfferingRecord) -> Result<Self> {
        match record.service_kind {
            ServiceOfferingKindCode::Boarding => {
                record.ensure_empty_cross_variant_fields(ServiceOfferingKindCode::Boarding)?;
                Ok(Self::Boarding {
                    accommodation: record
                        .boarding_accommodation
                        .ok_or_else(|| {
                            ServiceOfferingRecord::mismatch(
                                ShapeMismatchReason::RequiredFieldMissing,
                            )
                        })?
                        .into(),
                    included_care: record
                        .boarding_included_care
                        .into_iter()
                        .map(Into::into)
                        .collect(),
                    add_ons: record
                        .boarding_add_ons
                        .into_iter()
                        .map(Into::into)
                        .collect(),
                })
            }
            ServiceOfferingKindCode::Daycare => {
                record.ensure_empty_cross_variant_fields(ServiceOfferingKindCode::Daycare)?;
                Ok(Self::Daycare {
                    format: record
                        .daycare_format
                        .ok_or_else(|| {
                            ServiceOfferingRecord::mismatch(
                                ShapeMismatchReason::RequiredFieldMissing,
                            )
                        })?
                        .into(),
                    eligibility_rules: record
                        .daycare_eligibility_rules
                        .into_iter()
                        .map(Into::into)
                        .collect(),
                })
            }
            ServiceOfferingKindCode::Grooming => {
                record.ensure_empty_cross_variant_fields(ServiceOfferingKindCode::Grooming)?;
                let service = record
                    .grooming_service
                    .ok_or_else(|| {
                        ServiceOfferingRecord::mismatch(ShapeMismatchReason::RequiredFieldMissing)
                    })?
                    .into();
                let cadence = match record.grooming_cadence_weeks {
                    Some(weeks) => {
                        domain::operations::GroomingCadence::EveryWeeks(weeks.try_into()?)
                    }
                    None => domain::operations::GroomingCadence::Unknown,
                };
                Ok(Self::Grooming { service, cadence })
            }
            ServiceOfferingKindCode::Training => {
                record.ensure_empty_cross_variant_fields(ServiceOfferingKindCode::Training)?;
                Ok(Self::Training {
                    program: record
                        .training_program
                        .ok_or_else(|| {
                            ServiceOfferingRecord::mismatch(
                                ShapeMismatchReason::RequiredFieldMissing,
                            )
                        })?
                        .try_into()?,
                })
            }
            ServiceOfferingKindCode::RetailPartnerProduct => {
                record.ensure_empty_cross_variant_fields(
                    ServiceOfferingKindCode::RetailPartnerProduct,
                )?;
                Ok(Self::RetailPartnerProduct {
                    partner: record
                        .retail_partner
                        .ok_or_else(|| {
                            ServiceOfferingRecord::mismatch(
                                ShapeMismatchReason::RequiredFieldMissing,
                            )
                        })?
                        .into(),
                    category: record
                        .retail_product_category
                        .ok_or_else(|| {
                            ServiceOfferingRecord::mismatch(
                                ShapeMismatchReason::RequiredFieldMissing,
                            )
                        })?
                        .into(),
                })
            }
        }
    }
}

impl TryFrom<domain::operations::CadenceWeeks> for StoredCadenceWeeks {
    type Error = Error;

    fn try_from(value: domain::operations::CadenceWeeks) -> Result<Self> {
        Self::try_new(value.get()).map_err(|err| Error::InvalidDomainValue {
            field: StorageField::GroomingCadenceWeeks,
            reason: err.to_string(),
        })
    }
}

impl TryFrom<StoredCadenceWeeks> for domain::operations::CadenceWeeks {
    type Error = Error;

    fn try_from(value: StoredCadenceWeeks) -> Result<Self> {
        domain::operations::CadenceWeeks::try_new(value.get()).map_err(|err| {
            Error::InvalidDomainValue {
                field: StorageField::GroomingCadenceWeeks,
                reason: err.to_string(),
            }
        })
    }
}

impl TryFrom<domain::operations::TrainingProgramDurationWeeks>
    for StoredTrainingProgramDurationWeeks
{
    type Error = Error;

    fn try_from(value: domain::operations::TrainingProgramDurationWeeks) -> Result<Self> {
        Self::try_new(value.get()).map_err(|err| Error::InvalidDomainValue {
            field: StorageField::TrainingProgramDurationWeeks,
            reason: err.to_string(),
        })
    }
}

impl TryFrom<StoredTrainingProgramDurationWeeks>
    for domain::operations::TrainingProgramDurationWeeks
{
    type Error = Error;

    fn try_from(value: StoredTrainingProgramDurationWeeks) -> Result<Self> {
        domain::operations::TrainingProgramDurationWeeks::try_new(value.get()).map_err(|err| {
            Error::InvalidDomainValue {
                field: StorageField::TrainingProgramDurationWeeks,
                reason: err.to_string(),
            }
        })
    }
}

impl TryFrom<domain::operations::TrainingProgram> for TrainingProgramRecord {
    type Error = Error;

    fn try_from(value: domain::operations::TrainingProgram) -> Result<Self> {
        Ok(match value {
            domain::operations::TrainingProgram::StayAndStudy { duration } => Self::StayAndStudy {
                duration_weeks: duration.try_into()?,
            },
            domain::operations::TrainingProgram::TutorSession => Self::TutorSession,
            domain::operations::TrainingProgram::GroupClass => Self::GroupClass,
            domain::operations::TrainingProgram::PuppyKindergarten => Self::PuppyKindergarten,
            domain::operations::TrainingProgram::PrivateLesson => Self::PrivateLesson,
            domain::operations::TrainingProgram::AkcCanineGoodCitizenPrep => {
                Self::AkcCanineGoodCitizenPrep
            }
        })
    }
}

impl TryFrom<TrainingProgramRecord> for domain::operations::TrainingProgram {
    type Error = Error;

    fn try_from(value: TrainingProgramRecord) -> Result<Self> {
        Ok(match value {
            TrainingProgramRecord::StayAndStudy { duration_weeks } => Self::StayAndStudy {
                duration: duration_weeks.try_into()?,
            },
            TrainingProgramRecord::TutorSession => Self::TutorSession,
            TrainingProgramRecord::GroupClass => Self::GroupClass,
            TrainingProgramRecord::PuppyKindergarten => Self::PuppyKindergarten,
            TrainingProgramRecord::PrivateLesson => Self::PrivateLesson,
            TrainingProgramRecord::AkcCanineGoodCitizenPrep => Self::AkcCanineGoodCitizenPrep,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreServiceContractsRecord {
    pub location_id: domain::entities::LocationId,
    pub boarding: domain::operations::boarding::Contract,
    pub daycare: domain::operations::daycare::Contract,
    pub grooming: domain::operations::grooming::Contract,
    pub training: domain::operations::training::Contract,
    pub retail: domain::operations::retail::Contract,
}

impl CoreServiceContractsRecord {
    pub const fn record_kind(&self) -> RecordKind {
        RecordKind::CoreServiceContracts
    }

    pub fn encode_json(&self) -> Result<String> {
        serde_json::to_string(self)
            .map_err(|source| Error::Codec(CodecError::JsonEncode { source }))
    }

    pub fn decode_json(raw: &str) -> Result<Self> {
        serde_json::from_str(raw).map_err(|source| Error::Codec(CodecError::JsonDecode { source }))
    }
}

impl From<domain::operations::CoreServiceContracts> for CoreServiceContractsRecord {
    fn from(contracts: domain::operations::CoreServiceContracts) -> Self {
        Self {
            location_id: contracts.location_id,
            boarding: contracts.boarding,
            daycare: contracts.daycare,
            grooming: contracts.grooming,
            training: contracts.training,
            retail: contracts.retail,
        }
    }
}

impl From<CoreServiceContractsRecord> for domain::operations::CoreServiceContracts {
    fn from(record: CoreServiceContractsRecord) -> Self {
        Self::builder()
            .location_id(record.location_id)
            .boarding(record.boarding)
            .daycare(record.daycare)
            .grooming(record.grooming)
            .training(record.training)
            .retail(record.retail)
            .build()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct TechnologyEcosystemRecord {
    pub core_portal: CoreOperatingSystemCode,
    pub data_access: Vec<DataAccessPatternCode>,
    pub adjacent_systems: Vec<AdjacentSystemCode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoreOperatingSystemCode {
    Gingr,
    MixedSystems,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataAccessPatternCode {
    Api,
    Webhook,
    DataExport,
    Warehouse,
    BusinessIntelligenceDashboard,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdjacentSystemCode {
    AvatureRecruiting,
    Ga4,
    Amplitude,
    GoogleTagManager,
    Hris,
    LaborScheduling,
    Payroll,
    MarketingAutomation,
    Ticketing,
    CallCenterTelephony,
    Reviews,
    EmailSmsMarketing,
    BusinessIntelligence,
    DataLake,
}

impl From<domain::operations::TechnologyEcosystem> for TechnologyEcosystemRecord {
    fn from(value: domain::operations::TechnologyEcosystem) -> Self {
        Self::builder()
            .core_portal(value.core_portal.into())
            .data_access(value.data_access.into_iter().map(Into::into).collect())
            .adjacent_systems(value.adjacent_systems.into_iter().map(Into::into).collect())
            .build()
    }
}

impl From<TechnologyEcosystemRecord> for domain::operations::TechnologyEcosystem {
    fn from(value: TechnologyEcosystemRecord) -> Self {
        Self::builder()
            .core_portal(value.core_portal.into())
            .data_access(value.data_access.into_iter().map(Into::into).collect())
            .adjacent_systems(value.adjacent_systems.into_iter().map(Into::into).collect())
            .build()
    }
}

macro_rules! bidirectional_code_map {
    ($storage:ty, $domain:ty, { $($storage_variant:ident => $domain_variant:ident),+ $(,)? }) => {
        impl From<$storage> for $domain {
            fn from(value: $storage) -> Self {
                match value {
                    $(<$storage>::$storage_variant => Self::$domain_variant,)+
                }
            }
        }

        impl From<$domain> for $storage {
            fn from(value: $domain) -> Self {
                match value {
                    $(<$domain>::$domain_variant => Self::$storage_variant,)+
                }
            }
        }
    };
}

bidirectional_code_map!(BoardingAccommodationCode, domain::operations::BoardingAccommodation, {
    ClassicSuite => ClassicSuite,
    LuxurySuite => LuxurySuite,
    CatCondo => CatCondo,
});

bidirectional_code_map!(BoardingCareFeatureCode, domain::operations::BoardingCareFeature, {
    DailyHousekeeping => DailyHousekeeping,
    PottyWalks => PottyWalks,
    Bedding => Bedding,
    PawgressReport => PawgressReport,
    FeedingSupport => FeedingSupport,
    MedicationSupport => MedicationSupport,
});

bidirectional_code_map!(BoardingAddOnCode, domain::operations::BoardingAddOn, {
    Playtime => Playtime,
    ExitBath => ExitBath,
    PremiumSuite => PremiumSuite,
    Grooming => Grooming,
    TrainingSession => TrainingSession,
});

bidirectional_code_map!(DaycareFormatCode, domain::operations::DaycareFormat, {
    AllDayPlay => AllDayPlay,
    HalfDayPlay => HalfDayPlay,
    DayBoarding => DayBoarding,
    DayPlayPlusRoom => DayPlayPlusRoom,
    CatIndividualPlaytime => CatIndividualPlaytime,
});

bidirectional_code_map!(DaycareEligibilityRuleCode, domain::operations::DaycareEligibilityRule, {
    TemperamentReviewRequired => TemperamentReviewRequired,
    SpayNeuterRequiredForGroupPlay => SpayNeuterRequiredForGroupPlay,
    VaccineProofRequired => VaccineProofRequired,
    StaffToPetRatioRequired => StaffToPetRatioRequired,
});

bidirectional_code_map!(GroomingServiceCode, domain::operations::GroomingService, {
    MiniGroom => MiniGroom,
    FullGroom => FullGroom,
    ExitBath => ExitBath,
    FullBath => FullBath,
    PremiumBath => PremiumBath,
    NailTrim => NailTrim,
    NailDremel => NailDremel,
    EarCleaning => EarCleaning,
    CoatSkinSpecificProduct => CoatSkinSpecificProduct,
    FirstTimeGroomingOffer => FirstTimeGroomingOffer,
});

bidirectional_code_map!(RetailPartnerCode, domain::operations::RetailPartner, {
    VirbacCalmCare => VirbacCalmCare,
    PurinaProPlanVeterinarySupplements => PurinaProPlanVeterinarySupplements,
    PurinaEnBoardingDiet => PurinaEnBoardingDiet,
});

bidirectional_code_map!(RetailProductCategoryCode, domain::operations::RetailProductCategory, {
    Supplement => Supplement,
    InHouseDiet => InHouseDiet,
    PersonalizedUpsell => PersonalizedUpsell,
});
bidirectional_code_map!(CoreOperatingSystemCode, domain::operations::CoreOperatingSystem, {
    Gingr => Gingr,
    MixedSystems => MixedSystems,
    Unknown => Unknown,
});

bidirectional_code_map!(DataAccessPatternCode, domain::operations::DataAccessPattern, {
    Api => Api,
    Webhook => Webhook,
    DataExport => DataExport,
    Warehouse => Warehouse,
    BusinessIntelligenceDashboard => BusinessIntelligenceDashboard,
    Unknown => Unknown,
});

bidirectional_code_map!(AdjacentSystemCode, domain::operations::AdjacentSystem, {
    AvatureRecruiting => AvatureRecruiting,
    Ga4 => Ga4,
    Amplitude => Amplitude,
    GoogleTagManager => GoogleTagManager,
    Hris => Hris,
    LaborScheduling => LaborScheduling,
    Payroll => Payroll,
    MarketingAutomation => MarketingAutomation,
    Ticketing => Ticketing,
    CallCenterTelephony => CallCenterTelephony,
    Reviews => Reviews,
    EmailSmsMarketing => EmailSmsMarketing,
    BusinessIntelligence => BusinessIntelligence,
    DataLake => DataLake,
});

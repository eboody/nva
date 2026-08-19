use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Storage shape for the pet-resort portfolio facts used to seed operating assumptions.
pub struct PetResortPortfolioRecord {
    /// Portfolio operator represented by the seed record.
    pub operator: OperatorCode,
    /// Number of resorts represented by the portfolio fact.
    pub resort_count: StoredResortCount,
    /// Portfolio organization model used in operating assumptions.
    pub structure: PortfolioStructureCode,
    /// Business lines included in the portfolio fact.
    pub business_lines: Vec<BusinessLineCode>,
    /// Pet-resort brands included in the portfolio fact.
    pub brands: Vec<PetResortBrandRecord>,
}

impl PetResortPortfolioRecord {}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
/// Stable operator code used in portfolio seed records.
pub enum OperatorCode {
    #[serde(rename = "nva")]
    #[strum(serialize = "nva")]
    /// Stable storage code for national veterinary associates.
    NationalVeterinaryAssociates,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
/// Stable portfolio-structure codes for pet-resort operating assumptions.
pub enum PortfolioStructureCode {
    /// Stable storage code for federated multi brand.
    FederatedMultiBrand,
    /// Stable storage code for single brand.
    SingleBrand,
    /// Provider supplied an unrecognized value; preserve it for audit instead of failing closed.
    Unknown,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
/// Stable business-line codes for NVA portfolio membership.
pub enum BusinessLineCode {
    /// Stable storage code for general practice veterinary hospitals.
    GeneralPracticeVeterinaryHospitals,
    /// Stable storage code for pet resorts.
    PetResorts,
    /// Stable storage code for equine.
    Equine,
    /// Stable storage code for specialty emergency hospitals.
    SpecialtyEmergencyHospitals,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
/// Stored pet-resort brand descriptor with code plus display name.
pub enum PetResortBrandRecord {
    /// Enumerated brand known to the pet-resort context pack.
    Known {
        /// Stable brand code promoted into a domain brand.
        code: PetResortBrandCode,
    },
    /// Non-enumerated brand preserved with a validated display name.
    Other {
        /// Validated display name for a brand not yet represented by a stable code.
        name: StoredBrandName,
    },
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
/// Stable brand codes for NVA pet-resort banners.
pub enum PetResortBrandCode {
    /// Stable storage code for nva pet resorts.
    NvaPetResorts,
    /// Stable storage code for pet suites.
    PetSuites,
    /// Stable storage code for pooch hotel.
    PoochHotel,
    /// Stable storage code for elite suites.
    EliteSuites,
    /// Stable storage code for the bark side.
    TheBarkSide,
    /// Stable storage code for woofdorf astoria.
    WoofdorfAstoria,
    /// Stable storage code for doggie district.
    DoggieDistrict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// Positive resort count persisted for portfolio seed facts.
pub struct StoredResortCount(u16);

impl StoredResortCount {
    /// Validates and wraps a positive quantity before it is persisted.
    pub const fn try_new(value: u16) -> std::result::Result<Self, StoredResortCountError> {
        if value == 0 {
            return Err(StoredResortCountError::ZeroResorts);
        }
        Ok(Self(value))
    }

    /// Returns the provider numeric identifier kept on this wrapper.
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
/// Validation failures for persisted resort-count quantities.
pub enum StoredResortCountError {
    #[error("stored pet resort portfolios require at least one resort")]
    /// Stable storage code for zero resorts.
    ZeroResorts,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Non-empty pet-resort brand display name persisted beside the stable brand code.
pub struct StoredBrandName(String);

impl StoredBrandName {
    /// Validates and wraps a positive storage quantity before persistence.
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

    /// Returns the normalized provider or storage string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<PetResortPortfolioRecord> for pet_resort::Portfolio {
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

impl TryFrom<pet_resort::Portfolio> for PetResortPortfolioRecord {
    type Error = Error;

    fn try_from(domain_portfolio: pet_resort::Portfolio) -> Result<Self> {
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

impl From<OperatorCode> for pet_resort::Operator {
    fn from(value: OperatorCode) -> Self {
        match value {
            OperatorCode::NationalVeterinaryAssociates => Self::NationalVeterinaryAssociates,
        }
    }
}

impl From<pet_resort::Operator> for OperatorCode {
    fn from(value: pet_resort::Operator) -> Self {
        match value {
            pet_resort::Operator::NationalVeterinaryAssociates => {
                Self::NationalVeterinaryAssociates
            }
        }
    }
}

impl From<PortfolioStructureCode> for pet_resort::PortfolioStructure {
    fn from(value: PortfolioStructureCode) -> Self {
        match value {
            PortfolioStructureCode::FederatedMultiBrand => Self::FederatedMultiBrand,
            PortfolioStructureCode::SingleBrand => Self::SingleBrand,
            PortfolioStructureCode::Unknown => Self::Unknown,
        }
    }
}

impl From<pet_resort::PortfolioStructure> for PortfolioStructureCode {
    fn from(value: pet_resort::PortfolioStructure) -> Self {
        match value {
            pet_resort::PortfolioStructure::FederatedMultiBrand => Self::FederatedMultiBrand,
            pet_resort::PortfolioStructure::SingleBrand => Self::SingleBrand,
            pet_resort::PortfolioStructure::Unknown => Self::Unknown,
        }
    }
}

impl From<BusinessLineCode> for pet_resort::BusinessLine {
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

impl From<pet_resort::BusinessLine> for BusinessLineCode {
    fn from(value: pet_resort::BusinessLine) -> Self {
        match value {
            pet_resort::BusinessLine::GeneralPracticeVeterinaryHospitals => {
                Self::GeneralPracticeVeterinaryHospitals
            }
            pet_resort::BusinessLine::PetResorts => Self::PetResorts,
            pet_resort::BusinessLine::Equine => Self::Equine,
            pet_resort::BusinessLine::SpecialtyEmergencyHospitals => {
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

impl TryFrom<PetResortBrandRecord> for pet_resort::Brand {
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

impl TryFrom<pet_resort::Brand> for PetResortBrandRecord {
    type Error = Error;

    fn try_from(value: pet_resort::Brand) -> Result<Self> {
        Ok(match value {
            pet_resort::Brand::NvaPetResorts => Self::Known {
                code: PetResortBrandCode::NvaPetResorts,
            },
            pet_resort::Brand::PetSuites => Self::Known {
                code: PetResortBrandCode::PetSuites,
            },
            pet_resort::Brand::PoochHotel => Self::Known {
                code: PetResortBrandCode::PoochHotel,
            },
            pet_resort::Brand::EliteSuites => Self::Known {
                code: PetResortBrandCode::EliteSuites,
            },
            pet_resort::Brand::TheBarkSide => Self::Known {
                code: PetResortBrandCode::TheBarkSide,
            },
            pet_resort::Brand::WoofdorfAstoria => Self::Known {
                code: PetResortBrandCode::WoofdorfAstoria,
            },
            pet_resort::Brand::DoggieDistrict => Self::Known {
                code: PetResortBrandCode::DoggieDistrict,
            },
            pet_resort::Brand::Other { name } => Self::Other {
                name: StoredBrandName::try_new(name.into_inner())?,
            },
        })
    }
}

impl From<PetResortBrandCode> for pet_resort::Brand {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "service_kind", rename_all = "snake_case", deny_unknown_fields)]
/// Canonical storage shape for one service-line offering.
///
/// Each variant owns exactly its meaningful fields, so cross-service combinations cannot be
/// constructed or deserialized.
pub enum ServiceOfferingRecord {
    /// Boarding accommodation with its included care and optional add-ons.
    Boarding {
        /// Accommodation code offered for boarding.
        accommodation: boarding::AccommodationCode,
        /// Care features included in the boarding offer.
        included_care: Vec<boarding::CareFeatureCode>,
        /// Optional add-ons available with the boarding offer.
        add_ons: Vec<boarding::AddOnCode>,
    },
    /// Daycare format and its eligibility requirements.
    Daycare {
        /// Daycare delivery format.
        format: daycare::FormatCode,
        /// Requirements that must be satisfied before daycare participation.
        eligibility_rules: Vec<daycare::EligibilityRuleCode>,
    },
    /// Grooming service and its complete cadence value.
    Grooming {
        /// Grooming service offered.
        service: grooming::ServiceCode,
        /// Complete cadence value for the grooming offer.
        cadence: grooming::CadenceRecord,
    },
    /// Training program.
    Training {
        /// Training program offered.
        program: training::ProgramRecord,
    },
    /// Retail partner product and merchandise category.
    RetailPartnerProduct {
        /// Partner supplying the retail product.
        partner: retail::PartnerCode,
        /// Merchandise category of the retail product.
        category: retail::ProductCategoryCode,
    },
}

impl ServiceOfferingRecord {}

impl TryFrom<domain::operations::ServiceOffering> for ServiceOfferingRecord {
    type Error = Error;

    fn try_from(value: domain::operations::ServiceOffering) -> Result<Self> {
        Ok(match value {
            domain::operations::ServiceOffering::Boarding {
                accommodation,
                included_care,
                add_ons,
            } => Self::Boarding {
                accommodation: accommodation.into(),
                included_care: included_care.into_iter().map(Into::into).collect(),
                add_ons: add_ons.into_iter().map(Into::into).collect(),
            },
            domain::operations::ServiceOffering::Daycare {
                format,
                eligibility_rules,
            } => Self::Daycare {
                format: format.into(),
                eligibility_rules: eligibility_rules.into_iter().map(Into::into).collect(),
            },
            domain::operations::ServiceOffering::Grooming { service, cadence } => Self::Grooming {
                service: service.into(),
                cadence: grooming::CadenceRecord::from_domain(cadence)?,
            },
            domain::operations::ServiceOffering::Training { program } => Self::Training {
                program: program.try_into()?,
            },
            domain::operations::ServiceOffering::RetailPartnerProduct { partner, category } => {
                Self::RetailPartnerProduct {
                    partner: partner.into(),
                    category: category.into(),
                }
            }
        })
    }
}

impl TryFrom<ServiceOfferingRecord> for domain::operations::ServiceOffering {
    type Error = Error;

    fn try_from(record: ServiceOfferingRecord) -> Result<Self> {
        Ok(match record {
            ServiceOfferingRecord::Boarding {
                accommodation,
                included_care,
                add_ons,
            } => Self::Boarding {
                accommodation: accommodation.into(),
                included_care: included_care.into_iter().map(Into::into).collect(),
                add_ons: add_ons.into_iter().map(Into::into).collect(),
            },
            ServiceOfferingRecord::Daycare {
                format,
                eligibility_rules,
            } => Self::Daycare {
                format: format.into(),
                eligibility_rules: eligibility_rules.into_iter().map(Into::into).collect(),
            },
            ServiceOfferingRecord::Grooming { service, cadence } => Self::Grooming {
                service: service.into(),
                cadence: cadence.into_domain()?,
            },
            ServiceOfferingRecord::Training { program } => Self::Training {
                program: program.try_into()?,
            },
            ServiceOfferingRecord::RetailPartnerProduct { partner, category } => {
                Self::RetailPartnerProduct {
                    partner: partner.into(),
                    category: category.into(),
                }
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Storage snapshot of the service-line rules enabled for a location.
pub struct CoreServiceContractsRecord {
    /// Location whose operating day or service rules is described.
    pub location_id: domain::entities::LocationId,
    /// Boarding rules capabilities for the location.
    pub boarding: boarding::ContractRecord,
    /// Daycare rules capabilities for the location.
    pub daycare: daycare::ContractRecord,
    /// Grooming rules capabilities for the location.
    pub grooming: grooming::ContractRecord,
    /// Training rules capabilities for the location.
    pub training: training::ContractRecord,
    /// Retail rules capabilities for the location.
    pub retail: retail::ContractRecord,
}

impl CoreServiceContractsRecord {}

impl From<service_core::ServiceContracts> for CoreServiceContractsRecord {
    fn from(contracts: service_core::ServiceContracts) -> Self {
        Self {
            location_id: contracts.location_id,
            boarding: contracts.boarding.into(),
            daycare: contracts.daycare.into(),
            grooming: contracts.grooming.into(),
            training: contracts.training.into(),
            retail: contracts.retail.into(),
        }
    }
}

impl From<CoreServiceContractsRecord> for service_core::ServiceContracts {
    fn from(record: CoreServiceContractsRecord) -> Self {
        Self::builder()
            .location_id(record.location_id)
            .boarding(record.boarding.into())
            .daycare(record.daycare.into())
            .grooming(record.grooming.into())
            .training(record.training.into())
            .retail(record.retail.into())
            .build()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
/// Stored view of the systems that produce operational data and adjacent labor signals.
pub struct TechnologyEcosystemRecord {
    /// Primary operating portal expected to originate pet-resort facts.
    pub core_portal: CoreOperatingSystemCode,
    /// Access paths available for extracting source evidence.
    pub data_access: Vec<DataAccessPatternCode>,
    /// Nearby systems that may corroborate or enrich operational evidence.
    pub adjacent_systems: Vec<AdjacentSystemCode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Stable codes for operational source systems that may feed NVA workflows.
pub enum CoreOperatingSystemCode {
    /// Stable storage code for a provider-neutral management system.
    ProviderManagementSystem,
    /// Stable storage code for mixed systems.
    MixedSystems,
    /// Provider supplied an unrecognized value; preserve it for audit instead of failing closed.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Stable codes for how operational facts are accessed from source systems.
pub enum DataAccessPatternCode {
    /// Stable storage code for api.
    Api,
    /// Stable storage code for webhook.
    Webhook,
    /// Stable storage code for data export.
    DataExport,
    /// Stable storage code for warehouse.
    Warehouse,
    /// Stable storage code for business intelligence dashboard.
    BusinessIntelligenceDashboard,
    /// Provider supplied an unrecognized value; preserve it for audit instead of failing closed.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Stable codes for adjacent systems that provide labor, recruiting, marketing, or analytics evidence.
pub enum AdjacentSystemCode {
    /// Stable storage code for avature recruiting.
    AvatureRecruiting,
    /// Stable storage code for ga4.
    Ga4,
    /// Stable storage code for amplitude.
    Amplitude,
    /// Stable storage code for google tag manager.
    GoogleTagManager,
    /// Stable storage code for hris.
    Hris,
    /// Stable storage code for labor scheduling.
    LaborScheduling,
    /// Stable storage code for payroll.
    Payroll,
    /// Stable storage code for marketing automation.
    MarketingAutomation,
    /// Stable storage code for ticketing.
    Ticketing,
    /// Stable storage code for call center telephony.
    CallCenterTelephony,
    /// Stable storage code for reviews.
    Reviews,
    /// Stable storage code for email sms marketing.
    EmailSmsMarketing,
    /// Stable storage code for business intelligence.
    BusinessIntelligence,
    /// Stable storage code for data lake.
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

macro_rules! impl_sensitive_storage_debug {
    ($($type:ident),+ $(,)?) => {
        $(
            impl std::fmt::Debug for $type {
                fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    formatter.write_str(concat!(stringify!($type), "([REDACTED])"))
                }
            }
        )+
    };
}

impl_sensitive_storage_debug!(
    SiteFinanceOutcomeRecord,
    ManagerDailyBriefReportingGroup,
    DataQualityIssueRecord,
    DataQualitySourceImportRunRecord,
    DataQualitySyncGapRecord,
    SourceQualityBacklogRow,
    ImportFreshnessRow,
    DataQualityHygieneReportingGroup,
    DataQualityHygieneOutcomeSummary,
    DataQualityHygieneLineageIds,
    ApprovalOutboxLineageIds,
    CurrentApprovalReviewerCapability,
    WorkflowResultRecord,
    ReviewPacketRecord,
    ApprovalRecordRow,
    DataQualityHygieneOutcomeRow,
    SiteFinanceOutcomeRow,
    AuditEventRecord,
    PendingOutboxRecord,
    ApprovedInternalHandoffAuthority,
    ApprovalOutboxBindingRecord,
    DataQualityHygieneLocalPersistenceRecords,
    SiteFinanceLocalPersistenceRecords,
    ApprovalOutboxProjection,
);

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

bidirectional_code_map!(CoreOperatingSystemCode, service_core::OperatingSystem, {
    ProviderManagementSystem => ProviderManagementSystem,
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

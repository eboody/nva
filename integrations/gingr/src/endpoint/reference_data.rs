use super::{AnimalId, Method, Request, SpeciesId};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
/// Typed Gingr request/response value for get locations.
pub struct GetLocations;

impl Request for GetLocations {
    fn method(&self) -> Method {
        Method::Get
    }

    fn path(&self) -> &'static str {
        "/api/v1/get_locations"
    }

    fn parameters(&self) -> Vec<(String, String)> {
        Vec::new()
    }
}

macro_rules! simple_reference_endpoint {
    ($name:ident, $path:literal) => {
        #[derive(Clone, Debug, Default, PartialEq, Eq)]
        /// Zero-parameter Gingr reference-data endpoint generated from a static API path.
        pub struct $name;

        impl Request for $name {
            fn method(&self) -> Method {
                Method::Get
            }

            fn path(&self) -> &'static str {
                $path
            }

            fn parameters(&self) -> Vec<(String, String)> {
                Vec::new()
            }
        }
    };
}

simple_reference_endpoint!(GetSpecies, "/api/v1/get_species");
simple_reference_endpoint!(GetBreeds, "/api/v1/get_breeds");
simple_reference_endpoint!(GetTemperaments, "/api/v1/get_temperaments");

#[derive(Clone, Debug, Default, PartialEq, Eq)]
/// Typed Gingr request/response value for get vets.
pub struct GetVets {
    include_all_information: bool,
}

impl GetVets {
    /// Starts a typed builder for this Gingr endpoint request.
    pub fn builder() -> GetVetsBuilder {
        GetVetsBuilder::default()
    }
}

#[derive(Clone, Debug, Default)]
/// Typed Gingr request/response value for get vets builder.
pub struct GetVetsBuilder {
    include_all_information: bool,
}

impl GetVetsBuilder {
    /// Requests Gingr vet records with extended fields included.
    pub fn include_all_information(mut self, include_all_information: bool) -> Self {
        self.include_all_information = include_all_information;
        self
    }

    /// Builds the typed Gingr request after all parameters have been validated.
    pub fn build(self) -> GetVets {
        GetVets {
            include_all_information: self.include_all_information,
        }
    }
}

impl Request for GetVets {
    fn method(&self) -> Method {
        Method::Get
    }

    fn path(&self) -> &'static str {
        "/api/v1/get_vets"
    }

    fn parameters(&self) -> Vec<(String, String)> {
        if self.include_all_information {
            vec![("vetFlag".to_owned(), "true".to_owned())]
        } else {
            Vec::new()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Typed Gingr request/response value for get immunization types.
pub struct GetImmunizationTypes {
    species: SpeciesId,
}

impl GetImmunizationTypes {
    /// Creates the wrapper from an already validated value.
    pub const fn new(species: SpeciesId) -> Self {
        Self { species }
    }
}

impl Request for GetImmunizationTypes {
    fn method(&self) -> Method {
        Method::Get
    }

    fn path(&self) -> &'static str {
        "/api/v1/get_immunization_types"
    }

    fn parameters(&self) -> Vec<(String, String)> {
        vec![("species_id".to_owned(), self.species.to_string())]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Typed Gingr request/response value for get animal immunizations.
pub struct GetAnimalImmunizations {
    animal: AnimalId,
}

impl GetAnimalImmunizations {
    /// Creates the wrapper from an already validated value.
    pub const fn new(animal: AnimalId) -> Self {
        Self { animal }
    }
}

impl Request for GetAnimalImmunizations {
    fn method(&self) -> Method {
        Method::Get
    }

    fn path(&self) -> &'static str {
        "/api/v1/get_animal_immunizations"
    }

    fn parameters(&self) -> Vec<(String, String)> {
        vec![("animal_id".to_owned(), self.animal.to_string())]
    }
}

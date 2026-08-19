//! Private, opaque diagnostics for sensitive entity aggregates.

macro_rules! opaque_debug {
    ($type:ty, $name:literal) => {
        impl std::fmt::Debug for $type {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(concat!($name, "([REDACTED])"))
            }
        }
    };
}

opaque_debug!(crate::entities::Customer, "Customer");
opaque_debug!(crate::entities::approval::Record, "ApprovalRecord");
opaque_debug!(crate::entities::Document, "Document");
opaque_debug!(crate::entities::CareNote, "CareNote");
opaque_debug!(crate::entities::Incident, "Incident");
opaque_debug!(crate::entities::Message, "Message");

//! App-owned external tool classification used by executable agent policy.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Decision choices for external tool candidate in the agent tool workflow; each value routes reviewed source facts to the right queue, draft, or staff gate.
pub enum ExternalToolCandidate {
    /// Selects the provider portal for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
    ProviderPortal,
    /// Selects payment provider for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
    PaymentProvider,
    /// Selects sms provider for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
    SmsProvider,
    /// Selects email provider for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
    EmailProvider,
    /// Selects file storage for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
    FileStorage,
    /// Selects ocr or document ai for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
    OcrOrDocumentAi,
    /// Selects camera or webcam provider for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
    CameraOrWebcamProvider,
    /// Selects hermes kanban for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
    HermesKanban,
    /// Selects hermes cron or webhook for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
    HermesCronOrWebhook,
    /// Selects postgres for the agent tool decision model so the app can choose a review, evidence, or draft path without taking live action.
    Postgres,
}

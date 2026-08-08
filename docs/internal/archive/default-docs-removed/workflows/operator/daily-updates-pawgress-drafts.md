# Daily Updates / Pawgress Drafts

Daily Updates / Pawgress drafts help care staff and front-desk agents turn routine stay/daycare observations into warm, factual customer-message drafts without starting from a blank screen. The workflow uses source-backed care notes, approved evidence, and a reviewable [workflow packet](../../glossary-workflow-state-terms.md#workflow-packet) to prepare a [draft](../../glossary-workflow-state-terms.md#draft) while staff keep authority over customer delivery, media use, and sensitive care language.

Status: MVP preview/draft contract. Source and test evidence supports local draft/review behavior, not production Pawgress delivery, media publication, live messaging integration, care-task completion, incident/medical decisions, or provider/PMS writes.

Navigation: start with the [operator workflow index](README.md). Entity-first backlinks: [PetSuites core entities](../../design/entity-atlas-petsuites-core-entities.md), [workflow packets](../../design/entity-atlas-workflow-packets-agents.md), [review gates and blocked actions](../../design/entity-atlas-review-safety-boundaries.md), and [source/provenance/data quality](../../design/source-provenance-data-quality-atlas.md).

## Problem solved and time saved

- Problem solved: staff notes, task evidence, and approved media refs must become customer-safe daily updates without copying raw internal notes, inventing care details, or smoothing over concerns.
- First roles whose time is saved: care staff and front-desk agents who write routine pet-parent updates.
- Secondary reviewers/operators: managers, behavior/medical/privacy reviewers, and front-desk leads who should see exceptions instead of re-reading every routine note from scratch.
- Pet-resort example: a customer-visible daycare note such as "Bella played fetch and relaxed in the shade" can become concise draft copy; a behavior, medication, medical, complaint, wrong-media, or internal-only note is flagged or omitted instead of sent.

The labor value is draft-writing and review time avoided. The page must not claim verified production labor savings until a live outcome loop records before/after writing and review minutes.

## Source data and featured entities

Source data must come from provider/read-model/domain facts with a [source ref](../../glossary-architecture-terms.md#source-ref-domainsourcerecordref), [provenance](../../glossary-architecture-terms.md#provenance-domainsourceprovenance), audit ref, or equivalent reviewable evidence. Model memory, prior AI summaries, raw provider payloads, and unreviewed internal notes are not customer-message truth.

| Entity or source fact | Why the workflow needs it | Source of record / authority | Evidence citation |
| --- | --- | --- | --- |
| Care note | Supplies the observed feeding/play/bathroom/rest/behavior/medical/general fact that may be summarized or withheld. | Staff/source-system note evidence normalized as `domain::entities::CareNote`; visibility, kind, and source-quality wording determine review posture. | Source `domain/src/entities.rs` (`domain::entities::{CareNote, care_note::{Kind, Visibility, Subject}}`); source `app/src/daily_update.rs` (`MvpPreviewRequest.notes`, `IncludedFact`, `OmittedFact`, `SuppressionRecord`); tests `app/tests/daily_care_update_mvp.rs`. |
| Pet, owner, reservation/update subject | Tells reviewers whose stay/daycare update is being drafted and prevents wrong-pet or wrong-reservation copy. | Domain pet/customer/reservation identifiers and the triggering `domain::workflow::Event`; humans/system-of-record own corrections. | Source `app/src/daily_update.rs` (`MvpPreviewRequest`, `daily_care_update::Input`); source `domain/src/workflow.rs` (`Event`, `Subject`); tests `app/tests/daily_care_update_mvp.rs`. |
| Customer message draft | Holds the proposed owner-facing copy, channel hint, language, tone, audience, redaction profile, and approved media/document refs. | App draft artifact; customer send and attachment authority remains with staff/review gate and approved delivery system. | Source `app/src/daily_update.rs` (`CustomerMessageDraft`, `MediaDocumentRef`, `daily_care_update::Output`); Rustdoc `target/doc/app/daily_update/struct.CustomerMessageDraft.html`; workflow spec `docs/workflows/daily-care-update-agent.md#5-output-schema`. |
| Included, omitted, and suppressed facts | Shows exactly what source facts were used in customer copy and what was suppressed for visibility, medical/medication, behavior, incident/safety, payment/billing, source-ambiguous, or media-review reasons. | App output contract plus care-note evidence; reviewers decide whether omissions are acceptable or need follow-up. | Source `app/src/daily_update.rs` (`IncludedFact`, `OmittedFact`, `OmissionReason`, `SuppressionRecord`); source `domain/src/message.rs` (`SuppressionReason`, `ReviewState`); tests `app/tests/daily_care_update_mvp.rs`; spec `docs/workflows/daily-care-update-agent.md#included_facts`. |
| Internal flags and review disposition | Routes missing, sensitive, behavior, medical/medication, policy, and raw-internal-note concerns to the right human review. | App review packet and domain `policy::ReviewGate`; human reviewers own final customer wording and delivery approval. | Source `app/src/daily_update.rs` (`InternalFlag`, `InternalFlagCode`, `ReviewDisposition`, `review_gate_for` behavior); source `domain/src/policy.rs` (`ReviewGate`); tests `app/tests/daily_care_update_mvp.rs`. |
| Approval record and send stub | Proves the current MVP creates an approval request and a blocked send stub rather than a live send. | App/domain approval state; final message send is an approved outbox/provider action not implemented by this workflow. | Source `app/src/daily_update.rs` (`MvpPreview.approval`, `SendStub`, `SendMode::ApprovalRequiredStub`); source `domain/src/entities.rs` (`approval::Record`); tests `app/tests/daily_care_update_mvp.rs`. |
| Approved media refs and policy state | Optional evidence for photo/video mentions; prevents claiming a photo exists or publishing media without consent/suitability review. | Media/photo policy and approved media refs; privacy/media reviewer or approved policy owns release. | Workflow spec `docs/workflows/daily-care-update-agent.md#photo-policy`; safety rules `docs/workflows/daily-care-update-agent-parts/safety-rules.md`; current app MVP does not implement media publication. |

Related entities to mention without making them the page center:

- Incident or complaint: may trigger manager/incident review, but the daily update page must not decide incident disposition or owner-notification language.
- Medication, medical, feeding, bathroom, and behavior evidence: may explain why a note is gated or suppressed, but medical/behavior decisions remain human-reviewed.
- Message approval lifecycle: tracks approval requested/approved/rejected state; it does not by itself authorize live delivery.
- Audit event/source evidence: preserves lineage for replay and review; it is not customer copy.
- Data-quality issue: missing, stale, wrong-pet, conflicting, or unverified evidence may become a separate hygiene/review item.

## Featured contracts

| Layer | Contract | What it authorizes | What it does not authorize |
| --- | --- | --- | --- |
| `app` | `app::daily_update::{MvpPreviewRequest, MvpPreview, build_mvp_preview}` | Building a local preview packet from reviewed workflow event evidence and care notes. | Production send, media publication, care-task completion, provider/PMS mutation, or verified labor savings. |
| `app` | `app::daily_update::{CustomerMessageDraft, IncludedFact, OmittedFact, InternalFlag, ReviewDisposition, SendStub}` | Draft copy, included/omitted evidence lists, review flags, review-required disposition, and blocked send stub. | Treating the draft as approved customer communication or hiding unsupported facts. |
| `app` | `app::daily_update::daily_care_update::{Input, Output, Agent}` implementing `app::agents::WorkflowAgent` | Agent prompt-packet semantics for transforming notes into a structured reviewable output. | Free-form model output as source-of-truth or a bypass around deterministic validators. |
| `domain` | `domain::entities::{CareNote, Message, approval::Record}` and `domain::entities::care_note::{Kind, Visibility, Subject}` | Business vocabulary for care evidence, customer-message draft state, and approval records. | Provider-specific payload authority, medical/incident decisions, or final send authority. |
| `domain` | `domain::workflow::{Event, EventType, PolicyContext, AllowedAction}` and `domain::policy::ReviewGate` | Source event and policy/review-gate vocabulary for allowed draft/summarize work. | Expanding allowed actions into live side effects. |
| `domain` | `domain::message::{Direction, Channel, Status, BodyRef}` | Message state vocabulary for outbound drafts and approval-requested status. | Customer delivery, channel consent, provider response handling, or retry semantics. |
| `storage` | No dedicated durable Daily Updates outcome record identified in current evidence. | Planned/future place to persist draft-writing minutes, review disposition, suppression reasons, and send outcomes. | Claiming measured production labor savings today. |
| `integrations/gingr` / provider evidence | Reservation/back-of-house/source records may supply upstream pet/stay/task evidence when normalized. | Source evidence and mapping boundary. | Raw provider payloads as customer copy or domain truth without app/domain review. |

## Authority and source of truth

- Staff notes and task evidence are source evidence for what was observed; the agent may summarize them only when visibility, freshness, and review state allow it.
- `domain::entities::CareNote` and `care_note::Visibility` own whether a note is internal-only, customer-visible, or customer-visible-after-review. `CareNote::is_customer_visible_without_review` still blocks medication, medical, and behavior notes from no-review customer use.
- `domain::workflow::Event` and `PolicyContext` own the triggering event (`DailyNoteCreated` or `DailyUpdateNeeded`), allowed actions (`SummarizeCareNotes`, `DraftCustomerMessage`), automation level, and required review gates.
- `app::daily_update::build_mvp_preview` owns the MVP transformation from source-backed notes into preview packet, draft, included/omitted fact lists, suppression records, held media/document refs, approval request, send stub, and audit log.
- `domain::policy::ReviewGate` and `domain::entities::approval::Record` own the approval vocabulary. Staff, managers, medical/behavior/privacy reviewers, or approved systems of record own final approval decisions.
- Customer message delivery, media publication, outbox handling, provider writes, and channel consent are outside the current MVP evidence and must remain approved-system/human actions.

## Agent work, approvals, and blocked actions

Agent may:

- summarize routine, source-backed care notes into concise customer-safe draft copy;
- list included facts and omitted facts so reviewers can audit every customer-visible claim;
- flag sensitive, missing, stale, conflicting, internal-only, behavior, medical/medication, policy, media/privacy, or customer-message approval gaps;
- draft internal review tasks such as collect missing evidence, review/replace media, clarify ambiguous notes, or route manager review;
- return a structured output that the app validates and holds behind review gates.

Human must approve:

- every current customer send, because the MVP output is review-required and `ReviewDisposition::allows_live_send()` returns false;
- customer wording for behavior, health, medication, incident, complaint, payment/policy, legal/liability, privacy/media, or service-recovery-sensitive facts;
- media/photo use, replacement, or publication;
- any correction when source notes are stale, conflicting, duplicated, wrong-pet, or wrong-reservation;
- any future auto-send policy, deterministic template, channel/consent policy, outbox/effect ledger, and retention/audit policy.

Blocked by default:

- autonomous customer sends or Pawgress delivery;
- photo/video/media publication or attachment without approved media refs and policy;
- marking feeding, medication, play, bathroom, cleaning, incident, handoff, or photo tasks complete;
- medical advice, diagnosis, medication decisions, incident disposition, complaint resolution, or behavior eligibility conclusions;
- provider/PMS writes, schedule changes, booking/status changes, payment/refund/discount movement, or customer-service promises;
- copying raw internal notes, staff debate, provider payloads, source ids, other pet/customer identities, or unreviewed sensitive facts into customer copy;
- inventing meals, play, bathroom events, medication status, photos, staff actions, or reassurance to make a draft feel complete.

## Outcome and labor value

- Estimated labor value: reduced minutes spent writing routine updates and reduced manager/front-desk review time because routine drafts and exceptions are separated.
- Measured outcome record or field: current app output records review disposition, internal flags, included/omitted facts, approval request, send stub, and audit log; a durable Daily Updates labor/outcome storage projection is not identified yet.
- Current evidence status: supported MVP preview/draft contract and tests. The workflow can demonstrate routine draft creation, sensitive/payment/incident/source-ambiguous concern suppression, unapproved media/document ref holding, review-required disposition, approval requested lifecycle, blocked send stub, audit lineage, and flat JSON review-gate output.
- Gap/future source need: add durable outcome capture for draft-writing minutes saved, review minutes saved, approval/rejection/suppression reasons, unsafe-update prevention, exact approved payload, outbox/send result if delivery is later implemented, and correction/void lineage when source evidence changes.

## Contract crosswalk links

Use the [workflow packet row](../../entity-atlas/contract-crosswalk/workflow-packets.md#workflow-by-workflow-entity-map) for the bidirectional path from this workflow page back to the entities it consumes and produces. Use [surface inventory](../../entity-atlas/contract-crosswalk/surface-inventory.md) for source/Rustdoc/test proof, [source/provider flows](../../entity-atlas/contract-crosswalk/source-provider-flows.md) for source-entry and normalization evidence, [storage/persistence](../../entity-atlas/contract-crosswalk/storage-persistence.md) for `message/outcome storage gap`, and [runtime exposure](../../entity-atlas/contract-crosswalk/runtime-exposure.md) for API/worker/CLI/web/script exposure. Rustdoc/module path: `app::daily_update::CustomerMessageDraft`; operator-facing entity family: `Daily update / Pawgress draft packet`.

## Evidence citations

Evidence: Source `app/src/daily_update.rs` (`app::daily_update::{MvpPreviewRequest, MvpPreview, CustomerMessageDraft, ReviewDisposition, InternalFlag, IncludedFact, OmittedFact, SendStub, SendMode, build_mvp_preview}` and `app::daily_update::daily_care_update::{Input, Output, Agent}`); Rustdoc `target/doc/app/daily_update/index.html` and `target/doc/app/daily_update/daily_care_update/index.html` generated by `cargo doc --no-deps --workspace`; tests `app/tests/daily_care_update_mvp.rs`; status: supported local/MVP preview and draft contract, no production Pawgress delivery or media publication evidence.

Evidence: Source `domain/src/entities.rs` (`domain::entities::{CareNote, Message, approval::Record}`, `domain::entities::care_note::{Kind, Visibility, Subject}`, `CareNote::is_customer_visible_without_review`); Rustdoc `target/doc/domain/entities/index.html`; tests `app/tests/daily_care_update_mvp.rs`; status: supported domain vocabulary for care-note evidence, message approval records, and blocked review gates.

Evidence: Source `domain/src/workflow.rs` (`domain::workflow::{Event, EventType, PolicyContext, AllowedAction}`), `domain/src/policy.rs` (`domain::policy::ReviewGate`), and `domain/src/message.rs` (`domain::message::{Direction, Channel, Status, BodyRef}`); Rustdoc `target/doc/domain/workflow/index.html`, `target/doc/domain/policy/index.html`, and `target/doc/domain/message/index.html`; status: supported shared event/review/message vocabulary.

Evidence: Supporting specs `docs/workflows/daily-care-update-agent.md`, `docs/workflows/daily-care-update-agent-parts/output-schema.md`, `docs/workflows/daily-care-update-agent-parts/safety-rules.md`, `docs/workflows/daily-care-update-agent-parts/staff-note-capture.md`, and `docs/workflows/daily-care-update-agent-parts/example-transformations.md`; status: conservative design artifacts that define source fields, safety gates, output shape, and examples without authorizing live sends.

Evidence: Planning/evidence maps `docs/design/entity-driven-workflow-page-template.md`, `docs/design/operator-workflow-page-inventory.md`, and `docs/design/workflow-page-source-rustdoc-map.md`; status: page-shape and citation guidance for operator-facing documentation.

Commands for verification:

- Generate Rustdoc when source links need refreshing: `cargo doc --no-deps --workspace`.
- Validate local Markdown links before handoff: `./scripts/check_markdown_links.py`.

Caveats to preserve:

- Current evidence supports MVP preview/draft contracts and tests only.
- No production Pawgress delivery, live messaging integration, media publication, care-task completion, provider/PMS mutation, or NVA labor-savings measurement is proven by the cited code.
- Durable Daily Updates outcome/labor storage is planned/future unless a later source adds a concrete storage record.

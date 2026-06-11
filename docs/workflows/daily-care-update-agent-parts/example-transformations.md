# Daily care update example transformations

Purpose: define example transformations from terse staff notes and photo/media facts into customer-safe daily-care update draft packets. These are modeling examples for the Daily Care Update Agent; they do not authorize autonomous customer sending, provider/PMS write-back, care-task completion, medication verification, incident disposition, or media publication.

Source basis:

- `docs/workflows/daily-care-update-agent-parts/inputs.md` is the canonical input packet.
- Daily updates are draft/review outputs by default. Customer-message sends remain approval-gated.
- Each customer-facing sentence must trace to source-backed care evidence.
- Sensitive or unresolved facts produce review flags, suppression reasons, or internal tasks instead of cheerful filler.
- Photo/media refs are review-gated until photo/privacy/consent policy is approved.

## Shared draft output shape used in examples

The examples below use this JSON-compatible `structured_output` shape. Field names are provisional and intended to guide later schema/domain work.

```json
{
  "draft_idempotency_key": "daily-update:<reservation_id>:<pet_id>:<service_date>:<window>",
  "workflow_status": "DraftReady | NeedsStaffReview | NeedsManagerReview | NeedsMoreInformation | SuppressedPendingReview",
  "review_policy": "DraftOnly | StaffApprovalRequired | ManagerApprovalRequired",
  "customer_update": {
    "channel": "portal",
    "subject": "Today's update for <Pet>",
    "body": "Customer-safe draft text, or null when suppressed.",
    "photo_media_refs": ["media_ref_123"]
  },
  "included_facts": [
    {
      "fact_id": "fact_001",
      "claim": "Fact stated or implied in customer copy.",
      "source_ref": "care_note:note_123",
      "source_state": "approved_for_customer_summary | recorded_internal | needs_review",
      "used_in": ["body", "photo_media_refs"]
    }
  ],
  "omitted_facts": [
    {
      "source_ref": "care_note:note_124",
      "fact": "Internal/sensitive/unsupported fact not placed in customer copy.",
      "reason": "privacy | sensitivity | unsupported | contradiction | missing_policy | requires_review"
    }
  ],
  "review_flags": [
    {
      "flag": "PhotoConsentReview | MedicationReview | HealthReview | BehaviorReview | MissingRequiredEvidence | ComplaintReview | StaffUncertainty | CustomerMessageApproval",
      "severity": "info | low | medium | high",
      "reason": "Why a human must review or why the draft is safe only as a draft."
    }
  ],
  "recommended_actions": [
    {
      "action": "CreateInternalTask | DraftCustomerMessage | HoldCustomerMessage | RequestStaffClarification",
      "reason": "Operational next step."
    }
  ]
}
```

## General transformation rules

1. Preserve source separation. Raw staff notes are evidence, not customer copy. The output may paraphrase only approved or review-eligible facts.
2. Never infer care completion from vague text. Medication, feeding, bathroom, bath/grooming, play, and photo facts need explicit source evidence.
3. Default to draft/review. Even `DraftReady` means ready for the review UI, not sent.
4. Suppress routine upbeat copy when health, safety, incident, medication exception, behavior/aggression, complaint, or contradictory facts are unresolved.
5. Do not mention other pets, customers, staff names/initials, facility problems, or internal blame unless an approved policy explicitly allows it.
6. Do not diagnose. Use neutral observation language and route health concerns to review.
7. If a required photo/update fact is missing, say so internally and create a task; do not imply a photo exists.

## Example 1: Normal happy day with meal, play, bathroom, and photo facts

### Input notes/photos

```json
{
  "event": "DailyUpdateNeeded",
  "reservation_id": "res_board_1001",
  "pet": {"pet_id": "pet_milo", "name": "Milo", "species": "dog"},
  "service_date": "2026-06-11",
  "window": "afternoon",
  "channel": "portal",
  "care_notes": [
    {
      "note_id": "note_meal_1",
      "classification": "meal/feeding",
      "visibility": "customer-safe after review",
      "review_state": "approved for customer summary",
      "text": "AM breakfast: ate all."
    },
    {
      "note_id": "note_play_1",
      "classification": "play/enrichment",
      "visibility": "customer-safe after review",
      "review_state": "approved for customer summary",
      "text": "Group play: happy, chased ball, took rest break."
    },
    {
      "note_id": "note_bathroom_1",
      "classification": "bathroom/elimination",
      "visibility": "customer-safe after review",
      "review_state": "approved for customer summary",
      "text": "Potty walk complete: pee and normal stool."
    }
  ],
  "media": [
    {
      "media_ref": "media:milo-yard-001",
      "purpose": "PetStatusCheck",
      "review_state": "approved_for_customer_summary",
      "suitability": "pet_only_clear_current_day"
    }
  ]
}
```

### Output schema values

```json
{
  "draft_idempotency_key": "daily-update:res_board_1001:pet_milo:2026-06-11:afternoon",
  "workflow_status": "DraftReady",
  "review_policy": "StaffApprovalRequired",
  "customer_update": {
    "channel": "portal",
    "subject": "Today's update for Milo",
    "body": "Milo had a happy day with us! He enjoyed group play, chased the ball, and took a nice rest break. He ate all of his breakfast and had a normal potty walk. We also have a photo from his play time for you to review.",
    "photo_media_refs": ["media:milo-yard-001"]
  },
  "included_facts": [
    {
      "fact_id": "fact_meal_all",
      "claim": "Milo ate all of his breakfast.",
      "source_ref": "care_note:note_meal_1",
      "source_state": "approved_for_customer_summary",
      "used_in": ["body"]
    },
    {
      "fact_id": "fact_play_ball_rest",
      "claim": "Milo enjoyed group play, chased the ball, and took a rest break.",
      "source_ref": "care_note:note_play_1",
      "source_state": "approved_for_customer_summary",
      "used_in": ["body"]
    },
    {
      "fact_id": "fact_bathroom_normal",
      "claim": "Milo had a potty walk with normal stool.",
      "source_ref": "care_note:note_bathroom_1",
      "source_state": "approved_for_customer_summary",
      "used_in": ["body"]
    },
    {
      "fact_id": "fact_photo_available",
      "claim": "A current-day approved photo is available.",
      "source_ref": "media:media:milo-yard-001",
      "source_state": "approved_for_customer_summary",
      "used_in": ["body", "photo_media_refs"]
    }
  ],
  "omitted_facts": [],
  "review_flags": [
    {
      "flag": "CustomerMessageApproval",
      "severity": "info",
      "reason": "All customer-facing daily updates remain review-gated until an approved auto-send policy exists."
    }
  ],
  "recommended_actions": [
    {
      "action": "DraftCustomerMessage",
      "reason": "Routine facts are source-backed and customer-safe after staff approval."
    }
  ]
}
```

### Why wording is safe

- It uses only positive routine facts with approved customer-summary state.
- It does not invent duration, staff names, other pets, or medical/behavior conclusions.
- It says the photo is available only because an approved media ref exists.
- It remains a draft with staff approval required.

## Example 2: Shy or settling-in pet with honest, reassuring wording

### Input notes/photos

```json
{
  "event": "DailyUpdateNeeded",
  "reservation_id": "res_board_1002",
  "pet": {"pet_id": "pet_luna", "name": "Luna", "species": "cat"},
  "service_date": "2026-06-11",
  "window": "evening",
  "care_notes": [
    {
      "note_id": "note_mood_1",
      "classification": "mood/behavior",
      "visibility": "customer-safe after review",
      "review_state": "approved for customer summary",
      "text": "Luna shy/hiding early, accepted slow approach, came out for treats in PM."
    },
    {
      "note_id": "note_meal_2",
      "classification": "meal/feeding",
      "visibility": "customer-safe after review",
      "review_state": "approved for customer summary",
      "text": "Dinner: ate about half. Water refreshed."
    },
    {
      "note_id": "note_staff_internal_1",
      "classification": "mood/behavior",
      "visibility": "internal-only",
      "review_state": "recorded/internal",
      "text": "Do not rush; may swat if cornered."
    }
  ],
  "media": []
}
```

### Output schema values

```json
{
  "draft_idempotency_key": "daily-update:res_board_1002:pet_luna:2026-06-11:evening",
  "workflow_status": "DraftReady",
  "review_policy": "StaffApprovalRequired",
  "customer_update": {
    "channel": "portal",
    "subject": "Today's update for Luna",
    "body": "Luna is still settling in, and our team is giving her the quiet, patient approach she prefers. She was shy earlier today, then came out for treats this afternoon. She ate about half of her dinner, and her water has been refreshed.",
    "photo_media_refs": []
  },
  "included_facts": [
    {
      "fact_id": "fact_shy_settling",
      "claim": "Luna was shy earlier and is settling in with a patient approach.",
      "source_ref": "care_note:note_mood_1",
      "source_state": "approved_for_customer_summary",
      "used_in": ["body"]
    },
    {
      "fact_id": "fact_treats_pm",
      "claim": "Luna came out for treats in the afternoon.",
      "source_ref": "care_note:note_mood_1",
      "source_state": "approved_for_customer_summary",
      "used_in": ["body"]
    },
    {
      "fact_id": "fact_half_dinner",
      "claim": "Luna ate about half of dinner and water was refreshed.",
      "source_ref": "care_note:note_meal_2",
      "source_state": "approved_for_customer_summary",
      "used_in": ["body"]
    }
  ],
  "omitted_facts": [
    {
      "source_ref": "care_note:note_staff_internal_1",
      "fact": "May swat if cornered.",
      "reason": "internal behavior-handling note; not customer-safe without behavior review wording"
    }
  ],
  "review_flags": [
    {
      "flag": "CustomerMessageApproval",
      "severity": "info",
      "reason": "Routine but still review-gated."
    }
  ],
  "recommended_actions": [
    {
      "action": "DraftCustomerMessage",
      "reason": "The message is honest about shyness without alarming or exposing internal handling language."
    }
  ]
}
```

### Why wording is safe

- It does not pretend Luna was fully relaxed or playful.
- It converts “shy/hiding” into factual, reassuring wording backed by the note.
- It omits the internal “may swat” handling note because that requires behavior-review wording.
- It avoids promises such as “she will be fine tomorrow.”

## Example 3: Missing/partial note data that can still be sent as a limited draft

### Input notes/photos

```json
{
  "event": "DailyUpdateNeeded",
  "reservation_id": "res_daycare_1003",
  "pet": {"pet_id": "pet_bella", "name": "Bella", "species": "dog"},
  "service_date": "2026-06-11",
  "window": "afternoon",
  "care_notes": [
    {
      "note_id": "note_play_2",
      "classification": "play/enrichment",
      "visibility": "customer-safe after review",
      "review_state": "approved for customer summary",
      "text": "Daycare yard: Bella played fetch and relaxed in shade."
    }
  ],
  "missing_expected_evidence": ["meal status", "bathroom status"],
  "media": [
    {
      "media_ref": "media:bella-fetch-001",
      "purpose": "PetStatusCheck",
      "review_state": "approved_for_customer_summary",
      "suitability": "pet_only_clear_current_day"
    }
  ]
}
```

### Output schema values

```json
{
  "draft_idempotency_key": "daily-update:res_daycare_1003:pet_bella:2026-06-11:afternoon",
  "workflow_status": "NeedsStaffReview",
  "review_policy": "StaffApprovalRequired",
  "customer_update": {
    "channel": "portal",
    "subject": "Today's update for Bella",
    "body": "Bella enjoyed time in the daycare yard today. She played fetch and relaxed in the shade. We also have a photo from her play time for you to review.",
    "photo_media_refs": ["media:bella-fetch-001"]
  },
  "included_facts": [
    {
      "fact_id": "fact_fetch_shade",
      "claim": "Bella played fetch and relaxed in the shade.",
      "source_ref": "care_note:note_play_2",
      "source_state": "approved_for_customer_summary",
      "used_in": ["body"]
    },
    {
      "fact_id": "fact_photo_available",
      "claim": "A current-day approved photo is available.",
      "source_ref": "media:media:bella-fetch-001",
      "source_state": "approved_for_customer_summary",
      "used_in": ["body", "photo_media_refs"]
    }
  ],
  "omitted_facts": [
    {
      "source_ref": "expected_evidence:meal_status",
      "fact": "Meal status unknown.",
      "reason": "missing source evidence; do not mention meals"
    },
    {
      "source_ref": "expected_evidence:bathroom_status",
      "fact": "Bathroom status unknown.",
      "reason": "missing source evidence; do not mention bathroom"
    }
  ],
  "review_flags": [
    {
      "flag": "MissingRequiredEvidence",
      "severity": "low",
      "reason": "Meal and bathroom fields are absent. Staff should confirm whether a limited play/photo update is acceptable for this service line."
    },
    {
      "flag": "CustomerMessageApproval",
      "severity": "info",
      "reason": "Customer-facing draft requires approval."
    }
  ],
  "recommended_actions": [
    {
      "action": "DraftCustomerMessage",
      "reason": "A limited draft is possible because it only states supported play/photo facts."
    },
    {
      "action": "RequestStaffClarification",
      "reason": "Ask staff to add meal/bathroom evidence if those facts are required by policy."
    }
  ]
}
```

### Why wording is safe

- It produces a limited draft rather than filling missing meal/bathroom details.
- It includes only supported play/photo facts.
- It clearly routes the missing fields to staff review.

## Example 4: Missing/partial note data that must be held

### Input notes/photos

```json
{
  "event": "DailyUpdateNeeded",
  "reservation_id": "res_board_1004",
  "pet": {"pet_id": "pet_rocky", "name": "Rocky", "species": "dog"},
  "service_date": "2026-06-11",
  "window": "evening",
  "care_notes": [
    {
      "note_id": "note_raw_1",
      "classification": "mixed/unknown",
      "visibility": "internal-only",
      "review_state": "recorded/internal",
      "text": "Rocky okay."
    }
  ],
  "missing_expected_evidence": ["meal status", "bathroom status", "mood/play status", "required Pawgress photo"]
}
```

### Output schema values

```json
{
  "draft_idempotency_key": "daily-update:res_board_1004:pet_rocky:2026-06-11:evening",
  "workflow_status": "NeedsMoreInformation",
  "review_policy": "StaffApprovalRequired",
  "customer_update": {
    "channel": "portal",
    "subject": null,
    "body": null,
    "photo_media_refs": []
  },
  "included_facts": [],
  "omitted_facts": [
    {
      "source_ref": "care_note:note_raw_1",
      "fact": "Rocky okay.",
      "reason": "too vague and internal-only; cannot support customer-facing claims"
    },
    {
      "source_ref": "expected_evidence:required_pawgress_photo",
      "fact": "Required photo missing.",
      "reason": "missing media evidence; do not imply a photo exists"
    }
  ],
  "review_flags": [
    {
      "flag": "MissingRequiredEvidence",
      "severity": "high",
      "reason": "No customer-safe care facts are available and a required photo is missing."
    }
  ],
  "recommended_actions": [
    {
      "action": "HoldCustomerMessage",
      "reason": "No source-backed customer-safe draft can be generated."
    },
    {
      "action": "RequestStaffClarification",
      "reason": "Ask staff to add meal, bathroom, mood/play, and required photo evidence."
    }
  ]
}
```

### Why wording is safe

- No customer copy is generated because the only note is vague and internal-only.
- The agent does not convert “okay” into invented meal, play, or bathroom facts.
- Missing required media becomes an internal task/review reason.

## Example 5: Medication administered normally

### Input notes/photos

```json
{
  "event": "DailyNoteCreated",
  "reservation_id": "res_board_1005",
  "pet": {"pet_id": "pet_daisy", "name": "Daisy", "species": "dog"},
  "service_date": "2026-06-11",
  "window": "evening",
  "care_profile_snapshot": {
    "medications": [
      {
        "medication_ref": "med:daisy:thyro-tabs",
        "name_display": "Thyro-Tabs",
        "schedule": "evening",
        "review_requirement": "reviewed_instruction_on_file"
      }
    ]
  },
  "care_notes": [
    {
      "note_id": "note_med_1",
      "classification": "medication",
      "visibility": "customer-safe after review",
      "review_state": "approved for customer summary",
      "authorized_staff_evidence": true,
      "text": "PM Thyro-Tabs given per instructions. No issue noted."
    },
    {
      "note_id": "note_mood_2",
      "classification": "mood/behavior",
      "visibility": "customer-safe after review",
      "review_state": "approved for customer summary",
      "text": "Daisy enjoyed individual play and cuddles."
    }
  ]
}
```

### Output schema values

```json
{
  "draft_idempotency_key": "daily-update:res_board_1005:pet_daisy:2026-06-11:evening",
  "workflow_status": "NeedsStaffReview",
  "review_policy": "StaffApprovalRequired",
  "customer_update": {
    "channel": "portal",
    "subject": "Today's update for Daisy",
    "body": "Daisy enjoyed individual play and cuddles today. Her evening medication was given per the instructions on file, with no issue noted.",
    "photo_media_refs": []
  },
  "included_facts": [
    {
      "fact_id": "fact_individual_play",
      "claim": "Daisy enjoyed individual play and cuddles.",
      "source_ref": "care_note:note_mood_2",
      "source_state": "approved_for_customer_summary",
      "used_in": ["body"]
    },
    {
      "fact_id": "fact_med_given",
      "claim": "Evening medication was given per reviewed instructions with no issue noted.",
      "source_ref": "care_note:note_med_1 + care_profile:med:daisy:thyro-tabs",
      "source_state": "approved_for_customer_summary",
      "used_in": ["body"]
    }
  ],
  "omitted_facts": [
    {
      "source_ref": "care_profile:med:daisy:thyro-tabs.name_display",
      "fact": "Specific medication name Thyro-Tabs.",
      "reason": "minimize medical detail in customer copy unless template/policy requires naming it"
    }
  ],
  "review_flags": [
    {
      "flag": "MedicationReview",
      "severity": "low",
      "reason": "Medication facts are sensitive and should be staff-reviewed even when normal."
    },
    {
      "flag": "CustomerMessageApproval",
      "severity": "info",
      "reason": "Customer-facing draft requires approval."
    }
  ],
  "recommended_actions": [
    {
      "action": "DraftCustomerMessage",
      "reason": "Medication administration is explicitly evidenced and normal, but still review-gated."
    }
  ]
}
```

### Why wording is safe

- It requires both reviewed medication instructions and authorized staff evidence.
- It does not infer dose accuracy beyond “per instructions on file.”
- It minimizes medication name/detail in the customer copy while preserving source refs for review.

## Example 6: Medication issue requiring review

### Input notes/photos

```json
{
  "event": "DailyNoteCreated",
  "reservation_id": "res_board_1006",
  "pet": {"pet_id": "pet_charlie", "name": "Charlie", "species": "dog"},
  "service_date": "2026-06-11",
  "window": "morning",
  "care_profile_snapshot": {
    "medications": [
      {
        "medication_ref": "med:charlie:am-capsule",
        "name_display": "AM capsule",
        "schedule": "morning",
        "review_requirement": "reviewed_instruction_on_file"
      }
    ]
  },
  "care_notes": [
    {
      "note_id": "note_med_issue_1",
      "classification": "medication",
      "visibility": "internal-only",
      "review_state": "needs manager review",
      "authorized_staff_evidence": true,
      "text": "AM med refused/spit out. Not sure full dose swallowed. Left msg for lead."
    },
    {
      "note_id": "note_play_3",
      "classification": "play/enrichment",
      "visibility": "customer-safe after review",
      "review_state": "approved for customer summary",
      "text": "Charlie walked calmly outside."
    }
  ]
}
```

### Output schema values

```json
{
  "draft_idempotency_key": "daily-update:res_board_1006:pet_charlie:2026-06-11:morning",
  "workflow_status": "SuppressedPendingReview",
  "review_policy": "ManagerApprovalRequired",
  "customer_update": {
    "channel": "portal",
    "subject": null,
    "body": null,
    "photo_media_refs": []
  },
  "included_facts": [],
  "omitted_facts": [
    {
      "source_ref": "care_note:note_med_issue_1",
      "fact": "AM medication may not have been fully swallowed.",
      "reason": "medication exception requiring manager/medical-review wording before customer communication"
    },
    {
      "source_ref": "care_note:note_play_3",
      "fact": "Charlie walked calmly outside.",
      "reason": "routine positive update suppressed until medication issue review decides communication path"
    }
  ],
  "review_flags": [
    {
      "flag": "MedicationReview",
      "severity": "high",
      "reason": "Medication refusal/uncertain dose is a sensitive care exception."
    }
  ],
  "recommended_actions": [
    {
      "action": "HoldCustomerMessage",
      "reason": "Do not send routine daily update while medication issue is unresolved."
    },
    {
      "action": "CreateInternalTask",
      "reason": "Route to manager/authorized medical-care reviewer for next steps and customer communication."
    }
  ]
}
```

### Why wording is safe

- It does not bury a medication exception inside a cheerful update.
- It does not tell the customer “medication given” when full administration is uncertain.
- It routes the issue to manager review before any customer copy is drafted.

## Example 7: Concerning stool/vomiting requiring review

### Input notes/photos

```json
{
  "event": "DailyNoteCreated",
  "reservation_id": "res_board_1007",
  "pet": {"pet_id": "pet_ollie", "name": "Ollie", "species": "dog"},
  "service_date": "2026-06-11",
  "window": "afternoon",
  "care_notes": [
    {
      "note_id": "note_health_1",
      "classification": "bathroom/elimination",
      "visibility": "internal-only",
      "review_state": "needs manager review",
      "text": "Loose stool x2 after lunch; vomited small amount of kibble at 2:10. Monitoring."
    },
    {
      "note_id": "note_play_4",
      "classification": "play/enrichment",
      "visibility": "customer-safe after review",
      "review_state": "approved for customer summary",
      "text": "Ollie rested quietly after lunch."
    }
  ]
}
```

### Output schema values

```json
{
  "draft_idempotency_key": "daily-update:res_board_1007:pet_ollie:2026-06-11:afternoon",
  "workflow_status": "SuppressedPendingReview",
  "review_policy": "ManagerApprovalRequired",
  "customer_update": {
    "channel": "portal",
    "subject": null,
    "body": null,
    "photo_media_refs": []
  },
  "included_facts": [],
  "omitted_facts": [
    {
      "source_ref": "care_note:note_health_1",
      "fact": "Loose stool and vomiting observed.",
      "reason": "health concern; requires manager/health review and approved customer wording"
    },
    {
      "source_ref": "care_note:note_play_4",
      "fact": "Ollie rested quietly after lunch.",
      "reason": "do not send routine copy while health concern is unresolved"
    }
  ],
  "review_flags": [
    {
      "flag": "HealthReview",
      "severity": "high",
      "reason": "Vomiting and repeated loose stool are concerning health observations."
    }
  ],
  "recommended_actions": [
    {
      "action": "HoldCustomerMessage",
      "reason": "Routine daily update is suppressed pending health/manager review."
    },
    {
      "action": "CreateInternalTask",
      "reason": "Ask manager/authorized staff to determine monitoring, parent communication, and any vet/escalation policy."
    }
  ]
}
```

### Why wording is safe

- It avoids diagnosis and avoids reassuring beyond the evidence.
- It prevents a routine “rested quietly” update from masking a health concern.
- It requires approved health/manager wording before customer communication.

## Example 8: Mild behavior concern or aggression requiring review

### Input notes/photos

```json
{
  "event": "DailyNoteCreated",
  "reservation_id": "res_daycare_1008",
  "pet": {"pet_id": "pet_max", "name": "Max", "species": "dog"},
  "service_date": "2026-06-11",
  "window": "midday",
  "care_notes": [
    {
      "note_id": "note_behavior_1",
      "classification": "mood/behavior",
      "visibility": "internal-only",
      "review_state": "needs manager review",
      "text": "Max barked/growled at gate, snapped toward leash when moved to side yard. No contact/injury. Removed from group."
    },
    {
      "note_id": "note_meal_3",
      "classification": "meal/feeding",
      "visibility": "customer-safe after review",
      "review_state": "approved for customer summary",
      "text": "Ate lunch normally."
    }
  ]
}
```

### Output schema values

```json
{
  "draft_idempotency_key": "daily-update:res_daycare_1008:pet_max:2026-06-11:midday",
  "workflow_status": "SuppressedPendingReview",
  "review_policy": "ManagerApprovalRequired",
  "customer_update": {
    "channel": "portal",
    "subject": null,
    "body": null,
    "photo_media_refs": []
  },
  "included_facts": [],
  "omitted_facts": [
    {
      "source_ref": "care_note:note_behavior_1",
      "fact": "Barking/growling/snapping behavior and group removal.",
      "reason": "behavior/aggression concern requires behavior or manager review before customer wording"
    },
    {
      "source_ref": "care_note:note_meal_3",
      "fact": "Ate lunch normally.",
      "reason": "routine meal fact suppressed until behavior review decides update path"
    }
  ],
  "review_flags": [
    {
      "flag": "BehaviorReview",
      "severity": "high",
      "reason": "Snapping toward leash and removal from group are behavior/safety concerns."
    }
  ],
  "recommended_actions": [
    {
      "action": "HoldCustomerMessage",
      "reason": "Do not send routine customer update before behavior review."
    },
    {
      "action": "CreateInternalTask",
      "reason": "Route to manager/behavior lead for care plan, group-play eligibility, and customer communication."
    }
  ]
}
```

### Why wording is safe

- It does not soften or hide a safety-relevant behavior issue in customer copy.
- It avoids blame, labels, or conclusions such as “aggressive dog.”
- It preserves the precise internal facts for review while holding external messaging.

## Example 9: Missed feeding or refusal requiring review

### Input notes/photos

```json
{
  "event": "DailyNoteCreated",
  "reservation_id": "res_board_1009",
  "pet": {"pet_id": "pet_sadie", "name": "Sadie", "species": "dog"},
  "service_date": "2026-06-11",
  "window": "morning",
  "care_notes": [
    {
      "note_id": "note_feed_issue_1",
      "classification": "meal/feeding",
      "visibility": "internal-only",
      "review_state": "needs staff review",
      "text": "Breakfast not given during 7am round; found in bin at 9:30. Offered late, Sadie sniffed/refused. Lead notified."
    },
    {
      "note_id": "note_mood_3",
      "classification": "mood/behavior",
      "visibility": "customer-safe after review",
      "review_state": "approved for customer summary",
      "text": "Sadie was calm on morning walk."
    }
  ]
}
```

### Output schema values

```json
{
  "draft_idempotency_key": "daily-update:res_board_1009:pet_sadie:2026-06-11:morning",
  "workflow_status": "SuppressedPendingReview",
  "review_policy": "ManagerApprovalRequired",
  "customer_update": {
    "channel": "portal",
    "subject": null,
    "body": null,
    "photo_media_refs": []
  },
  "included_facts": [],
  "omitted_facts": [
    {
      "source_ref": "care_note:note_feed_issue_1",
      "fact": "Breakfast was missed/delayed and later refused.",
      "reason": "feeding exception and potential service issue require staff/manager review"
    },
    {
      "source_ref": "care_note:note_mood_3",
      "fact": "Sadie was calm on morning walk.",
      "reason": "routine positive fact suppressed pending feeding-exception review"
    }
  ],
  "review_flags": [
    {
      "flag": "FeedingExceptionReview",
      "severity": "high",
      "reason": "Missed/delayed feeding and refusal require review and may require customer communication."
    }
  ],
  "recommended_actions": [
    {
      "action": "HoldCustomerMessage",
      "reason": "Do not send a routine daily update while feeding exception is unresolved."
    },
    {
      "action": "CreateInternalTask",
      "reason": "Route to manager/lead to determine correction, monitoring, and approved customer wording."
    }
  ]
}
```

### Why wording is safe

- It does not say “ate breakfast” or omit the missed feeding issue.
- It avoids assigning blame in customer copy before review.
- It treats feeding exceptions as sensitive operational facts requiring review.

## Example 10: Staff uncertainty requiring review

### Input notes/photos

```json
{
  "event": "DailyNoteCreated",
  "reservation_id": "res_board_1010",
  "pet": {"pet_id": "pet_nala", "name": "Nala", "species": "dog"},
  "service_date": "2026-06-11",
  "window": "afternoon",
  "care_notes": [
    {
      "note_id": "note_uncertain_1",
      "classification": "mixed/unknown",
      "visibility": "internal-only",
      "review_state": "needs staff review",
      "text": "I think Nala ate? bowl empty when I checked but not sure if kennel mate got it. Potty maybe normal?"
    },
    {
      "note_id": "note_photo_uncertain_1",
      "classification": "photo/media",
      "visibility": "internal-only",
      "review_state": "needs staff review",
      "text": "Photo might be Nala, check markings."
    }
  ],
  "media": [
    {
      "media_ref": "media:nala-uncertain-001",
      "purpose": "PetStatusCheck",
      "review_state": "needs_staff_review",
      "suitability": "possible_wrong_pet"
    }
  ]
}
```

### Output schema values

```json
{
  "draft_idempotency_key": "daily-update:res_board_1010:pet_nala:2026-06-11:afternoon",
  "workflow_status": "NeedsMoreInformation",
  "review_policy": "StaffApprovalRequired",
  "customer_update": {
    "channel": "portal",
    "subject": null,
    "body": null,
    "photo_media_refs": []
  },
  "included_facts": [],
  "omitted_facts": [
    {
      "source_ref": "care_note:note_uncertain_1",
      "fact": "Meal and bathroom status uncertain.",
      "reason": "staff uncertainty; cannot support customer-facing claims"
    },
    {
      "source_ref": "media:media:nala-uncertain-001",
      "fact": "Photo may be wrong pet.",
      "reason": "privacy/identity risk; must not attach or mention photo"
    }
  ],
  "review_flags": [
    {
      "flag": "StaffUncertainty",
      "severity": "high",
      "reason": "Meal, bathroom, and photo identity are uncertain."
    },
    {
      "flag": "PhotoConsentReview",
      "severity": "high",
      "reason": "Possible wrong-pet media creates privacy and trust risk."
    }
  ],
  "recommended_actions": [
    {
      "action": "HoldCustomerMessage",
      "reason": "No reliable customer-safe facts are available."
    },
    {
      "action": "RequestStaffClarification",
      "reason": "Ask staff to verify meal/bathroom evidence and replace or approve media."
    }
  ]
}
```

### Why wording is safe

- It refuses to convert uncertain notes into definitive facts.
- It blocks possible wrong-pet media from customer use.
- It creates clarification work rather than hiding the uncertainty.

## Example 11: Complaint/customer-sensitive note requiring review

### Input notes/photos

```json
{
  "event": "DailyNoteCreated",
  "reservation_id": "res_board_1011",
  "pet": {"pet_id": "pet_coco", "name": "Coco", "species": "dog"},
  "customer": {"customer_id": "cust_1011", "complaint_suppression_state": "active_review"},
  "service_date": "2026-06-11",
  "window": "evening",
  "care_notes": [
    {
      "note_id": "note_complaint_1",
      "classification": "customer preference",
      "visibility": "internal-only",
      "review_state": "needs manager review",
      "text": "Owner complained yesterday that updates sound generic and photo was blurry. Wants manager to review all Coco messages."
    },
    {
      "note_id": "note_play_5",
      "classification": "play/enrichment",
      "visibility": "customer-safe after review",
      "review_state": "approved for customer summary",
      "text": "Coco enjoyed one-on-one play and belly rubs."
    },
    {
      "note_id": "note_photo_2",
      "classification": "photo/media",
      "visibility": "customer-safe after review",
      "review_state": "needs manager review",
      "text": "Photo is cute but slightly blurry; use if manager okay."
    }
  ],
  "media": [
    {
      "media_ref": "media:coco-blurry-001",
      "purpose": "PetStatusCheck",
      "review_state": "needs_manager_review",
      "suitability": "blurry"
    }
  ]
}
```

### Output schema values

```json
{
  "draft_idempotency_key": "daily-update:res_board_1011:pet_coco:2026-06-11:evening",
  "workflow_status": "NeedsManagerReview",
  "review_policy": "ManagerApprovalRequired",
  "customer_update": {
    "channel": "portal",
    "subject": "Today's update for Coco",
    "body": "Coco enjoyed one-on-one play today and soaked up plenty of belly rubs.",
    "photo_media_refs": []
  },
  "included_facts": [
    {
      "fact_id": "fact_one_on_one_play",
      "claim": "Coco enjoyed one-on-one play and belly rubs.",
      "source_ref": "care_note:note_play_5",
      "source_state": "approved_for_customer_summary",
      "used_in": ["body"]
    }
  ],
  "omitted_facts": [
    {
      "source_ref": "care_note:note_complaint_1",
      "fact": "Customer complained about generic updates and blurry photo; manager review requested.",
      "reason": "complaint/customer-sensitive internal note; do not expose in daily update copy"
    },
    {
      "source_ref": "media:media:coco-blurry-001",
      "fact": "Blurry photo exists.",
      "reason": "poor-photo/customer-sensitive context; hold unless manager approves or replace"
    }
  ],
  "review_flags": [
    {
      "flag": "ComplaintReview",
      "severity": "high",
      "reason": "Customer has active complaint/sensitivity around update quality."
    },
    {
      "flag": "PhotoConsentReview",
      "severity": "medium",
      "reason": "Blurry photo should not be attached automatically."
    }
  ],
  "recommended_actions": [
    {
      "action": "DraftCustomerMessage",
      "reason": "A concise non-generic text draft is possible from supported play facts."
    },
    {
      "action": "CreateInternalTask",
      "reason": "Manager should review wording and either approve text-only update or request replacement photo."
    }
  ]
}
```

### Why wording is safe

- It does not mention the complaint or blame prior staff/system behavior.
- It avoids attaching a known blurry photo automatically.
- It escalates to manager review because the customer has an active sensitivity around update quality.

## Review flag summary

| Situation | Customer copy? | Minimum review flag | Safe behavior |
|---|---:|---|---|
| Routine meal/play/bathroom/photo facts approved | Yes, draft only | `CustomerMessageApproval` | Draft concise, source-backed copy. |
| Shy/settling-in but non-sensitive and approved | Yes, draft only | `CustomerMessageApproval` | Be honest and reassuring; omit internal handling notes. |
| Some routine fields missing but enough approved facts remain | Maybe | `MissingRequiredEvidence` | Draft only supported facts; ask staff to fill missing fields if required. |
| No reliable customer-safe facts or required photo missing | No | `MissingRequiredEvidence` | Hold and request staff clarification/replacement media. |
| Medication administered normally | Maybe | `MedicationReview` | Include only if reviewed instruction + authorized evidence exist; keep staff-reviewed. |
| Medication exception/refusal/uncertain dose | No | `MedicationReview` | Suppress routine update; route to manager/authorized reviewer. |
| Vomiting, repeated loose stool, health concern | No | `HealthReview` | Suppress routine update; route to health/manager review. |
| Aggression, snapping, removal from group | No | `BehaviorReview` | Suppress routine update; route to behavior/manager review. |
| Missed/delayed feeding or refusal | No | `FeedingExceptionReview` | Suppress routine update; route to manager/lead review. |
| Staff uncertainty or conflicting evidence | No | `StaffUncertainty` | Hold, request clarification, do not infer. |
| Complaint/customer-sensitive note | Maybe, manager-only | `ComplaintReview` | Manager review; avoid complaint details in customer copy; replace poor media. |

## Implementation notes for downstream schema work

- `included_facts` should become the audit/citation surface used by validation: every sentence in `customer_update.body` must be traceable.
- `omitted_facts` is not customer-visible; it explains why source evidence was excluded, suppressed, or routed to review.
- `workflow_status` should be validated independently from the natural-language copy. A body may exist while status is still `NeedsStaffReview` or `NeedsManagerReview`.
- `review_policy` should map to current `DraftOnly` / `ManagerApprovalRequired` primitives until richer staff-vs-manager approval states are implemented.
- Media should be represented by `MediaRef` and suitability/review metadata, not raw images, unless a later approved image-analysis workflow permits pixel processing.
- These examples intentionally avoid staff initials, exact medication doses, diagnoses, complaint details, and operational blame because current policy does not approve those as routine customer copy.

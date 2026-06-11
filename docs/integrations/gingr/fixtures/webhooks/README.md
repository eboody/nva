# Gingr webhook fixture candidates

These fixtures are sanitized from Gingr's public Developer Guide examples and are intended for SDK parser/verifier tests only.

Source articles:
- https://support.gingrapp.com/hc/en-us/articles/25659829911565-Event-Data-Structure-Examples-Reference
- https://support.gingrapp.com/hc/en-us/articles/25660010986125-Event-Types-and-Response-Codes-Reference

Sanitization rules:
- No real customer data, real webhook URLs, or real signature keys.
- Names, addresses, emails, phones, operational notes, and exact business details are fictionalized.
- Signatures are recomputed with fake key `test-webhook-signature-key` using Gingr's documented message rule: `webhook_type` + `entity_id` + `entity_type`, no separators.
- These are shape candidates, not complete schemas. The `check_out` docs state invoice data may be included but is not present in the example.

Expected signatures with fake key `test-webhook-signature-key`:
- `check_out` + `76390` + `reservation` -> `e6d62e27528513a9c4aa399e3a79192aacc490cfeae202fa753f319967ab30eb`
- `email_sent` + `5917` + `owner` -> `2e1a768a873926e959d89b86d7a7f6c8b62dcf4426e2af6839f6dff306b9a1ef`

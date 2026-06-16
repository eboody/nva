# BI Data-Shape Discovery Questions

Date: 2026-06-16

## Purpose

These questions drive the current NVA setup work.

NVA's repository-wide goal is labor-cost reduction through creative automation, process redesign, operational tooling, employee-facing helpers such as SOP/chat assistants, reports, scheduling/staffing optimization, and workflow controls.

BI is not being asked to define that strategy. BI is relevant because it currently owns a database, and Gingr is one source feeding or informing it. These questions are therefore about factual data shape: what exists, where it comes from, how it joins, how reliable it is, and what assumptions are embedded in existing transformations.

No credentials or formal access are required for this stage. Rough answers, schema screenshots, table names, dashboard names, redacted examples, transformation snippets, or "not sure yet" answers are all useful.

## Short version to send

```text
Totally makes sense. Since BI owns the current database and Gingr is one of the sources, I mostly want to understand the factual data shape before we model anything incorrectly.

We’re not asking BI to define the product strategy or decide what labor-cost tools to build. We just want to understand what data exists, where it comes from, how it joins together, and what’s reliable vs messy.

No credentials/access needed yet. Rough answers, schema screenshots, table names, dashboard names, redacted examples, or “not sure yet” answers are all useful.

Key questions:

1. What database/BI system currently stores the Gingr-derived data?

2. What sources feed it today — Gingr API, Gingr reports/exports, webhooks, scheduled jobs, manual uploads, other systems?

3. Besides Gingr, what other systems or datasets are available/relevant — scheduling, timeclock, payroll/wage cost, POS/payments, capacity/rooms, communications, task/work-order systems, SOP/process docs?

4. What are the main table names, and what is each table’s grain — reservation, pet stay, customer, pet, invoice/payment, service/day, staff shift, etc.?

5. Which Gingr objects are represented — reservations, pets, owners, services, locations, rooms, invoices/payments, check-in/out, notes/incidents/medications?

6. What IDs are stable enough to join on — customer, pet, reservation/stay, invoice/payment, location, service, staff/user?

7. How does the data refresh — full refresh, incremental sync, daily/hourly/manual, backfills?

8. How are edits/deletes/merges/duplicates handled?

9. Which fields are reliable, and which are known to be messy or misleading?

10. Are there overloaded statuses/columns whose names don’t quite mean what they seem to mean?

11. Are raw source payloads or import batches retained anywhere, or only cleaned/transformed rows?

12. Is there provenance metadata — source endpoint/report, import timestamp, provider record ID, batch ID, transformation version?

13. Is there existing code that transforms Gingr data into BI tables/types? Even messy code, table names, or type names would help.

14. Are there reports/dashboards that are trusted as correct? Are there reports people distrust? Why?

15. What would be easiest to share before formal access — table list, schema screenshot, redacted sample rows, transformation code, dashboard screenshots, or notes on known data issues?
```

## Full working version

Use this version internally when shaping source contracts, provenance, identity resolution, read models, and future automation opportunities.

1. What database or BI system currently stores the Gingr-derived data?
   - Postgres, BigQuery, Snowflake, SQL Server, Sheets, Power BI, Tableau extracts, etc.?
   - Is it BI-only, production-facing, experimental, or a mix?

2. What sources feed that database today?
   - Gingr API?
   - Gingr reports/CSV exports?
   - Webhooks?
   - Scheduled jobs?
   - Manual uploads?
   - Other non-Gingr systems?

3. Besides Gingr, what other systems or datasets are currently available or relevant?
   - Staff scheduling?
   - Timeclock?
   - Payroll/wage cost?
   - POS/payments?
   - Capacity/rooms/locations?
   - Customer communications?
   - Task/work-order systems?
   - SOP/process documentation?
   - Spreadsheets/manual reports?

4. What is the grain of the main tables?
   - One row per reservation?
   - One row per pet stay?
   - One row per pet?
   - One row per customer/owner?
   - One row per invoice/payment?
   - One row per service/day?
   - One row per staff shift?
   - Something else?

5. What are the most important table names or type names?
   - Even just a list of table names would help.
   - If there are existing BI model/type names, those are useful too.

6. What Gingr objects are currently represented?
   - Reservations?
   - Pets/animals?
   - Owners/customers?
   - Services/add-ons?
   - Locations?
   - Rooms/runs/kennels?
   - Invoices/payments/deposits?
   - Check-in/check-out events?
   - Notes/incidents/medications?

7. What IDs are stable enough to join on?
   - Customer/owner ID?
   - Pet/animal ID?
   - Reservation/stay ID?
   - Invoice/payment ID?
   - Location ID?
   - Service ID?
   - Staff/user ID?
   - Are any IDs known to be unstable, duplicated, or reused?

8. How are Gingr records joined together today?
   - Reservation → pet?
   - Pet → owner?
   - Reservation → invoice/payment?
   - Reservation → service/add-ons?
   - Reservation → location/room?
   - Are these joins straightforward or messy?

9. How does the data refresh?
   - Full refresh?
   - Incremental sync?
   - Daily/hourly/manual?
   - Backfills?
   - Does historical data change after it is first pulled?

10. How are edits, deletes, merges, and duplicates handled?
    - Customer merges?
    - Pet merges?
    - Cancelled reservations?
    - Changed checkout dates?
    - Corrected payments?
    - Duplicate records?

11. Which fields are considered reliable?
    - Reservation dates?
    - Actual check-in/check-out times?
    - Service type/category?
    - Pet/customer IDs?
    - Location/room assignment?
    - Invoice/payment status?
    - Deposits?
    - Staff/user fields, if present?

12. Which fields are known to be messy or misleading?
    - Statuses?
    - Timestamps?
    - Service labels?
    - Add-ons?
    - Owner/pet relationships?
    - Checkout/completion fields?
    - Payment/deposit fields?
    - Notes/free-text fields?

13. Are there any overloaded statuses or columns?
    - Fields that mean different things in different contexts?
    - Report columns that don’t quite mean what their names imply?
    - Values that staff use inconsistently?

14. Are raw source payloads retained anywhere?
    - Raw API responses?
    - Raw CSV/report exports?
    - Import batches?
    - Source file names?
    - Pull timestamps?
    - Or does the DB only store cleaned/transformed rows?

15. Is there provenance metadata?
    - Source system?
    - Source endpoint/report name?
    - Import batch ID?
    - Pulled/imported timestamp?
    - Original provider record ID?
    - Transformation version?

16. Is there existing code that transforms Gingr data into BI tables/types?
    - If yes, can we inspect it eventually?
    - If not, can we at least see the table/type names or rough transformation steps?

17. Are there existing reports/dashboards that are trusted as “correct”?
    - Which ones?
    - What tables do they rely on?

18. Are there existing reports/dashboards that people distrust?
    - Which ones?
    - Why: bad joins, stale data, duplicate records, missing fields, manual cleanup, inconsistent definitions?

19. Are there any documented metric definitions?
    - Occupancy?
    - Utilization?
    - Revenue?
    - Average stay length?
    - Cancellation/no-show?
    - Service counts?
    - Customer retention?
    - Anything else?

20. If only one helpful artifact can be shared before formal access, what would be easiest?
    - Table list?
    - Schema screenshot?
    - Redacted sample rows?
    - Existing transformation code?
    - Dashboard screenshots?
    - Notes on known data issues?

## Decision rubric

Use `docs/discovery/bi-question-decision-rubric.md` to convert answers to these questions into concrete next-work decisions. The questions are the input; the rubric decides whether the next safest move is source inventory, contract refinement, projection, workflow validation, or labor-cost modeling.

## Modeling implications

These questions should guide current setup and architecture decisions:

- Model Gingr as one source adapter, not as the universal domain model.
- Keep source provenance first-class: source system, endpoint/report, batch, pulled-at timestamp, provider record ID, and transformation version.
- Preserve table grain and join assumptions explicitly.
- Treat reliability/messiness as data-quality facts, not incidental logs.
- Design source contracts so non-Gingr sources can be added later without rewriting the domain core.
- Avoid asking BI to define NVA's cost-reduction strategy; use BI answers to understand the existing factual data surface.
- Let NVA independently identify cost-reduction opportunities from the modeled operational facts.

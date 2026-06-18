# NVA Pet Resorts AI Program Context Pack

Context for evaluating / preparing for an engineering role helping NVA Pet Resorts implement AI agents and operational automation.

## Source messages

> Yeah that sounds great. The gist is, I run IT/Tech for a portfolio 170 Pet Resorts in the US. I’m setting up an “ai program” to build agents to automate/optimize operations. We’re finalizing an enterprise Claude contract this month, and then I’m looking to bring on an engineer to build agents for us. If interested we can work out the term + rate. Timing is likely early July. We can chat more this week, but that’s the job in a nutshell

> Hey dude my brother is hiring a dev to help his company NVA streamline processes with ai. He’s newly leading their AI adoption efforts. I mentioned you may be interested and he said to make an intro if you are. Want me to T up that convo?

---

## 1. What NVA / NVA Pet Resorts is

**NVA = National Veterinary Associates.** It is a major pet-care operator with multiple business lines:

- General Practice veterinary hospitals
- Pet Resorts
- Equine
- Historically/sibling structure: Ethos Veterinary Health for specialty/emergency hospitals

Public NVA materials describe the company as a large pet-health community focused on innovation, medical excellence, and support for pets and their families.

The relevant division here is **NVA Pet Resorts**, which offers:

- Boarding
- Daycare
- Grooming
- Training

Their pet-resort positioning is: **convenience for pet owners, fun/trust/safety for pets, and strong local guest experience.**

The message says the contact runs IT/Tech for a portfolio of **170 Pet Resorts in the US**. Publicly, PetSuites itself says it has **80+ locations**, but NVA Pet Resorts appears to include more than PetSuites alone — other brands seen in job listings include **Pooch Hotel**, **Elite Suites**, **The Bark Side**, **Woofdorf Astoria**, **Doggie District**, etc.

So the internal operating portfolio likely looks like:

> A federated / multi-brand pet-resort network, with centralized technology/IT trying to impose scalable systems, data, reporting, and operational automation across many semi-localized service businesses.

That is exactly the kind of environment where agents could be useful if they are built around existing workflows rather than generic chatbots.

---

## 2. Business model: what these pet resorts actually do

NVA Pet Resorts is not just “dog boarding.” It is a multi-service, appointment/reservation, labor-scheduling, customer-service, and upsell business.

### Core services

#### Boarding

PetSuites boarding includes:

- Dog boarding
- Cat boarding
- Suite types such as classic/luxury
- Add-ons and upgrades
- Playtime options
- Daily housekeeping
- Potty walks
- Bedding
- “Pawgress Report” at end of stay
- Deposits and cancellation policies
- Peak/holiday minimum stays

Operational implications:

- Capacity management
- Room/suite availability
- Holiday demand spikes
- Check-in/check-out flows
- Pet profile requirements
- Medication / feeding / behavior notes
- Staff shift handoffs
- Payment/deposit handling
- Upsell opportunities: exit bath, playtime, grooming, premium suite, training

#### Daycare

Daycare includes:

- Full-day play / All Day Play
- Half-day play
- Day Boarding for dogs that are not suited to group play
- Day Play Plus Room
- Cat individual playtime
- Spay/neuter eligibility rules for group play

Operational implications:

- Temperament / eligibility tracking
- Group assignment
- Staff-to-pet ratios
- Incident tracking
- Pet health/behavior notes
- Daily recurring attendance
- Membership/package opportunities
- Fast front-desk throughput

#### Grooming

Grooming includes:

- Mini groom
- Full groom
- Exit bath
- Full bath
- Premium bath
- Nail trim / Dremel
- Ear cleaning
- Coat/skin-specific products
- First-time grooming offers

Operational implications:

- Groomer calendar optimization
- Breed/coat/time-estimate prediction
- No-show/cancellation management
- Rebooking cadence every 2–8 weeks
- Cross-sell grooming after daycare/boarding
- Customer reminders
- Groomer notes and service history

#### Training

Training includes:

- Drop-off “Stay and Study” programs
- 2-, 3-, 4-week programs
- Tutor sessions during daycare/boarding
- Group classes
- Puppy kindergarten
- Private lessons
- AKC Canine Good Citizen prep

Operational implications:

- High-value upsells
- Progress reporting
- Trainer availability
- Curriculum tracking
- Parent follow-up
- Outcome documentation
- Packages and recurring engagement

#### Retail / partner products

PetSuites lists partners like:

- Virbac CalmCare
- Purina Pro Plan Veterinary Supplements
- Purina EN as in-house diet for boarding guests

Operational implications:

- Point-of-sale sales
- Inventory
- Recommendation workflows
- Personalized upsells
- Supply chain and reorder tracking

---

## 3. Important tech clue: Gingr

The PetSuites reservation flow goes through:

> `petsuites.portal.gingrapp.com`

The loyalty page also says:

> Enrollment is automatic when you create an account in **Gingr**, your customer portal.

So Gingr appears to be a key operating/customer portal system for PetSuites, at least publicly.

**Gingr** is a common pet-care facility management platform. It generally covers things like reservations, pet/customer records, invoices, POS, packages, report cards, communications, etc.

From a candidate perspective, this matters a lot. Questions to ask:

- Do all 170 resorts use Gingr?
- Are all brands on the same Gingr instance/configuration?
- What APIs/webhooks/data exports are available?
- Is there a central data warehouse?
- Are there operational dashboards today?
- What other systems sit around Gingr? HRIS, labor scheduling, payroll, marketing automation, ticketing, data lake, BI, telephony, reviews, email/SMS, CRM, website forms, etc.

Possible ecosystem inferred from public materials/job posts:

- **Gingr**: pet-resort customer portal / reservations / operational PIMS-like system
- **Avature**: NVA careers / recruiting platform
- **GA4, Amplitude, Google Tag Manager**: mentioned in NVA Digital Analytics Manager job for broader NVA digital analytics
- **Python / analytics engineering / AI coding tools**: also mentioned in that job
- **Website network / app network** across many locations
- Likely common enterprise tools: Microsoft stack, HR/payroll, BI, ticketing, call center/telephony, email/SMS marketing — but do not assume specifics unless they say so

---

## 4. What their operational pain probably looks like

This is a multi-location services business, so the pain is probably not “we need a chatbot.” It is more likely:

### A. Labor efficiency

170 resorts likely have thousands of associates. One public LinkedIn search result for a Pet Resorts people leader referenced **5,000+ associates** supporting the Pet Resorts division. Even if approximate, labor is clearly huge.

Pain points:

- Scheduling
- Call volume
- New hire onboarding
- Training consistency
- Shift handoffs
- Policy lookups
- Staff turnover
- Manager coaching
- Daily/weekly reporting
- Payroll/labor cost control
- Safety/compliance documentation

### B. Customer communication load

Pet parents ask lots of repetitive but high-touch questions:

- “Do you have availability?”
- “What vaccines are required?”
- “Can I board both pets together?”
- “Can my dog do group play?”
- “What does my dog need for daycare?”
- “Can I add a bath?”
- “Can you send an update?”
- “What is check-out time?”
- “Can I cancel/change my reservation?”

AI opportunity:

- Assisted response drafting
- Reservation-change triage
- Policy-aware customer support
- Lead qualification
- Intake automation
- Post-stay follow-up
- Review response
- Personalized rebooking prompts

### C. Reservation / capacity optimization

Boarding/daycare/grooming/training are constrained by:

- Rooms
- Play yards
- Groomer slots
- Trainers
- Staff ratios
- Pet temperament
- Holiday peaks
- Check-in/check-out bottlenecks

AI opportunity:

- Demand forecasting
- No-show prediction
- Dynamic waitlist filling
- Capacity recommendations
- Add-on recommendations
- Holiday planning
- Over/understaffing alerts
- Revenue optimization without degrading care

### D. Data fragmentation

In a 170-site portfolio, even if the main system is Gingr, local operations likely vary.

Pain points:

- Different local practices
- Different data hygiene
- Different brands
- Inconsistent notes
- Inconsistent service packages
- Inconsistent KPIs
- Local manager reporting burden
- Regional leader visibility gaps

AI opportunity:

- Normalize messy operational data
- Generate manager summaries
- Create exception reports
- Detect outlier locations
- Surface “why did this site miss labor/revenue/CSAT?”
- Turn dashboards into action plans

### E. Sales, retention, and marketing

Public NVA Pet Resorts job listings for GMs/AGMs emphasize:

- Sales growth
- Lead generation
- Conversion
- Retention
- Memberships/services
- Digital outreach through social media, website, email
- Local business/community/referral relationships
- KPI-based management

AI opportunity:

- Lead follow-up agent
- Lapsed customer winback
- Package recommendation
- Grooming cadence reminder
- Boarding holiday campaign segmentation
- Review mining
- Local marketing content generation
- Referral source tracking
- “Next best action” for each customer/pet/account

### F. Training and standards

Pet care is emotionally sensitive. Bad automation can create safety and trust problems. But AI can be useful if it supports humans.

Pain points:

- New staff training
- SOP lookup
- Incident documentation
- Consistent handling of pet behavior notes
- Manager coaching
- Regulatory/safety policies
- Customer complaint handling

AI opportunity:

- SOP assistant
- Training quiz generator
- Shift-lead copilot
- Incident report drafting
- Policy-aware escalation guidance
- Customer complaint summarization
- “What happened today?” shift summary

---

## 5. Strong AI-agent use cases to bring up

If you want to sound immediately useful, do not pitch “we can use agents.” Pitch **bounded, high-ROI workflows**.

### 1. Resort Manager Daily Briefing Agent

Every morning, generate a brief for each GM/AGM:

- Today’s occupancy
- Daycare count
- Grooming schedule
- Training appointments
- Check-ins/check-outs
- VIP/high-risk pets
- Pets needing meds/special handling
- Staffing gaps
- Labor-to-revenue risk
- Open customer issues
- Upcoming deposits/cancellations
- Yesterday’s incidents
- Upsell opportunities

Why it matters:

- GMs are overloaded
- Reduces manual dashboard checking
- Creates consistency across 170 sites
- Easy to pilot with a few locations

### 2. Regional Ops Exception Agent

For regional leaders:

- Which locations are off-plan?
- Labor too high?
- Revenue soft?
- Leads not converting?
- Grooming slots underutilized?
- Daycare attendance dropped?
- Reviews declining?
- Incident spike?
- Cancellation spike?
- Holiday capacity risk?

Output should not be a dashboard. It should be:

> “Here are the 7 sites needing attention, why, evidence, and recommended next action.”

### 3. Customer Inbox / Call Deflection Agent

An agent that handles or drafts responses for common pet-parent requests:

- Boarding availability
- Requirements
- Cancellation rules
- Add grooming/bath
- Package questions
- New-customer onboarding
- Loyalty points
- Training options
- “My dog has anxiety; what do you recommend?”
- Escalate sensitive/medical/safety issues to humans

Important caveat:

- Keep human review for anything involving medical advice, aggressive behavior, refunds, incidents, or safety.

### 4. Lead Conversion Agent

Job listings explicitly mention lead capture, nurture, conversion, and digital channels.

Agent could:

- Watch new leads/forms/calls
- Classify intent
- Draft follow-up
- Recommend offer/service
- Remind staff to follow up
- Detect stale leads
- Summarize lead source performance
- Sync notes back to source system

### 5. Grooming Rebooking Agent

Grooming has predictable cadence. A useful agent could:

- Identify dogs due for bath/groom based on breed/service history
- Send personalized reminders
- Suggest time slots
- Bundle with daycare/boarding
- Recover lapsed grooming customers
- Fill low-utilization groomer capacity

### 6. Post-Stay / Pawgress Report Assistant

Public materials mention Pawgress Reports.

AI could help staff generate better reports from structured notes:

- “Milo played with X group, ate well, enjoyed splash pad, had medication at 2pm”
- Convert terse internal notes into warm customer-facing updates
- Flag negative/sensitive notes for review
- Enforce brand tone

### 7. Review / Reputation Agent

For 170 locations, reviews are a major signal.

Agent could:

- Monitor Google/Yelp/Facebook reviews
- Categorize issues: staff, cleanliness, pricing, injury, booking, grooming, wait time
- Draft local responses
- Escalate severe issues
- Identify repeat themes by region/brand
- Correlate review dips with staffing/incidents

### 8. SOP / Knowledge Agent

A Claude-based internal assistant trained/RAG’d on:

- Brand SOPs
- Safety rules
- Grooming policies
- Boarding policies
- Incident procedures
- Refund/cancellation policy
- Vaccination requirements
- System how-tos
- Manager playbooks

Use case:

> “A customer says their dog was scratched during group play. What do I do, what do I document, and who do I notify?”

### 9. Data Quality / Ops Hygiene Agent

Agents can help detect operational mess:

- Missing pet vaccination records
- Incomplete pet profiles
- Duplicate customers
- Missing temperament notes
- Open invoices
- Unclosed reservations
- Unused packages
- Staff notes that are too vague
- Inconsistent service naming across sites

### 10. AI Program Foundation / Platform Work

Since they are finalizing an enterprise Claude contract, they likely need someone to build the **internal AI platform layer**, not just one-off scripts:

- Claude API integration
- Authentication/authorization
- Prompt/version management
- RAG over internal docs
- Tool-use / function-calling architecture
- Audit logs
- Human-in-the-loop approval
- PII/privacy controls
- Evaluation harness
- Monitoring
- Cost controls
- Deployment templates
- Internal agent catalog

This is probably a strong differentiator: “I can build agents, but more importantly I can build the safe reusable rails for agents.”

---

## 6. What to say in the intro / first call

You want to come across as someone who understands that AI adoption in operations is mostly about **workflow integration, data access, trust, evals, and change management**.

Suggested positioning:

> “This sounds exactly like the kind of AI work I’m interested in: not novelty chatbots, but practical agents embedded into real operational workflows. For a 170-location pet-resort portfolio, I’d want to start by mapping the highest-volume repetitive workflows — customer comms, reservations, manager reporting, lead follow-up, review handling, and SOP lookup — then ship a few narrow agents with clear ROI and human-in-the-loop controls. Since you’re finalizing enterprise Claude, I’d also think about building the shared agent infrastructure early: secure data access, evals, logging, prompt/version control, and reusable integrations into systems like Gingr or whatever your core operating stack is.”

Good follow-up:

> “I’d be very interested. I’d love to understand your current systems, what workflows are burning the most time, and whether you’re looking for someone to build the AI platform, individual agents, or both.”

---

## 7. Smart discovery questions to ask him

### Business priority questions

1. What are the top 3 operational processes you most want to automate or optimize?
2. Is the goal labor savings, revenue lift, customer experience, better reporting, or all of the above?
3. Where is the biggest pain today: front desk, call center, GMs, regional ops, grooming/training, marketing, or IT?
4. Are you trying to build internal staff copilots, customer-facing agents, or back-office automation first?
5. What would make the AI program obviously successful after 90 days?

### Systems/data questions

6. What are the core systems across the pet-resort portfolio? Gingr everywhere, or mixed systems?
7. Do you have API access/webhooks/data exports from Gingr?
8. Is there already a central data warehouse or BI layer?
9. What systems handle email/SMS, phones, reviews, marketing, HR/labor scheduling, and ticketing?
10. Are customer/pet records centralized across brands or location-specific?

### Claude / architecture questions

11. Are you buying Claude for API use, Claude Enterprise seats, or both?
12. Do you already have Anthropic/Claude governance/security requirements from corporate?
13. Will agents need to run inside existing systems, Slack/Teams, web apps, or a custom internal portal?
14. Are you open to a lightweight internal agent platform, or do you already have an enterprise AI platform chosen?
15. How much autonomy are you comfortable with vs. human approval?

### Compliance/risk questions

16. What data restrictions matter most — pet parent PII, payment info, employee data, health/vaccine data?
17. Are there legal/compliance constraints around customer communications?
18. Should agents be prohibited from giving medical advice?
19. Do you need full audit trails for every AI action?
20. Who signs off on customer-facing automation?

### Engagement questions

21. Are you looking for a contractor, fractional engineer, or full-time hire?
22. Would this person own architecture and implementation, or work under an existing team?
23. Is there an internal IT/data team to partner with?
24. What’s the expected July timeline — discovery in July, pilot in July, or production build?
25. Do you have a first pilot already in mind?

---

## 8. Proposed 30/60/90-day plan

### First 30 days: discovery + pilot selection + foundation

Goals:

- Understand systems and data access
- Interview stakeholders
- Identify 3–5 high-ROI workflows
- Choose one low-risk pilot
- Define success metrics
- Build reusable Claude integration skeleton

Deliverables:

- Workflow map
- Systems/data access map
- Risk model
- Agent architecture
- Pilot spec
- Evaluation plan
- First prototype

Potential first pilots:

- Manager daily briefing
- SOP assistant
- Customer response drafting
- Lead follow-up assistant
- Review summarization/response drafting

### Days 31–60: pilot in limited locations

Goals:

- Run with 3–10 resorts
- Keep human-in-the-loop
- Instrument everything
- Compare baseline vs. AI-assisted workflow
- Collect staff feedback
- Improve prompts/tools/evals

Deliverables:

- Working pilot
- Usage analytics
- Error taxonomy
- Staff feedback
- ROI estimate
- Security/audit logs
- Rollout recommendation

### Days 61–90: productionize + second agent

Goals:

- Harden integrations
- Add monitoring/evals
- Create reusable agent framework
- Expand pilot to more locations
- Start second use case

Deliverables:

- Production agent v1
- Admin dashboard/logs
- Prompt/version management
- Documentation
- Training materials
- Rollout plan
- Roadmap for next 3–6 agents

---

## 9. What “good engineering” should mean here

This role should not be “write prompts in Claude.” The value is in building reliable systems around Claude.

### Agent design

- Narrow scopes
- Explicit tools
- Clear permissions
- Human approval for sensitive actions
- Structured outputs
- Deterministic business rules where possible
- LLM only where language/reasoning is actually useful

### Data architecture

- Source-of-truth discipline
- Do not let AI invent operational facts
- Pull facts from systems
- Log citations/source records
- Handle stale/missing data clearly
- Build data contracts around Gingr/BI/etc.

### Safety

- No medical advice
- No unauthorized refunds/promises
- No customer-facing send without approval at first
- Escalate incidents, aggression, injury, medication, legal complaints
- PII minimization
- Audit trails

### Evals

- Test set of real historical cases
- Golden answers for policy/SOP
- Regression testing for prompts
- Hallucination checks
- Tone/brand checks
- Action correctness checks
- Latency/cost monitoring

### Change management

- Staff need trust
- Managers need control
- Start as assistant/copilot, not replacement
- Make outputs editable
- Show why the agent suggested something
- Build feedback buttons directly into workflow

---

## 10. High-value AI roadmap for NVA Pet Resorts

### Phase 1: Internal copilots

Lowest risk, high adoption.

- SOP/policy assistant
- Daily manager brief
- Review summarizer
- Customer response drafting
- Lead follow-up drafting
- Incident report drafting
- Training/onboarding assistant

### Phase 2: Semi-automated operations

Human approval still present.

- Lead nurture agent
- Grooming rebooking agent
- Lapsed customer winback
- Boarding pre-arrival checklist automation
- Post-stay follow-up
- Capacity alerts
- Labor/revenue anomaly detection

### Phase 3: Customer-facing automation

Only after trust/evals are strong.

- Website reservation assistant
- FAQ / new customer intake
- Vaccination/pre-arrival document collection
- Appointment modification assistant
- Loyalty/rewards assistant
- Multi-location availability assistant

### Phase 4: Optimization / prediction

More data science-heavy.

- Demand forecasting
- Staffing recommendations
- Price/package optimization
- Churn prediction
- Customer lifetime value
- Groomer utilization optimization
- Regional performance benchmarking
- Incident risk pattern detection

---

## 11. Specific workflows to mention from their own materials

Use their language so it lands.

From PetSuites/NVA materials and job posts:

- Pawgress Reports
- Boarding reservations
- Daycare packages
- Loyalty / Pet Points Rewards
- Gingr customer portal
- Lead capture and conversion
- Website/email/social outreach
- Local market plans
- Sales, labor, expenses, customer satisfaction KPIs
- OSHA/cash handling/operational compliance
- Training/certification completion
- Resort-level EBITDA/profitability
- Grooming cadence
- Daycare eligibility rules
- Guest experience
- Team member engagement/retention

Example line:

> “I noticed your GM/AGM roles emphasize data-driven resort plans, lead conversion, labor/expense management, guest experience, and daily audits/checklists. Those are exactly the kinds of structured operational workflows where Claude agents can help — especially if they’re connected to Gingr and your reporting stack.”

---

## 12. Possible first-call pitch

You could say something like:

> “I did a little homework. NVA Pet Resorts looks like a multi-brand, multi-location operations business across boarding, daycare, grooming, and training, with Gingr at least visible in the PetSuites customer portal. The interesting AI opportunity to me isn’t a generic chatbot — it’s building agents around the repetitive workflows that happen hundreds of times a day across 170 resorts: customer inquiries, reservation changes, lead follow-up, grooming rebooking, manager daily briefs, review handling, SOP lookup, and operational exception reporting.
>
> If I were starting, I’d pick one or two high-volume workflows with clear ROI, keep humans in the loop, and build reusable Claude infrastructure around data access, audit logs, evals, and permissions so the second and third agents get cheaper and safer to launch.”

---

## 13. Suggested intro response to your friend

Polished version:

> Yeah, definitely interested — please make the intro. This sounds right up my alley: practical AI/agent work inside a real operations-heavy business rather than toy chatbot stuff. For a 170-location pet-resort portfolio, I’d be especially interested in workflows like customer comms, reservation/lead follow-up, manager reporting, SOP assistants, review handling, grooming/daycare rebooking, and ops exception reporting.
>
> Happy to chat this week and understand what systems they’re using, what Claude setup they’re finalizing, and whether they need someone to build individual agents, the underlying agent platform, or both.

More casual version:

> Yeah, T it up. That sounds super interesting — especially because it’s real-world ops automation across a big multi-location business, not just “add AI” fluff. I’d love to hear what systems they’re using, what workflows are most painful, and whether they need agent/platform architecture, hands-on implementation, or both.

---

## 14. Things to be careful about

### Do not over-index on “veterinary AI”

This is Pet Resorts, not primarily diagnosis/treatment. Avoid pitching medical AI unless they ask. Their pain is probably:

- Ops
- Labor
- Customer experience
- Marketing
- Reporting
- Training
- Multi-site standardization

### Do not pitch full autonomy too early

For pet care, trust matters. Pitch:

- Human-in-the-loop
- Draft/recommend/triage
- Logs and approvals
- Escalation rules

### Do not assume their systems

You can say:

> “I saw Gingr in the PetSuites public reservation flow — curious whether that’s universal across the portfolio.”

Not:

> “Since you use Gingr everywhere…”

### Do not sell “AI transformation” abstractions

Sell concrete outputs:

- Reduce front desk response load
- Fill grooming openings
- Increase lead conversion
- Give GMs daily action briefs
- Summarize reviews and incidents
- Standardize SOP answers

---

## 15. Best angle for the engineer

The highest-value framing is:

> “I can help you turn enterprise Claude into safe, measurable operational agents.”

Not:

> “I know how to use Claude.”

The value stack:

1. Understand messy business workflows
2. Integrate with real systems
3. Build agent tools/actions
4. Put guardrails around them
5. Measure ROI
6. Iterate with operators
7. Create reusable infrastructure so the AI program scales

That is probably exactly what someone “newly leading AI adoption efforts” needs.

# Notes



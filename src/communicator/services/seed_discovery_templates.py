"""
Seeds the first tenant's (NEWAX's) discovery flow template as data.
Any other tenant onboards the same way: insert another
DiscoveryFlowTemplate row with its own tenant_key -- the engine in
discovery_flow.py never changes.
"""

from __future__ import annotations

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.discovery_flow_template import DiscoveryFlowTemplate

NEWAX_TEMPLATE = {
    "tenant_key": "newax",
    "name": "NEWAX Business Inquiry Discovery",
    "intro_message": (
        "Great — let's figure out what you need. A few quick questions, then I'll have "
        "the right person follow up."
    ),
    "closing_message": (
        "Thanks — that gives us a clear picture. I've passed this to our team, and someone "
        "will follow up shortly to go deeper and figure out the right next step. If anything "
        "changes in the meantime, just message us here."
    ),
    "steps": [
        {
            "key": "organization_name",
            "question": "Happy to help. What's your organization called? If you're reaching out on your own behalf, just say so.",
            "free_text": True,
        },
        {
            "key": "industry",
            "question": "What industry are you in?",
            "options": [
                ["healthcare", "Healthcare"],
                ["education", "Education"],
                ["real_estate", "Real Estate"],
                ["legal", "Legal"],
                ["retail", "Retail"],
                ["hospitality", "Hospitality"],
                ["financial_services", "Financial Services"],
                ["professional_services", "Professional Services"],
                ["logistics", "Transport / Logistics"],
                ["manufacturing", "Manufacturing"],
                ["other", "Other"],
            ],
        },
        {
            "key": "contact_role",
            "question": "And what's your role there?",
            "options": [
                ["owner", "Founder / Owner"],
                ["executive", "Executive"],
                ["operations", "Operations"],
                ["it", "IT / Technology"],
                ["finance", "Finance"],
                ["sales_marketing", "Sales / Marketing"],
                ["employee", "Employee / Team Member"],
                ["other", "Other"],
            ],
        },
        {
            "key": "org_size_range",
            "question": "Roughly how many people are in your organization?",
            "options": [
                ["1_10", "1–10"],
                ["11_50", "11–50"],
                ["51_200", "51–200"],
                ["201_500", "201–500"],
                ["500_plus", "500+"],
            ],
        },
        {
            "key": "problem_description",
            "question": "Now the important part: what's the main operational problem you're trying to solve? Describe it however makes sense to you.",
            "free_text": True,
        },
        {
            "key": "reported_impact",
            "question": "How is this affecting your team day to day?",
            "options": [
                ["wastes_time", "Wastes employee time"],
                ["increases_costs", "Increases costs"],
                ["causes_errors", "Causes errors / duplicate work"],
                ["delays_decisions", "Delays decisions"],
                ["reduces_satisfaction", "Reduces customer satisfaction"],
                ["limits_visibility", "Limits management visibility"],
                ["other", "Other"],
            ],
        },
        {
            "key": "frequency",
            "question": "How often does this come up?",
            "options": [
                ["daily", "Daily"],
                ["weekly", "Weekly"],
                ["monthly", "Monthly"],
                ["occasionally", "Occasionally"],
            ],
        },
        {
            "key": "desired_outcome",
            "question": "If this were solved, what would improve most for you?",
            "options": [
                ["save_time", "Save time"],
                ["reduce_costs", "Reduce costs"],
                ["improve_accuracy", "Improve accuracy"],
                ["real_time_visibility", "Real-time visibility"],
                ["connect_systems", "Connect departments/systems"],
                ["other", "Something else"],
            ],
        },
        {
            "key": "timeline",
            "question": "When would you like to get started?",
            "options": [
                ["immediately", "Immediately"],
                ["2_4_weeks", "Within 2–4 weeks"],
                ["1_3_months", "Within 1–3 months"],
                ["3_6_months", "Within 3–6 months"],
                ["exploring", "Just exploring for now"],
            ],
        },
        {
            "key": "budget_range",
            "question": "Last one — do you have a rough budget in mind? This just helps us recommend the right approach, no pressure if you're not sure yet.",
            "options": [
                ["under_5k", "Under $5,000"],
                ["5k_20k", "$5,000–$20,000"],
                ["20k_50k", "$20,000–$50,000"],
                ["50k_plus", "$50,000+"],
                ["not_determined", "Not determined yet"],
            ],
        },
    ],
    "problem_category_keywords": {
        "disconnected_systems": [
            "separate system",
            "disconnect",
            "don't talk",
            "doesn't sync",
            "siloed",
        ],
        "manual_work": ["manual", "re-enter", "retype", "duplicate entry", "by hand"],
        "poor_visibility": ["no visibility", "can't see", "don't know status", "no view"],
        "limited_reporting": ["report", "reporting takes", "no reports"],
        "workflow_inefficiency": ["approval", "slow process", "bottleneck", "handover"],
        "data_management": ["duplicate record", "inconsistent data", "bad data"],
        "customer_experience": ["slow response", "customer complain", "support ticket"],
        "physical_operations": ["device", "sensor", "equipment", "facility", "iot"],
    },
    "capability_keywords": {
        "enterprise_software": [
            "portal",
            "application",
            "system",
            "platform",
            "software",
            "erp",
            "crm",
        ],
        "intelligent_automation": [
            "automat",
            "workflow",
            "integrat",
            "notification",
            "report",
            "sync",
        ],
        "connected_infrastructure": [
            "iot",
            "sensor",
            "device",
            "monitor",
            "telemetry",
            "equipment",
        ],
    },
    "capability_labels": {
        "enterprise_software": "Enterprise Software",
        "intelligent_automation": "Intelligent Automation",
        "connected_infrastructure": "Connected Infrastructure",
    },
    "scoring_weights": {
        "problem_clarity_max": 20,
        "operational_impact_max": 20,
        "capability_alignment_max": 20,
        "decision_access_by_role": {
            "owner": 15,
            "executive": 15,
            "operations": 10,
            "it": 10,
            "finance": 8,
            "sales_marketing": 8,
            "employee": 5,
            "other": 5,
        },
        "decision_access_default": 5,
        "timeline_readiness_by_value": {
            "immediately": 15,
            "2_4_weeks": 13,
            "1_3_months": 10,
            "3_6_months": 6,
            "exploring": 2,
        },
        "timeline_readiness_default": 2,
        "budget_alignment_by_value": {
            "50k_plus": 10,
            "20k_50k": 9,
            "5k_20k": 7,
            "under_5k": 4,
            "not_determined": 3,
        },
        "budget_alignment_default": 3,
    },
}


async def seed_newax_discovery_template(session: AsyncSession) -> DiscoveryFlowTemplate:
    existing = (
        await session.execute(
            select(DiscoveryFlowTemplate).where(
                DiscoveryFlowTemplate.tenant_key == NEWAX_TEMPLATE["tenant_key"]
            )
        )
    ).scalar_one_or_none()
    if existing is not None:
        for key, value in NEWAX_TEMPLATE.items():
            setattr(existing, key, value)
        await session.flush()
        return existing

    template = DiscoveryFlowTemplate(**NEWAX_TEMPLATE)
    session.add(template)
    await session.flush()
    return template

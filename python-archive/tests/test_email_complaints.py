from datetime import UTC, datetime

import pytest
from sqlalchemy import select
from sqlalchemy.exc import IntegrityError

from communicator.models.email_complaint import EmailComplaint, EmailComplaintType


@pytest.mark.asyncio
async def test_email_complaint_persists_normalized_evidence(db_session):
    complaint = EmailComplaint(
        source_event_id="postmark:complaint:evt-001",
        complaint_type=EmailComplaintType.RECIPIENT_SPAM,
        provider="postmark",
        provider_message_id="provider-message-001",
        recipient_email="recipient@example.com",
        dmarc_result="pass",
        spf_aligned=True,
        dkim_aligned=True,
        evidence_metadata={"record_type": "spam_complaint"},
        suppression_applied=True,
        reported_at=datetime(2026, 8, 4, tzinfo=UTC),
    )
    db_session.add(complaint)
    await db_session.flush()

    stored = await db_session.scalar(
        select(EmailComplaint).where(EmailComplaint.source_event_id == complaint.source_event_id)
    )

    assert stored is not None
    assert stored.complaint_type is EmailComplaintType.RECIPIENT_SPAM
    assert stored.recipient_email == "recipient@example.com"
    assert stored.suppression_applied is True


@pytest.mark.asyncio
async def test_email_complaint_source_event_is_idempotent(db_session):
    values = {
        "source_event_id": "dmarc:forensic:evt-002",
        "complaint_type": EmailComplaintType.DMARC_FORENSIC,
        "provider": "dmarc_mailbox",
        "recipient_email": "recipient@example.com",
        "reported_at": datetime(2026, 8, 4, tzinfo=UTC),
    }
    db_session.add(EmailComplaint(**values))
    await db_session.flush()
    db_session.add(EmailComplaint(**values))

    with pytest.raises(IntegrityError):
        await db_session.flush()

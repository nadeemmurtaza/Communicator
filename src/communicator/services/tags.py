from __future__ import annotations

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.contact import Contact
from communicator.models.tag import Tag


async def get_or_create_tag(session: AsyncSession, name: str, tenant_key: str = "newax") -> Tag:
    """Tags are unique per tenant, not globally -- two tenants can each have
    their own "vip-customers" tag without colliding."""
    stmt = select(Tag).where(Tag.name == name, Tag.tenant_key == tenant_key)
    tag = (await session.execute(stmt)).scalar_one_or_none()
    if tag is None:
        tag = Tag(name=name, tenant_key=tenant_key)
        session.add(tag)
        await session.flush()
    return tag


async def add_tag_to_contact(session: AsyncSession, contact: Contact, tag_name: str) -> None:
    tag = await get_or_create_tag(session, tag_name, tenant_key=contact.tenant_key)
    await session.refresh(contact, attribute_names=["tags"])
    if tag not in contact.tags:
        contact.tags.append(tag)
        await session.flush()


async def remove_tag_from_contact(session: AsyncSession, contact: Contact, tag_name: str) -> None:
    await session.refresh(contact, attribute_names=["tags"])
    contact.tags = [t for t in contact.tags if t.name != tag_name]
    await session.flush()

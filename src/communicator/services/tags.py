from __future__ import annotations

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.contact import Contact
from communicator.models.tag import Tag


async def get_or_create_tag(session: AsyncSession, name: str) -> Tag:
    stmt = select(Tag).where(Tag.name == name)
    tag = (await session.execute(stmt)).scalar_one_or_none()
    if tag is None:
        tag = Tag(name=name)
        session.add(tag)
        await session.flush()
    return tag


async def add_tag_to_contact(session: AsyncSession, contact: Contact, tag_name: str) -> None:
    tag = await get_or_create_tag(session, tag_name)
    await session.refresh(contact, attribute_names=["tags"])
    if tag not in contact.tags:
        contact.tags.append(tag)
        await session.flush()

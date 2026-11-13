# ERP-1 (yaseenecom) Schema Reference

This folder contains the Prisma schema files from the ERP-1 system (yaseenecom) for reference when building the sync bridge between the Communicator and the ERP.

## ERP Database Connection

- **Name**: `ERP-1 (yaseenecom)`
- **DBMS**: PostgreSQL 16.14
- **Host**: `localhost:5432`
- **Database**: `yaseen`
- **Connection ID**: `f3e6cb11-a98c-409c-bd63-f1828f9a6887`

## Schema Files

The ERP uses 61 Prisma schema files, one per module:

| Category | Key Files |
|----------|-----------|
| **Core Commerce** | `catalog.prisma`, `orders.prisma`, `cart.prisma`, `payments.prisma`, `pricing.prisma` |
| **Inventory** | `inventory.prisma`, `transfers.prisma`, `bom.prisma`, `production.prisma` |
| **Fulfilment** | `fulfilment.prisma` |
| **Accounting** | `accounting.prisma`, `finance.prisma`, `invoicing.prisma`, `receivables.prisma`, `settlements.prisma`, `tax.prisma` |
| **Purchasing** | `purchasing.prisma` |
| **CRM** | `crm.prisma`, `parties.prisma` |
| **HR** | `hr.prisma`, `payroll.prisma`, `attendance.prisma`, `approvals.prisma` |
| **Marketing** | `marketing.prisma`, `notify.prisma`, `notification-preferences.prisma` |
| **POS** | `pos.prisma` |
| **Customer Features** | `loyalty.prisma`, `reviews.prisma`, `wishlist.prisma`, `giftcards.prisma` |
| **Advanced** | `embeds.prisma`, `search.prisma`, `rag.prisma`, `bpmn-workflows.prisma`, `sso.prisma` |
| **WhatsApp** | `whatsapp-assistant.prisma` |
| **Config** | `schema.prisma` (datasource), `core.prisma` (shared types), `organization.prisma` |

## Integration Plan

The Communicator will connect to this ERP database directly to:
1. **Read** product catalog, inventory, and pricing from the ERP
2. **Write** orders placed via communication channels (WhatsApp, Instagram, Telegram, Email) to the ERP
3. **Sync** order status, payments, and inventory changes bidirectionally

### Key Overlap Modules

| ERP Module | Communicator Equivalent | Integration Approach |
|------------|------------------------|---------------------|
| `catalog.prisma` | `products` table | Read from ERP, sync to our SQLite cache |
| `orders.prisma` | `commerce_orders` table | Write orders from our channels to ERP |
| `inventory.prisma` | `inventory_count` field | Read inventory from ERP, update on order |
| `payments.prisma` | `commerce_payments` table | Read payment methods, write payment proofs |
| `loyalty.prisma` | `loyalty_programs` table | Read loyalty data from ERP |
| `reviews.prisma` | `customer_reviews` table | Read reviews from ERP |
| `pos.prisma` | Gap #18 POS system | Use ERP's POS instead of building ours |

## Next Steps

1. Build PostgreSQL connection bridge in Rust backend
2. Create sync adapter for ERP (similar to Shopify/WooCommerce adapters)
3. Wire channel templates to ERP product data
4. Test end-to-end: WhatsApp order → ERP database

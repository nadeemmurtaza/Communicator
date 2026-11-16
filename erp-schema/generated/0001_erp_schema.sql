-- ERP Schema generated from Prisma models
-- Generated on 2026-11-13 22:25:58

CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TYPE adminscopelevel AS ENUM ('org', 'brand', 'branch');

CREATE TYPE quantitysource AS ENUM ('typed', 'scale_weight', 'bom_recipe_scale');

CREATE TYPE approvalsubjecttype AS ENUM ('purchase_requisition', 'wallet_adjustment', 'b2b_order', 'broadcast_message');

CREATE TYPE merchantpayablestatus AS ENUM ('open', 'scheduled', 'paid', 'disputed');

CREATE TYPE spotcheckstatus AS ENUM ('pending', 'in_progress', 'completed', 'cancelled');

CREATE TYPE leavekind AS ENUM ('annual', 'sick', 'unpaid', 'other');

CREATE TYPE markdownsuggestionstatus AS ENUM ('pending', 'created', 'dismissed');

CREATE TYPE reviewstatus AS ENUM ('pending', 'published', 'rejected');

CREATE TYPE campaignrecipientstatus AS ENUM ('pending', 'sent', 'skipped_opt_out', 'skipped_no_contact', 'failed', 'opened', 'clicked');

CREATE TYPE tradingpartnerstatus AS ENUM ('pending', 'active', 'suspended');

CREATE TYPE auctionsettlementstatus AS ENUM ('invoicing', 'invoice_failed', 'awaiting_payment', 'paid', 'expired', 'cancelled');

CREATE TYPE bankfacilitycovenanttype AS ENUM ('max_utilization_bps', 'max_leverage', 'min_dscr');

CREATE TYPE settlementbatchstatus AS ENUM ('expected', 'confirmed', 'short', 'disputed');

CREATE TYPE partnerfactstatus AS ENUM ('received', 'applied', 'needs_review', 'rejected');

CREATE TYPE returnresolution AS ENUM ('refund', 'exchange', 'store_credit', 'repair', 'rejected');

CREATE TYPE walletadjustmentrequeststatus AS ENUM ('pending', 'approved', 'rejected');

CREATE TYPE ingeststatus AS ENUM ('queued', 'extracting', 'chunking', 'embedding', 'ready', 'failed');

CREATE TYPE tradingpartnerkind AS ENUM ('supplier', 'customer', 'both');

CREATE TYPE approvalstepstatus AS ENUM ('pending', 'approved', 'rejected', 'skipped');

CREATE TYPE whatsappmessagedirection AS ENUM ('inbound', 'outbound');

CREATE TYPE ticketstatus AS ENUM ('open', 'in_progress', 'resolved', 'closed');

CREATE TYPE accounttype AS ENUM ('asset', 'liability', 'equity', 'income', 'expense');

CREATE TYPE cartstockholdtier AS ENUM ('soft', 'hard');

CREATE TYPE campaignstatus AS ENUM ('draft', 'scheduled', 'sending', 'sent', 'cancelled');

CREATE TYPE cashmovementtype AS ENUM ('opening_float', 'cash_in', 'cash_out', 'drop', 'paid_out');

CREATE TYPE bankfacilitytype AS ENUM ('term_loan', 'running_finance', 'overdraft', 'letter_of_credit', 'bank_guarantee');

CREATE TYPE documentcorpus AS ENUM ('employee_documents', 'supplier_catalog', 'business_documents', 'knowledge_base');

CREATE TYPE interactionkind AS ENUM ('call', 'whatsapp', 'email', 'sms', 'visit', 'note', 'complaint');

CREATE TYPE aicallkind AS ENUM ('completion', 'embedding', 'vision');

CREATE TYPE measurementunit AS ENUM ('inch', 'cm');

CREATE TYPE aisuggestionkind AS ENUM ('alt_text', 'meta_title', 'meta_description', 'description', 'care_instructions');

CREATE TYPE idempotencykeystatus AS ENUM ('in_progress', 'completed');

CREATE TYPE purchaseorderstatus AS ENUM ('draft', 'sent', 'partial', 'received', 'cancelled');

CREATE TYPE bidkind AS ENUM ('manual', 'proxy_auto', 'sealed');

CREATE TYPE replicationquarantinestatus AS ENUM ('open', 'resolved');

CREATE TYPE depreciationmethod AS ENUM ('straight_line', 'declining_balance');

CREATE TYPE settlementmode AS ENUM ('none', 'internal_pl', 'invoiced');

CREATE TYPE paymentstatus AS ENUM ('pending', 'awaiting_approval', 'paid', 'failed', 'refunded', 'cancelled');

CREATE TYPE payrollbankaccounttype AS ENUM ('checking', 'savings');

CREATE TYPE returnstatus AS ENUM ('requested', 'approved', 'rejected', 'in_transit', 'received', 'inspected', 'refunded', 'closed');

CREATE TYPE userrole AS ENUM ('customer', 'staff', 'tailor', 'admin');

CREATE TYPE auctiontype AS ENUM ('english', 'sealed_bid', 'reverse');

CREATE TYPE bomstatus AS ENUM ('draft', 'active', 'archived');

CREATE TYPE ticketpriority AS ENUM ('low', 'normal', 'high', 'urgent');

CREATE TYPE manifeststatus AS ENUM ('open', 'dispatched');

CREATE TYPE bankfacilitystatus AS ENUM ('active', 'closed');

CREATE TYPE supportmessageauthortype AS ENUM ('customer', 'staff');

CREATE TYPE walletentrytype AS ENUM ('purchase', 'reserve', 'redeem', 'release_reservation', 'refund', 'adjustment');

CREATE TYPE leaverequeststatus AS ENUM ('pending', 'approved', 'rejected', 'cancelled');

CREATE TYPE auctionregistration AS ENUM ('open', 'verified', 'approval');

CREATE TYPE serializedunitstatus AS ENUM ('in_stock', 'sold', 'returned', 'rma_to_supplier', 'written_off');

CREATE TYPE expenserecurrence AS ENUM ('none', 'weekly', 'monthly', 'quarterly', 'yearly');

CREATE TYPE attendancebreaktype AS ENUM ('meal', 'rest');

CREATE TYPE approvalchainstatus AS ENUM ('pending', 'approved', 'rejected');

CREATE TYPE invoicestatus AS ENUM ('issued', 'paid', 'void');

CREATE TYPE clearancerole AS ENUM ('hr_admin', 'it_admin', 'facilities', 'finance', 'manager', 'employee');

CREATE TYPE stitchingorderstatus AS ENUM ('draft', 'awaiting_fabric', 'fabric_received', 'in_production', 'ready', 'completed', 'cancelled');

CREATE TYPE replicationclass AS ENUM ('fact', 'policy');

CREATE TYPE deviceplatform AS ENUM ('ios', 'android', 'web');

CREATE TYPE customerticketstatus AS ENUM ('open', 'awaiting_customer', 'in_progress', 'resolved', 'closed');

CREATE TYPE stitchingstage AS ENUM ('received', 'cutting', 'stitching', 'qc', 'ready', 'dispatched');

CREATE TYPE orderitemfulfillmentstatus AS ENUM ('ready', 'awaiting_stock', 'awaiting_production');

CREATE TYPE apikeysurface AS ENUM ('admin', 'storefront');

CREATE TYPE posordertype AS ENUM ('counter', 'tab', 'table');

CREATE TYPE auctionlotstatus AS ENUM ('pending', 'open', 'closed', 'withdrawn');

CREATE TYPE workordersource AS ENUM ('reorder', 'order_cascade', 'manual');

CREATE TYPE attendanceexceptionreason AS ENUM ('no_matching_shift', 'outside_roster', 'wrong_unit', 'while_on_leave', 'duplicate_suspected', 'missing_counterpart', 'outside_geofence', 'gps_spoofing_suspected');

CREATE TYPE nodestatus AS ENUM ('active', 'suspended');

CREATE TYPE onsitepopuptrigger AS ENUM ('immediate', 'timed', 'exit_intent');

CREATE TYPE instalmentplanstatus AS ENUM ('active', 'completed', 'defaulted', 'cancelled');

CREATE TYPE quotestatus AS ENUM ('sent', 'accepted', 'rejected', 'expired');

CREATE TYPE fiscalperiodstatus AS ENUM ('open', 'closed', 'locked');

CREATE TYPE customfieldvaluetype AS ENUM ('text', 'number', 'boolean', 'date', 'select', 'multi_select', 'json');

CREATE TYPE stockvariancestatus AS ENUM ('open', 'resolved');

CREATE TYPE notificationpreferencechannel AS ENUM ('email', 'sms', 'whatsapp', 'push');

CREATE TYPE journalentrystatus AS ENUM ('posted', 'reversed');

CREATE TYPE stocktransferstatus AS ENUM ('in_transit', 'received', 'discrepancy', 'reconciled', 'cancelled');

CREATE TYPE compensationbasis AS ENUM ('monthly', 'hourly', 'commission');

CREATE TYPE auctionstatus AS ENUM ('draft', 'scheduled', 'open', 'closing', 'closed', 'settled', 'cancelled');

CREATE TYPE permissioneffect AS ENUM ('grant', 'revoke');

CREATE TYPE apikeyrevocationreason AS ENUM ('owner', 'administrator', 'owner_deactivated');

CREATE TYPE stitchingselectiontype AS ENUM ('single', 'multi');

CREATE TYPE replicationop AS ENUM ('upsert', 'delete');

CREATE TYPE stockcountstatus AS ENUM ('draft', 'in_progress', 'completed', 'cancelled');

CREATE TYPE supplierbillstatus AS ENUM ('unpaid', 'partial', 'paid', 'pending_match_review');

CREATE TYPE aisuggestionstatus AS ENUM ('pending', 'approved', 'rejected');

CREATE TYPE posshiftstatus AS ENUM ('open', 'closed');

CREATE TYPE scopelevel AS ENUM ('org', 'brand', 'branch');

CREATE TYPE onsitemessagekind AS ENUM ('announcement_bar', 'popup', 'newsletter_capture', 'cookie_consent');

CREATE TYPE productkind AS ENUM ('goods', 'service', 'digital', 'stitched', 'unstitched', 'stitching_service');

CREATE TYPE discounttype AS ENUM ('percentage', 'fixed_amount', 'free_shipping', 'buy_x_get_y', 'tiered');

CREATE TYPE payrunstatus AS ENUM ('draft', 'approved', 'posted');

CREATE TYPE aifeature AS ENUM ('alt_text', 'seo_copy', 'size_recommendation', 'demand_forecast', 'cod_risk', 'semantic_search', 'visual_search', 'recommendations', 'whatsapp_assistant', 'markdown_suggestion', 'review_summary', 'import_cleanup', 'rag_ingestion', 'rag_retrieval', 'rag_generation', 'planning_narration');

CREATE TYPE posterminalstatus AS ENUM ('pending', 'active', 'suspended');

CREATE TYPE creditdebitnotestatus AS ENUM ('draft', 'issued', 'applied', 'void');

CREATE TYPE optiondisplay AS ENUM ('button', 'dropdown', 'swatch', 'text');

CREATE TYPE notificationpreferencecategory AS ENUM ('stitching_progress', 'marketing', 'back_in_stock');

CREATE TYPE requisitionapprovalstatus AS ENUM ('pending', 'approved', 'rejected', 'converted');

CREATE TYPE payrunadjustmentreason AS ENUM ('overpayment', 'underpayment', 'correction');

CREATE TYPE lifecyclechecklisttype AS ENUM ('onboarding', 'offboarding', 'promotion', 'transfer');

CREATE TYPE shipmentstatus AS ENUM ('pending', 'dispatched', 'in_transit', 'delivered', 'returned', 'lost');

CREATE TYPE fragmentstatus AS ENUM ('active', 'suspended');

CREATE TYPE discountscope AS ENUM ('order', 'product', 'category', 'collection', 'shipping');

CREATE TYPE productstatus AS ENUM ('draft', 'active', 'archived');

CREATE TYPE contentstatus AS ENUM ('draft', 'published', 'archived');

CREATE TYPE identityverificationstatus AS ENUM ('not_started', 'pending', 'approved', 'rejected', 'expired');

CREATE TYPE contactstatus AS ENUM ('lead', 'active', 'repeat', 'dormant', 'blocked');

CREATE TYPE rawmaterialunit AS ENUM ('gaz', 'metre', 'piece', 'spool', 'gram', 'millilitre');

CREATE TYPE productcrosssellkind AS ENUM ('manual', 'co_occurrence', 'embedding');

CREATE TYPE ssoconnectionstatus AS ENUM ('active', 'suspended');

CREATE TYPE stockmovementreason AS ENUM ('purchase_receipt', 'sale', 'cancellation', 'return', 'adjustment', 'damage', 'count', 'write_off', 'production_consumption', 'production_receipt', 'pos_sale', 'pos_offline_variance', 'transfer_out', 'transfer_in');

CREATE TYPE outboxstatus AS ENUM ('pending', 'processing', 'done', 'failed');

CREATE TYPE bankstatementlinetype AS ENUM ('credit', 'debit');

CREATE TYPE replicationdirection AS ENUM ('up', 'down');

CREATE TYPE loyaltytransactiontype AS ENUM ('earned', 'redeemed', 'reversed', 'adjusted');

CREATE TYPE shippingratetype AS ENUM ('flat', 'by_weight', 'by_price', 'by_item');

CREATE TYPE noderole AS ENUM ('branch', 'brand');

CREATE TYPE giftcardstatus AS ENUM ('active', 'redeemed', 'expired', 'cancelled');

CREATE TYPE leadstatus AS ENUM ('new', 'contacted', 'quoted', 'won', 'lost');

CREATE TYPE partyrolename AS ENUM ('supplier', 'manufacturer', 'distributor', 'retailer');

CREATE TYPE walletaccountstatus AS ENUM ('active', 'frozen', 'closed');

CREATE TYPE grantmode AS ENUM ('read', 'acting');

CREATE TYPE bidderstatus AS ENUM ('pending', 'approved', 'suspended', 'declined');

CREATE TYPE orderchannel AS ENUM ('web', 'pos');

CREATE TYPE attendancesource AS ENUM ('device', 'self', 'manager', 'import');

CREATE TYPE attendancekind AS ENUM ('in', 'out', 'break_start', 'break_end', 'adjustment');

CREATE TYPE instalmentplantype AS ENUM ('layby', 'instalment');

CREATE TYPE ssoprotocol AS ENUM ('oidc');

CREATE TYPE lifecyclecheckliststatus AS ENUM ('pending', 'in_progress', 'completed', 'blocked');

CREATE TYPE workorderstatus AS ENUM ('draft', 'released', 'completed', 'cancelled');

CREATE TYPE emailaccountpurpose AS ENUM ('transactional', 'marketing', 'support');

CREATE TYPE locationtype AS ENUM ('store', 'warehouse', 'in_transit', 'production');

CREATE TYPE bulletincategory AS ENUM ('policy', 'memo', 'notice', 'hr_announcement');

CREATE TYPE employmentstatus AS ENUM ('active', 'on_leave', 'suspended', 'left');

CREATE TYPE attendancedevicestatus AS ENUM ('pending', 'active', 'suspended');

CREATE TYPE codriskaction AS ENUM ('auto_confirm', 'confirmation_call', 'require_advance_payment');

CREATE TYPE inviteresponse AS ENUM ('no_response', 'accepted', 'declined', 'tentative');

CREATE TYPE taskstate AS ENUM ('open', 'in_progress', 'blocked', 'done', 'cancelled');

CREATE TYPE collectionsortstrategy AS ENUM ('manual', 'best_selling', 'newest', 'price_asc', 'price_desc', 'margin');

CREATE TYPE campaignchannel AS ENUM ('email', 'sms', 'whatsapp');

CREATE TYPE orderstatus AS ENUM ('pending', 'confirmed', 'processing', 'ready_to_ship', 'shipped', 'delivered', 'cancelled', 'returned');

CREATE TYPE cartstockholdstatus AS ENUM ('active', 'expired', 'released', 'consumed');

CREATE TYPE b2borderrequeststatus AS ENUM ('pending', 'approved', 'rejected', 'converted');

CREATE TYPE assetstatus AS ENUM ('available', 'assigned', 'retired', 'lost');

CREATE TABLE IF NOT EXISTS "returnrequestnumbercounter" (
    "year" INTEGER PRIMARY KEY,
    "counter" INTEGER DEFAULT 0 NOT NULL
);

CREATE TABLE IF NOT EXISTS "ticketmessage" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "ticketId" TEXT NOT NULL,
    "authorId" TEXT NOT NULL,
    "body" TEXT NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "productoptionvaluetranslation" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "valueId" TEXT NOT NULL,
    "locale" TEXT NOT NULL,
    "label" TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS "interbranchinvoicenumbercounter" (
    "year" INTEGER PRIMARY KEY,
    "counter" INTEGER DEFAULT 0 NOT NULL
);

CREATE TABLE IF NOT EXISTS "searchquerylog" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "locale" TEXT NOT NULL,
    "term" TEXT NOT NULL,
    "resultCount" INTEGER NOT NULL,
    "userId" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "compensationrate" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "userId" TEXT NOT NULL,
    "basis" compensationbasis NOT NULL,
    "amountMinor" INTEGER NOT NULL,
    "effectiveFrom" TIMESTAMPTZ NOT NULL,
    "effectiveTo" TIMESTAMPTZ,
    "createdById" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "productreviewsummary" (
    "productId" TEXT PRIMARY KEY,
    "summaryText" TEXT NOT NULL,
    "summaryTextUr" TEXT,
    "fitSignal" TEXT,
    "qualitySignal" TEXT,
    "commonComplaints" JSONB,
    "reviewCountAtGeneration" INTEGER NOT NULL,
    "generatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "cartitemstitchingoption" (
    "cartItemId" TEXT NOT NULL,
    "optionId" TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS "shippingrate" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "zoneId" TEXT NOT NULL,
    "methodId" TEXT NOT NULL,
    "type"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`shippingratetype' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "fee" INTEGER NOT NULL,
    "baseFee" INTEGER DEFAULT 0 NOT NULL,
    "includedUnits" INTEGER DEFAULT 0 NOT NULL,
    "freeAbove" INTEGER,
    "minValue" INTEGER,
    "maxValue" INTEGER,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL
);

CREATE TABLE IF NOT EXISTS "broadcastmessage" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "senderId" TEXT NOT NULL,
    "audienceId" TEXT NOT NULL,
    "subject" TEXT NOT NULL,
    "body" TEXT NOT NULL,
    "recipientCount" INTEGER NOT NULL,
    "excludedCount" INTEGER DEFAULT 0 NOT NULL,
    "approvalChainId" TEXT UNIQUE,
    "sentAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "inboundemailreply" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "inboundEmailId" TEXT NOT NULL,
    "actorId" TEXT,
    "body" TEXT NOT NULL,
    "sentAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "returnreasontranslation" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "reasonId" TEXT NOT NULL,
    "locale" TEXT NOT NULL,
    "label" TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS "payrunadjustment" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "payRunId" TEXT NOT NULL,
    "userId" TEXT NOT NULL,
    "reason" payrunadjustmentreason NOT NULL,
    "amountMinor" INTEGER NOT NULL,
    "note" TEXT NOT NULL,
    "createdById" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "productcrosssell" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "productId" TEXT NOT NULL,
    "relatedProductId" TEXT NOT NULL,
    "kind" productcrosssellkind NOT NULL,
    "order" INTEGER DEFAULT 0 NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "productbundle" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "variantId" TEXT NOT NULL UNIQUE,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "workorder" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "number" TEXT NOT NULL UNIQUE,
    "variantId" TEXT NOT NULL,
    "quantityOrdered" INTEGER NOT NULL,
    "quantityCompleted" INTEGER DEFAULT 0 NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`workorderstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "source" workordersource NOT NULL,
    "sourceRefType" TEXT,
    "sourceRefId" TEXT,
    "locationId" TEXT,
    "notes" TEXT,
    "releasedAt" TIMESTAMPTZ,
    "completedAt" TIMESTAMPTZ,
    "cancelledAt" TIMESTAMPTZ,
    "createdById" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "auctionsettlement" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "lotId" TEXT NOT NULL,
    "auctionId" TEXT NOT NULL,
    "bidderId" TEXT NOT NULL,
    "userId" TEXT NOT NULL,
    "attempt" INTEGER DEFAULT 1 NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`auctionsettlementstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "currency" TEXT NOT NULL,
    "hammerMinor" TEXT NOT NULL,
    "premiumBps" INTEGER DEFAULT 0 NOT NULL,
    "premiumMinor" TEXT DEFAULT 0 NOT NULL,
    "totalMinor" TEXT NOT NULL,
    "paidMinor" TEXT DEFAULT 0 NOT NULL,
    "dueAt" TIMESTAMPTZ NOT NULL,
    "orderId" TEXT,
    "orderNumber" TEXT,
    "failureReason" TEXT,
    "resolvedAt" TIMESTAMPTZ,
    "resolvedById" TEXT,
    "resolutionNote" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "expensecategory" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "key" TEXT NOT NULL UNIQUE,
    "name" TEXT NOT NULL,
    "isDirect" BOOLEAN DEFAULT FALSE NOT NULL,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "order" INTEGER DEFAULT 0 NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "userpermission" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "userId" TEXT NOT NULL,
    "permissionId" TEXT NOT NULL,
    "effect" permissioneffect NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "bidder" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "auctionId" TEXT NOT NULL,
    "userId" TEXT NOT NULL,
    "alias" TEXT NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`bidderstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "decidedById" TEXT,
    "decidedAt" TIMESTAMPTZ,
    "decisionReason" TEXT,
    "appealText" TEXT,
    "appealedAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "onsitemessagetranslation" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "onsiteMessageId" TEXT NOT NULL,
    "locale" TEXT NOT NULL,
    "headline" TEXT,
    "body" TEXT,
    "ctaLabel" TEXT
);

CREATE TABLE IF NOT EXISTS "lifecyclechecklist" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "userId" TEXT NOT NULL,
    "type" lifecyclechecklisttype NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`lifecyclecheckliststatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "createdById" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "completedAt" TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS "b2borderrequest" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "b2bAccountId" TEXT NOT NULL,
    "requestedByContactId" TEXT NOT NULL,
    "requestedByUserId" TEXT,
    "cartToken" TEXT NOT NULL,
    "locale" TEXT NOT NULL,
    "paymentDto" JSONB NOT NULL,
    "amountMinor" INTEGER NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`b2borderrequeststatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "orderId" TEXT UNIQUE,
    "rejectionNote" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "emailverificationtoken" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "userId" TEXT NOT NULL,
    "tokenHash" TEXT NOT NULL UNIQUE,
    "expiresAt" TIMESTAMPTZ NOT NULL,
    "usedAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "replicationcursor" (
    "peerNodeId" TEXT NOT NULL,
    "direction" replicationdirection NOT NULL,
    "lastSequence" TEXT NOT NULL,
    "lastSyncAt" TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS "page" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "slug" TEXT NOT NULL UNIQUE,
    "template"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`TEXT' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`contentstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "publishedAt" TIMESTAMPTZ,
    "noindex" BOOLEAN DEFAULT FALSE NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "taskstagedefinition" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "key" TEXT NOT NULL UNIQUE,
    "label" TEXT NOT NULL,
    "order" INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS "payrollbankaccount" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "userId" TEXT NOT NULL UNIQUE,
    "routingNumberOrSortCode" TEXT,
    "accountNumber" TEXT,
    "iban" TEXT,
    "bic" TEXT,
    "accountType" payrollbankaccounttype,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "interaction" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "contactId" TEXT,
    "leadId" TEXT,
    "kind" interactionkind NOT NULL,
    "body" TEXT NOT NULL,
    "actorId" TEXT,
    "occurredAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "customersupportticket" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "contactId" TEXT NOT NULL,
    "orderId" TEXT,
    "subject" TEXT NOT NULL,
    "category" TEXT NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`customerticketstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "priority"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`ticketpriority' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "assignedToUserId" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "resolvedAt" TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS "onsitemessage" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "kind" onsitemessagekind NOT NULL,
    "pagePattern" TEXT,
    "segmentId" TEXT,
    "trigger"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`onsitepopuptrigger' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "delaySeconds" INTEGER,
    "linkUrl" TEXT,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "startsAt" TIMESTAMPTZ,
    "endsAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "attendancedevice" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "code" TEXT NOT NULL UNIQUE,
    "label" TEXT NOT NULL,
    "orgUnitId" TEXT NOT NULL,
    "deviceKeyHash" TEXT UNIQUE,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`attendancedevicestatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "lastSeenAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "customercreditterm" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "customerId" TEXT NOT NULL UNIQUE,
    "creditLimitMinor" INTEGER NOT NULL,
    "paymentTermDays" INTEGER DEFAULT 30 NOT NULL,
    "isOnHold" BOOLEAN DEFAULT FALSE NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "emailaccount" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "label" TEXT NOT NULL,
    "purpose"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`emailaccountpurpose' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "isDefault" BOOLEAN DEFAULT FALSE NOT NULL,
    "fromName" TEXT NOT NULL,
    "fromEmail" TEXT NOT NULL,
    "replyTo" TEXT,
    "smtpHost" TEXT NOT NULL,
    "smtpPort" INTEGER DEFAULT 587 NOT NULL,
    "smtpSecure" BOOLEAN DEFAULT FALSE NOT NULL,
    "smtpUser" TEXT NOT NULL,
    "smtpPasswordEncrypted" TEXT NOT NULL,
    "smtpVerifiedAt" TIMESTAMPTZ,
    "imapHost" TEXT,
    "imapPort" INTEGER,
    "imapSecure" BOOLEAN DEFAULT TRUE NOT NULL,
    "imapUser" TEXT,
    "imapPasswordEncrypted" TEXT,
    "imapVerifiedAt" TIMESTAMPTZ,
    "imapLastPolledAt" TIMESTAMPTZ,
    "imapLastUid" INTEGER,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "instalmentplan" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "orderId" TEXT NOT NULL UNIQUE,
    "type" instalmentplantype NOT NULL,
    "totalMinor" INTEGER NOT NULL,
    "depositMinor" INTEGER DEFAULT 0 NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`instalmentplanstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "cancelledAt" TIMESTAMPTZ,
    "defaultedAt" TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS "adminscope" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "userId" TEXT NOT NULL,
    "level" adminscopelevel NOT NULL,
    "retailBrandId" TEXT,
    "branchNodeId" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "bannertranslation" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "bannerId" TEXT NOT NULL,
    "locale" TEXT NOT NULL,
    "headline" TEXT,
    "subtext" TEXT,
    "altText" TEXT
);

CREATE TABLE IF NOT EXISTS "returnreasoncode" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "key" TEXT NOT NULL UNIQUE,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "order" INTEGER DEFAULT 0 NOT NULL
);

CREATE TABLE IF NOT EXISTS "offer" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "title" TEXT NOT NULL,
    "description" TEXT NOT NULL,
    "discountLabel" TEXT,
    "validFrom" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "validUntil" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "auction" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "number" TEXT NOT NULL UNIQUE,
    "organizationId" TEXT,
    "outletLocationId" TEXT,
    "type"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`auctiontype' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "title" TEXT NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`auctionstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "startsAt" TIMESTAMPTZ NOT NULL,
    "endsAt" TIMESTAMPTZ NOT NULL,
    "currency" TEXT NOT NULL,
    "terms" TEXT,
    "bidRules" JSONB,
    "registration"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`auctionregistration' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "createdById" TEXT NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "shippingzone" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "key" TEXT NOT NULL UNIQUE,
    "order" INTEGER DEFAULT 0 NOT NULL,
    "cities" TEXT[] DEFAULT '{}' NOT NULL,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "isCatchAll" BOOLEAN DEFAULT FALSE NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "optiontemplatetranslation" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "templateId" TEXT NOT NULL,
    "locale" TEXT NOT NULL,
    "label" TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS "productimage" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "productId" TEXT NOT NULL,
    "variantId" TEXT,
    "url" TEXT NOT NULL,
    "alt" TEXT,
    "order" INTEGER DEFAULT 0 NOT NULL,
    "isPrimary" BOOLEAN DEFAULT FALSE NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "sourcedFromGlobalReferenceId" TEXT
);

CREATE TABLE IF NOT EXISTS "variantforecast" (
    "variantId" TEXT PRIMARY KEY,
    "forecastedDailyDemand" DOUBLE PRECISION DEFAULT 0 NOT NULL,
    "seasonalFactor" DOUBLE PRECISION,
    "projectedStockoutAt" TIMESTAMPTZ,
    "reasonEn" TEXT,
    "computedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "safetyStockUnits" INTEGER,
    "suggestedReorderPoint" INTEGER,
    "serviceLevelPercent" INTEGER
);

CREATE TABLE IF NOT EXISTS "instalmentpayment" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "planId" TEXT NOT NULL,
    "paymentId" TEXT NOT NULL UNIQUE,
    "receivedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "isDeposit" BOOLEAN DEFAULT FALSE NOT NULL
);

CREATE TABLE IF NOT EXISTS "redirect" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "fromPath" TEXT NOT NULL UNIQUE,
    "toPath" TEXT NOT NULL,
    "statusCode" INTEGER DEFAULT 301 NOT NULL,
    "isAutomatic" BOOLEAN DEFAULT FALSE NOT NULL,
    "hitCount" INTEGER DEFAULT 0 NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "tasktransition" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "taskId" TEXT NOT NULL,
    "fromState" taskstate,
    "toState" taskstate NOT NULL,
    "fromStageId" TEXT,
    "toStageId" TEXT,
    "byUserId" TEXT,
    "reason" TEXT,
    "occurredAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "meetinginvitation" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "meetingId" TEXT NOT NULL,
    "userId" TEXT NOT NULL,
    "required" BOOLEAN DEFAULT TRUE NOT NULL,
    "response"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`inviteresponse' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "respondedAt" TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS "partyroleassignment" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "partyId" TEXT NOT NULL,
    "role" partyrolename NOT NULL
);

CREATE TABLE IF NOT EXISTS "address" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "userId" TEXT,
    "label" TEXT,
    "fullName" TEXT NOT NULL,
    "phone" TEXT NOT NULL,
    "line1" TEXT NOT NULL,
    "line2" TEXT,
    "city" TEXT NOT NULL,
    "province" TEXT NOT NULL,
    "postalCode" TEXT,
    "country"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`TEXT' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "isDefault" BOOLEAN DEFAULT FALSE NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "productsizebias" (
    "productId" TEXT PRIMARY KEY,
    "tooSmallPct" DOUBLE PRECISION DEFAULT 0 NOT NULL,
    "trueToSizePct" DOUBLE PRECISION DEFAULT 0 NOT NULL,
    "tooLargePct" DOUBLE PRECISION DEFAULT 0 NOT NULL,
    "sampleSize" INTEGER DEFAULT 0 NOT NULL,
    "shift" INTEGER DEFAULT 0 NOT NULL,
    "computedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "dunningpolicy" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "stepsJson" JSONB NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "bundlecomponent" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "bundleId" TEXT NOT NULL,
    "componentVariantId" TEXT NOT NULL,
    "quantity" INTEGER DEFAULT 1 NOT NULL
);

CREATE TABLE IF NOT EXISTS "employeeprofile" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "userId" TEXT NOT NULL UNIQUE,
    "employeeCode" TEXT NOT NULL UNIQUE,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`employmentstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "joinedOn" TIMESTAMPTZ NOT NULL,
    "leftOn" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "goodsreceipt" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "number" TEXT NOT NULL UNIQUE,
    "purchaseOrderId" TEXT NOT NULL,
    "receivedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "receivedById" TEXT,
    "notes" TEXT,
    "freightCost" INTEGER DEFAULT 0 NOT NULL,
    "dutyCost" INTEGER DEFAULT 0 NOT NULL,
    "otherCost" INTEGER DEFAULT 0 NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "locationId" TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS "shippingmethodtranslation" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "methodId" TEXT NOT NULL,
    "locale" TEXT NOT NULL,
    "label" TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS "audience" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "orgUnitId" TEXT,
    "includeDescendants" BOOLEAN DEFAULT TRUE NOT NULL,
    "minRankLevel" INTEGER,
    "positionIds" TEXT[] NOT NULL,
    "createdByUserId" TEXT NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "productvariantoptionvalue" (
    "variantId" TEXT NOT NULL,
    "optionId" TEXT NOT NULL,
    "valueId" TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS "passwordresettoken" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "userId" TEXT NOT NULL,
    "tokenHash" TEXT NOT NULL UNIQUE,
    "expiresAt" TIMESTAMPTZ NOT NULL,
    "usedAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "manifest" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "number" TEXT NOT NULL UNIQUE,
    "courierKey" TEXT NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`manifeststatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "dispatchedAt" TIMESTAMPTZ,
    "createdById" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "devicetoken" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "userId" TEXT NOT NULL,
    "token" TEXT NOT NULL UNIQUE,
    "platform"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`deviceplatform' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "appVersion" TEXT,
    "locale" TEXT,
    "lastSeenAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "order" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "number" TEXT NOT NULL UNIQUE,
    "userId" TEXT,
    "contactId" TEXT,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`orderstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "currency"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`TEXT' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "channel"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`orderchannel' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "locationId" TEXT,
    "terminalId" TEXT,
    "shiftId" TEXT,
    "originNodeId" TEXT,
    "guestName" TEXT,
    "guestEmail" TEXT,
    "guestPhone" TEXT,
    "sourceCartId" TEXT,
    "shippingAddress" JSONB NOT NULL,
    "billingAddress" JSONB,
    "itemsSubtotal" INTEGER NOT NULL,
    "stitchingSubtotal" INTEGER DEFAULT 0 NOT NULL,
    "itemDiscountTotal" INTEGER DEFAULT 0 NOT NULL,
    "orderDiscount" INTEGER DEFAULT 0 NOT NULL,
    "shippingFee" INTEGER DEFAULT 0 NOT NULL,
    "codFee" INTEGER DEFAULT 0 NOT NULL,
    "taxAmount" INTEGER DEFAULT 0 NOT NULL,
    "grandTotal" INTEGER NOT NULL,
    "shippingZone" TEXT,
    "customerNote" TEXT,
    "staffNote" TEXT,
    "cancelReason" TEXT,
    "placedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "confirmedAt" TIMESTAMPTZ,
    "deliveredAt" TIMESTAMPTZ,
    "cancelledAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "reviewRequestSentAt" TIMESTAMPTZ,
    "codRiskScore" INTEGER,
    "codRiskAction" codriskaction,
    "codRiskComputedAt" TIMESTAMPTZ,
    "codRiskReasonEn" TEXT,
    "sourceType" TEXT,
    "sourceRef" TEXT
);

CREATE TABLE IF NOT EXISTS "shift" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "userId" TEXT NOT NULL,
    "orgUnitId" TEXT NOT NULL,
    "startsAt" TIMESTAMPTZ NOT NULL,
    "endsAt" TIMESTAMPTZ NOT NULL,
    "timezone" TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS "workordernumbercounter" (
    "year" INTEGER PRIMARY KEY,
    "counter" INTEGER DEFAULT 0 NOT NULL
);

CREATE TABLE IF NOT EXISTS "productvariant" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "productId" TEXT NOT NULL,
    "sku" TEXT NOT NULL UNIQUE,
    "size" TEXT,
    "color" TEXT,
    "colorHex" TEXT,
    "label" TEXT,
    "priceOverride" INTEGER,
    "stock" INTEGER DEFAULT 0 NOT NULL,
    "lowStockThreshold" INTEGER DEFAULT 3 NOT NULL,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "costPrice" INTEGER DEFAULT 0 NOT NULL,
    "reorderPoint" INTEGER,
    "reorderQuantity" INTEGER,
    "preferredSupplierId" TEXT,
    "lastReceivedAt" TIMESTAMPTZ,
    "barcode" TEXT UNIQUE,
    "skuIsManual" BOOLEAN DEFAULT FALSE NOT NULL,
    "producedInHouse" BOOLEAN DEFAULT FALSE NOT NULL,
    "lotTracked" BOOLEAN DEFAULT FALSE NOT NULL,
    "tracksSerials" BOOLEAN DEFAULT FALSE NOT NULL,
    "warrantyMonths" INTEGER,
    "taxClassId" TEXT,
    "order" INTEGER DEFAULT 0 NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "outboxevent" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "name" TEXT NOT NULL,
    "payload" JSONB NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`outboxstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "attempts" INTEGER DEFAULT 0 NOT NULL,
    "lastError" TEXT,
    "availableAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "processedAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "productaisuggestion" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "productId" TEXT NOT NULL,
    "imageId" TEXT,
    "kind" aisuggestionkind NOT NULL,
    "locale" TEXT NOT NULL,
    "suggestedText" TEXT NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`aisuggestionstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "reviewedAt" TIMESTAMPTZ,
    "reviewedById" TEXT
);

CREATE TABLE IF NOT EXISTS "organization" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "name" TEXT NOT NULL,
    "allowInterBrandTransfers" BOOLEAN DEFAULT FALSE NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "purchaseorderitem" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "purchaseOrderId" TEXT NOT NULL,
    "variantId" TEXT NOT NULL,
    "quantityOrdered" INTEGER NOT NULL,
    "quantityReceived" INTEGER DEFAULT 0 NOT NULL,
    "unitCost" INTEGER NOT NULL,
    "lineTotal" INTEGER NOT NULL,
    "note" TEXT
);

CREATE TABLE IF NOT EXISTS "whatsappmessage" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "conversationId" TEXT NOT NULL,
    "direction" whatsappmessagedirection NOT NULL,
    "body" TEXT NOT NULL,
    "aiGenerated" BOOLEAN DEFAULT FALSE NOT NULL,
    "externalMessageId" TEXT UNIQUE,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "shipmentitem" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "shipmentId" TEXT NOT NULL,
    "orderItemId" TEXT NOT NULL,
    "quantity" INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS "auditlog" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "actorId" TEXT,
    "action" TEXT NOT NULL,
    "entity" TEXT NOT NULL,
    "entityId" TEXT,
    "data" JSONB,
    "ip" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "identityverification" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "userId" TEXT NOT NULL UNIQUE,
    "nidSubjectId" TEXT,
    "nidCaseId" TEXT,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`identityverificationstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "method" TEXT,
    "failureReason" TEXT,
    "startedAt" TIMESTAMPTZ,
    "resolvedAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "cartstockhold" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "cartId" TEXT NOT NULL,
    "variantId" TEXT NOT NULL,
    "quantity" INTEGER NOT NULL,
    "tier"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`cartstockholdtier' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`cartstockholdstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "heartbeatAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "expiresAt" TIMESTAMPTZ NOT NULL,
    "renewCount" INTEGER DEFAULT 0 NOT NULL,
    "maxRenewals" INTEGER DEFAULT 5 NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "pricelistassignment" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "priceListId" TEXT NOT NULL,
    "customerId" TEXT,
    "tagId" TEXT
);

CREATE TABLE IF NOT EXISTS "poslinemodifier" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "lineRef" TEXT NOT NULL,
    "key" TEXT NOT NULL,
    "label" TEXT NOT NULL,
    "priceDeltaMinor" INTEGER DEFAULT 0 NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "inboundemail" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "emailAccountId" TEXT NOT NULL,
    "messageId" TEXT NOT NULL,
    "inReplyTo" TEXT,
    "fromAddress" TEXT NOT NULL,
    "fromName" TEXT,
    "subject" TEXT,
    "bodyText" TEXT,
    "receivedAt" TIMESTAMPTZ NOT NULL,
    "contactId" TEXT,
    "leadId" TEXT,
    "interactionId" TEXT UNIQUE,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "workcenter" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "name" TEXT NOT NULL,
    "orgUnitId" TEXT,
    "machineCount" INTEGER DEFAULT 1 NOT NULL,
    "shiftsPerDay" INTEGER DEFAULT 1 NOT NULL,
    "hoursPerShift" INTEGER DEFAULT 8 NOT NULL,
    "efficiencyFactor" DOUBLE PRECISION DEFAULT 1.0 NOT NULL,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "posparkedsale" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "shiftId" TEXT NOT NULL,
    "orderType"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`posordertype' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "label" TEXT NOT NULL,
    "basketSnapshot" JSONB NOT NULL,
    "parkedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "retrievedAt" TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS "contact" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "phoneE164" TEXT UNIQUE,
    "email" TEXT UNIQUE,
    "name" TEXT NOT NULL,
    "city" TEXT,
    "gender" TEXT,
    "dateOfBirth" TIMESTAMPTZ,
    "avatarUrl" TEXT,
    "source"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`TEXT' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`contactstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "ownerId" TEXT,
    "userId" TEXT UNIQUE,
    "notes" TEXT,
    "orderCount" INTEGER DEFAULT 0 NOT NULL,
    "lifetimeValue" INTEGER DEFAULT 0 NOT NULL,
    "avgOrderValue" INTEGER DEFAULT 0 NOT NULL,
    "lastOrderAt" TIMESTAMPTZ,
    "firstOrderAt" TIMESTAMPTZ,
    "rtoCount" INTEGER DEFAULT 0 NOT NULL,
    "marketingOptOut" BOOLEAN DEFAULT FALSE NOT NULL,
    "rfmRecencyScore" INTEGER,
    "rfmFrequencyScore" INTEGER,
    "rfmMonetaryScore" INTEGER,
    "rfmComputedAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "account" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "code" TEXT NOT NULL UNIQUE,
    "name" TEXT NOT NULL,
    "type" accounttype NOT NULL,
    "parentId" TEXT,
    "isIntercompany" BOOLEAN DEFAULT FALSE NOT NULL,
    "isBank" BOOLEAN DEFAULT FALSE NOT NULL,
    "isControl" BOOLEAN DEFAULT FALSE NOT NULL,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "leavetype" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "key" TEXT NOT NULL UNIQUE,
    "name" TEXT NOT NULL,
    "kind" leavekind NOT NULL,
    "accrualDaysPerRun" DOUBLE PRECISION DEFAULT 0 NOT NULL,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL
);

CREATE TABLE IF NOT EXISTS "requisitionline" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "requisitionId" TEXT NOT NULL,
    "variantId" TEXT NOT NULL,
    "quantity" INTEGER NOT NULL,
    "estimatedUnitCostMinor" INTEGER
);

CREATE TABLE IF NOT EXISTS "stocktransferline" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "transferId" TEXT NOT NULL,
    "variantId" TEXT NOT NULL,
    "quantityDispatched" INTEGER NOT NULL,
    "quantityReceived" INTEGER,
    "unitCost" INTEGER
);

CREATE TABLE IF NOT EXISTS "review" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "productId" TEXT NOT NULL,
    "userId" TEXT,
    "orderItemId" TEXT UNIQUE,
    "rating" INTEGER NOT NULL,
    "title" TEXT,
    "body" TEXT,
    "photoUrls" TEXT[] DEFAULT '{}' NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`reviewstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "fitFeedback" TEXT,
    "helpfulCount" INTEGER DEFAULT 0 NOT NULL,
    "staffReply" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "orgunitgrant" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "userId" TEXT NOT NULL,
    "orgUnitId" TEXT NOT NULL,
    "permissionKeys" TEXT[] NOT NULL,
    "mode"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`grantmode' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "expiresAt" TIMESTAMPTZ NOT NULL,
    "grantedByUserId" TEXT NOT NULL,
    "countersignedByUserId" TEXT,
    "reason" TEXT NOT NULL,
    "revokedAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "spotcheck" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "policyId" TEXT NOT NULL,
    "locationId" TEXT NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`spotcheckstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "scheduledFor" TIMESTAMPTZ NOT NULL,
    "startedAt" TIMESTAMPTZ,
    "completedAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "user" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "email" TEXT UNIQUE,
    "phone" TEXT UNIQUE,
    "passwordHash" TEXT,
    "firstName" TEXT,
    "lastName" TEXT,
    "gender" TEXT,
    "dateOfBirth" TIMESTAMPTZ,
    "avatarUrl" TEXT,
    "role"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`userrole' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "emailVerifiedAt" TIMESTAMPTZ,
    "marketingEmailOptOut" BOOLEAN DEFAULT FALSE NOT NULL,
    "lastLoginAt" TIMESTAMPTZ,
    "ssoExternalId" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "storeCreditBalance" INTEGER DEFAULT 0 NOT NULL,
    "loyaltyPoints" INTEGER DEFAULT 0 NOT NULL,
    "referralCode" TEXT UNIQUE
);

CREATE TABLE IF NOT EXISTS "embeddedwidget" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "key" TEXT NOT NULL UNIQUE,
    "label" TEXT NOT NULL,
    "targetUrl" TEXT NOT NULL,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "createdByUserId" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "party" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "name" TEXT NOT NULL,
    "contactName" TEXT,
    "phone" TEXT,
    "email" TEXT,
    "address" TEXT,
    "city" TEXT,
    "taxId" TEXT,
    "notes" TEXT,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "externalRef" TEXT,
    "externalSystem" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "documentsource" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "organizationId" TEXT,
    "storageKey" TEXT NOT NULL,
    "originalName" TEXT NOT NULL,
    "corpus" documentcorpus NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`ingeststatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "pageCount" INTEGER,
    "ingestedAt" TIMESTAMPTZ,
    "failureReason" TEXT,
    "uploadedById" TEXT,
    "claimedAt" TIMESTAMPTZ,
    "attempts" INTEGER DEFAULT 0 NOT NULL,
    "nextAttemptAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "b2baccount" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "customerId" TEXT NOT NULL UNIQUE,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "requiresPurchaseOrder" BOOLEAN DEFAULT FALSE NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "product" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "slug" TEXT NOT NULL UNIQUE,
    "kind" productkind NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`productstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "categoryId" TEXT,
    "basePrice" INTEGER NOT NULL,
    "compareAtPrice" INTEGER,
    "brandId" TEXT,
    "fabric" TEXT,
    "season" TEXT,
    "tags" TEXT[] DEFAULT '{}' NOT NULL,
    "code" TEXT,
    "noindex" BOOLEAN DEFAULT FALSE NOT NULL,
    "canonicalOverride" TEXT,
    "pieceCount" INTEGER,
    "fabricMeters" DOUBLE PRECISION,
    "stitchingAvailable" BOOLEAN DEFAULT FALSE NOT NULL,
    "isConsumable" BOOLEAN DEFAULT FALSE NOT NULL,
    "garmentTypeId" TEXT,
    "shippingClassId" TEXT,
    "featuredRank" INTEGER DEFAULT 0 NOT NULL,
    "publishedAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "stockcountline" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "stockCountId" TEXT NOT NULL,
    "variantId" TEXT NOT NULL,
    "expected" INTEGER,
    "counted" INTEGER,
    "note" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "lotstock" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "lotId" TEXT NOT NULL,
    "locationId" TEXT NOT NULL,
    "quantity" INTEGER DEFAULT 0 NOT NULL
);

CREATE TABLE IF NOT EXISTS "loyaltytransaction" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "userId" TEXT NOT NULL,
    "orderId" TEXT,
    "type" loyaltytransactiontype NOT NULL,
    "points" INTEGER NOT NULL,
    "note" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "campaignrecipient" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "campaignId" TEXT NOT NULL,
    "contactId" TEXT NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`campaignrecipientstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "sentAt" TIMESTAMPTZ,
    "openedAt" TIMESTAMPTZ,
    "clickedAt" TIMESTAMPTZ,
    "failureReason" TEXT
);

CREATE TABLE IF NOT EXISTS "journalentry" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "number" TEXT NOT NULL UNIQUE,
    "date" TIMESTAMPTZ NOT NULL,
    "periodId" TEXT NOT NULL,
    "source" TEXT NOT NULL,
    "sourceRef" TEXT,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`journalentrystatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "reversalOfId" TEXT UNIQUE,
    "memo" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "phoneoptout" (
    "phoneE164" TEXT PRIMARY KEY,
    "optedOutAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "bankfacilitycovenant" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "facilityId" TEXT NOT NULL,
    "covenantType" bankfacilitycovenanttype NOT NULL,
    "targetThreshold" DOUBLE PRECISION NOT NULL,
    "description" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "compensationband" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "rankId" TEXT NOT NULL UNIQUE,
    "minSalaryMinor" INTEGER NOT NULL,
    "midpointSalaryMinor" INTEGER NOT NULL,
    "maxSalaryMinor" INTEGER NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "stitchingordernumbercounter" (
    "year" INTEGER PRIMARY KEY,
    "counter" INTEGER DEFAULT 0 NOT NULL
);

CREATE TABLE IF NOT EXISTS "blogposttranslation" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "blogPostId" TEXT NOT NULL,
    "locale" TEXT NOT NULL,
    "title" TEXT NOT NULL,
    "body" JSONB NOT NULL,
    "excerpt" TEXT,
    "metaTitle" TEXT,
    "metaDescription" TEXT,
    "ogImageUrl" TEXT
);

CREATE TABLE IF NOT EXISTS "attendancegeofencezone" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "orgUnitId" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "centerLat" DOUBLE PRECISION NOT NULL,
    "centerLng" DOUBLE PRECISION NOT NULL,
    "radiusMeters" INTEGER NOT NULL,
    "polygonVertices" JSONB,
    "enforcePolygon" BOOLEAN DEFAULT FALSE NOT NULL,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "supplierleadtimeobservation" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "supplierId" TEXT NOT NULL,
    "purchaseOrderId" TEXT NOT NULL,
    "observedDays" INTEGER NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "taxclass" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "key" TEXT NOT NULL UNIQUE,
    "name" TEXT NOT NULL,
    "isDefault" BOOLEAN DEFAULT FALSE NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "shippingclass" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "key" TEXT NOT NULL UNIQUE,
    "handlingFee" INTEGER DEFAULT 0 NOT NULL,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "lead" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "contactId" TEXT,
    "name" TEXT NOT NULL,
    "phone" TEXT,
    "email" TEXT,
    "message" TEXT,
    "source"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`TEXT' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`leadstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "ownerId" TEXT,
    "quotedValue" INTEGER,
    "lostReason" TEXT,
    "convertedOrderId" TEXT,
    "convertedAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "blogpost" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "slug" TEXT NOT NULL UNIQUE,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`contentstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "publishedAt" TIMESTAMPTZ,
    "authorId" TEXT,
    "category" TEXT,
    "tags" TEXT[] DEFAULT '{}' NOT NULL,
    "heroImageUrl" TEXT,
    "noindex" BOOLEAN DEFAULT FALSE NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "segment" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "name" TEXT NOT NULL UNIQUE,
    "description" TEXT,
    "filter" JSONB NOT NULL,
    "createdById" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "shipmentevent" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "shipmentId" TEXT NOT NULL,
    "status" shipmentstatus NOT NULL,
    "rawStatus" TEXT,
    "meta" JSONB,
    "occurredAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "loyaltytier" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "name" TEXT NOT NULL,
    "minSpend" INTEGER NOT NULL,
    "benefitDescriptionEn" TEXT NOT NULL,
    "benefitDescriptionUr" TEXT,
    "sortOrder" INTEGER DEFAULT 0 NOT NULL,
    "earnMultiplierBps" INTEGER DEFAULT 10000 NOT NULL
);

CREATE TABLE IF NOT EXISTS "billofmaterial" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "variantId" TEXT NOT NULL UNIQUE,
    "name" TEXT,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "entityattribute" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "entityType" TEXT NOT NULL,
    "entityId" TEXT NOT NULL,
    "attributeKey" TEXT NOT NULL,
    "attributeValue" TEXT NOT NULL,
    "valueType"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`TEXT' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "searchsynonym" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "locale" TEXT NOT NULL,
    "term" TEXT NOT NULL,
    "synonym" TEXT NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "escalationfire" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "taskId" TEXT NOT NULL,
    "ruleId" TEXT NOT NULL,
    "fireNumber" INTEGER NOT NULL,
    "climbedToPositionId" TEXT NOT NULL,
    "firedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "refreshtoken" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "userId" TEXT NOT NULL,
    "tokenHash" TEXT NOT NULL UNIQUE,
    "expiresAt" TIMESTAMPTZ NOT NULL,
    "revokedAt" TIMESTAMPTZ,
    "userAgent" TEXT,
    "ip" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "familyId" TEXT NOT NULL,
    "rotatedAt" TIMESTAMPTZ,
    "replacedById" TEXT UNIQUE
);

CREATE TABLE IF NOT EXISTS "auctionlotimage" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "lotId" TEXT NOT NULL,
    "url" TEXT NOT NULL,
    "storageKey" TEXT NOT NULL,
    "order" INTEGER NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "followuptask" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "contactId" TEXT,
    "leadId" TEXT,
    "assigneeId" TEXT,
    "note" TEXT NOT NULL,
    "dueAt" TIMESTAMPTZ NOT NULL,
    "isDone" BOOLEAN DEFAULT FALSE NOT NULL,
    "doneAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "bank" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "name" TEXT NOT NULL UNIQUE,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "posterminal" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "code" TEXT NOT NULL UNIQUE,
    "label" TEXT NOT NULL,
    "locationId" TEXT NOT NULL,
    "deviceKeyHash" TEXT UNIQUE,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`posterminalstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "lastSeenAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "invoice" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "number" TEXT NOT NULL UNIQUE,
    "tradingPartnerId" TEXT NOT NULL,
    "orderId" TEXT,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`invoicestatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "currency" TEXT NOT NULL,
    "issueDate" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "dueDate" TIMESTAMPTZ,
    "subtotalMinor" INTEGER NOT NULL,
    "taxMinor" INTEGER DEFAULT 0 NOT NULL,
    "totalMinor" INTEGER NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "taxrate" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "taxClassId" TEXT NOT NULL,
    "rateBps" INTEGER NOT NULL,
    "effectiveAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "endedAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "orderdiscount" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "orderId" TEXT NOT NULL,
    "discountId" TEXT,
    "promoCodeId" TEXT,
    "name" TEXT NOT NULL,
    "type" TEXT NOT NULL,
    "amount" INTEGER NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "supplierproduct" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "supplierId" TEXT NOT NULL,
    "variantId" TEXT NOT NULL,
    "supplierSku" TEXT,
    "lastCost" INTEGER DEFAULT 0 NOT NULL,
    "moq" INTEGER DEFAULT 1 NOT NULL,
    "leadTimeDays" INTEGER,
    "isPreferred" BOOLEAN DEFAULT FALSE NOT NULL
);

CREATE TABLE IF NOT EXISTS "escalationrule" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "sourceType" TEXT NOT NULL UNIQUE,
    "afterMinutes" INTEGER NOT NULL,
    "stepsPerFire" INTEGER DEFAULT 1 NOT NULL,
    "ceilingRankLevel" INTEGER,
    "maxFires" INTEGER DEFAULT 3 NOT NULL
);

CREATE TABLE IF NOT EXISTS "idempotencykey" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "scope" TEXT NOT NULL,
    "key" TEXT NOT NULL,
    "requestHash" TEXT NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`idempotencykeystatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "responseStatus" INTEGER,
    "responseBody" JSONB,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "completedAt" TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS "productoptionvalue" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "optionId" TEXT NOT NULL,
    "key" TEXT NOT NULL,
    "swatchHex" TEXT,
    "swatchUrl" TEXT,
    "order" INTEGER DEFAULT 0 NOT NULL
);

CREATE TABLE IF NOT EXISTS "settlementbatch" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "settlementProviderId" TEXT NOT NULL,
    "externalBatchRef" TEXT NOT NULL,
    "expectedAmountMinor" INTEGER NOT NULL,
    "settledAmountMinor" INTEGER,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`settlementbatchstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "settledAt" TIMESTAMPTZ,
    "writtenOffAt" TIMESTAMPTZ,
    "writeoffRequestedById" TEXT,
    "writeoffApprovedById" TEXT,
    "writeoffNote" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "rolegrant" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "roleId" TEXT NOT NULL,
    "permissionId" TEXT NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "productoptiontranslation" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "optionId" TEXT NOT NULL,
    "locale" TEXT NOT NULL,
    "label" TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS "instalmentscheduleline" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "planId" TEXT NOT NULL,
    "dueDate" TIMESTAMPTZ NOT NULL,
    "amountMinor" INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS "journalline" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "entryId" TEXT NOT NULL,
    "accountId" TEXT NOT NULL,
    "debitMinor" INTEGER DEFAULT 0 NOT NULL,
    "creditMinor" INTEGER DEFAULT 0 NOT NULL,
    "currency" TEXT NOT NULL,
    "fxRateScaled" INTEGER DEFAULT 1000000 NOT NULL,
    "baseAmountMinor" INTEGER NOT NULL,
    "organizationId" TEXT,
    "retailBrandId" TEXT,
    "branchNodeId" TEXT,
    "costCentreId" TEXT,
    "contactId" TEXT,
    "supplierId" TEXT,
    "counterpartyBrandId" TEXT
);

CREATE TABLE IF NOT EXISTS "bankfacilitymarkupaccrual" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "facilityId" TEXT NOT NULL,
    "periodStart" TIMESTAMPTZ NOT NULL,
    "periodEnd" TIMESTAMPTZ NOT NULL,
    "amountMinor" INTEGER NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "bid" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "lotId" TEXT NOT NULL,
    "bidderId" TEXT NOT NULL,
    "amountMinor" TEXT NOT NULL,
    "kind"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`bidkind' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "seq" INTEGER NOT NULL,
    "acceptedAt" TIMESTAMPTZ NOT NULL,
    "idempotencyKey" TEXT NOT NULL,
    "prevHash" TEXT,
    "hash" TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS "analyticsevent" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "type" TEXT NOT NULL,
    "path" TEXT,
    "source"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`TEXT' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "sessionId" TEXT NOT NULL,
    "meta" JSONB,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "measurementfield" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "garmentTypeId" TEXT NOT NULL,
    "key" TEXT NOT NULL,
    "labelEn" TEXT NOT NULL,
    "labelUr" TEXT NOT NULL,
    "unit"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`measurementunit' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "min" DOUBLE PRECISION NOT NULL,
    "max" DOUBLE PRECISION NOT NULL,
    "required" BOOLEAN DEFAULT TRUE NOT NULL,
    "order" INTEGER DEFAULT 0 NOT NULL,
    "helpEn" TEXT,
    "helpUr" TEXT,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL
);

CREATE TABLE IF NOT EXISTS "location" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "name" TEXT NOT NULL,
    "code" TEXT NOT NULL UNIQUE,
    "type"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`locationtype' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "retailBrandId" TEXT,
    "originNodeId" TEXT,
    "isDefault" BOOLEAN DEFAULT FALSE NOT NULL,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "address" JSONB,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "optiontemplate" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "key" TEXT NOT NULL UNIQUE,
    "display"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`optiondisplay' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "order" INTEGER DEFAULT 0 NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "pricelistentry" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "priceListId" TEXT NOT NULL,
    "variantId" TEXT NOT NULL,
    "minQuantity" INTEGER DEFAULT 1 NOT NULL,
    "unitPriceMinor" INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS "bomrecipe" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "variantId" TEXT NOT NULL UNIQUE,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`bomstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "version" INTEGER DEFAULT 1 NOT NULL,
    "outputQuantity" INTEGER DEFAULT 1 NOT NULL,
    "wastageBps" INTEGER DEFAULT 0 NOT NULL,
    "labourCost" INTEGER DEFAULT 0 NOT NULL,
    "overheadCost" INTEGER DEFAULT 0 NOT NULL,
    "rolledUpUnitCost" INTEGER DEFAULT 0 NOT NULL,
    "costComplete" BOOLEAN DEFAULT FALSE NOT NULL,
    "costComputedAt" TIMESTAMPTZ,
    "notes" TEXT,
    "createdById" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "auctionresult" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "lotId" TEXT NOT NULL UNIQUE,
    "topBidderId" TEXT,
    "topBidMinor" TEXT,
    "reserveMet" BOOLEAN NOT NULL,
    "bidCount" INTEGER NOT NULL,
    "closedAt" TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS "ordereconomics" (
    "orderId" TEXT PRIMARY KEY,
    "revenue" INTEGER NOT NULL,
    "shippingIncome" INTEGER NOT NULL,
    "codFeeIncome" INTEGER NOT NULL,
    "cogs" INTEGER NOT NULL,
    "shippingCost" INTEGER NOT NULL,
    "shippingCostEstimated" BOOLEAN DEFAULT FALSE NOT NULL,
    "gatewayFee" INTEGER NOT NULL,
    "packagingCost" INTEGER NOT NULL,
    "contribution" INTEGER NOT NULL,
    "computedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "auctionlot" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "auctionId" TEXT NOT NULL,
    "lotNo" INTEGER NOT NULL,
    "title" TEXT NOT NULL,
    "description" TEXT,
    "sourceType" TEXT,
    "sourceId" TEXT,
    "quantity" INTEGER DEFAULT 1 NOT NULL,
    "startPriceMinor" TEXT NOT NULL,
    "reservePriceMinor" TEXT,
    "currentBidMinor" TEXT,
    "leaderBidderId" TEXT,
    "bidSeq" INTEGER DEFAULT 0 NOT NULL,
    "endsAt" TIMESTAMPTZ,
    "extendedMs" INTEGER DEFAULT 0 NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`auctionlotstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "permission" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "key" TEXT NOT NULL UNIQUE,
    "label" TEXT NOT NULL,
    "category" TEXT NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "discount" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "name" TEXT NOT NULL,
    "type" discounttype NOT NULL,
    "scope"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`discountscope' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "value" INTEGER NOT NULL,
    "maxDiscount" INTEGER,
    "minSubtotal" INTEGER,
    "minQuantity" INTEGER,
    "productIds" TEXT[] DEFAULT '{}' NOT NULL,
    "categoryIds" TEXT[] DEFAULT '{}' NOT NULL,
    "customerSegmentId" TEXT,
    "firstOrderOnly" BOOLEAN DEFAULT FALSE NOT NULL,
    "buyQuantity" INTEGER,
    "getQuantity" INTEGER,
    "getDiscountBps" INTEGER,
    "startsAt" TIMESTAMPTZ,
    "endsAt" TIMESTAMPTZ,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "isStackable" BOOLEAN DEFAULT FALSE NOT NULL,
    "priority" INTEGER DEFAULT 0 NOT NULL,
    "usageLimit" INTEGER,
    "usageLimitPerUser" INTEGER,
    "usageCount" INTEGER DEFAULT 0 NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "collectionproduct" (
    "collectionId" TEXT NOT NULL,
    "productId" TEXT NOT NULL,
    "order" INTEGER DEFAULT 0 NOT NULL
);

CREATE TABLE IF NOT EXISTS "replicationquarantine" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "originNodeId" TEXT NOT NULL,
    "sequence" TEXT NOT NULL,
    "entity" TEXT NOT NULL,
    "entityId" TEXT NOT NULL,
    "op" replicationop NOT NULL,
    "payload" JSONB NOT NULL,
    "schemaVersion" INTEGER NOT NULL,
    "reason" TEXT NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`replicationquarantinestatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "resolvedAt" TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS "stockadjustment" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "variantId" TEXT NOT NULL,
    "delta" INTEGER NOT NULL,
    "resultingStock" INTEGER NOT NULL,
    "reason" TEXT NOT NULL,
    "note" TEXT,
    "actorId" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "whatsappconversation" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "contactId" TEXT NOT NULL,
    "phoneE164" TEXT NOT NULL,
    "startedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "lastMessageAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "handedOffToHuman" BOOLEAN DEFAULT FALSE NOT NULL,
    "handoffReason" TEXT,
    "handedOffAt" TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS "lotallocation" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "lotId" TEXT NOT NULL,
    "stockMovementId" TEXT NOT NULL,
    "quantity" INTEGER NOT NULL,
    "allocatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "stockmovement" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "variantId" TEXT NOT NULL,
    "delta" INTEGER NOT NULL,
    "resultingStock" INTEGER NOT NULL,
    "reason" stockmovementreason NOT NULL,
    "locationId" TEXT,
    "refType" TEXT,
    "refId" TEXT,
    "unitCost" INTEGER,
    "note" TEXT,
    "actorId" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "returnrequestitem" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "returnId" TEXT NOT NULL,
    "orderItemId" TEXT NOT NULL,
    "reasonId" TEXT NOT NULL,
    "quantity" INTEGER NOT NULL,
    "restockable" BOOLEAN,
    "restocked" BOOLEAN DEFAULT FALSE NOT NULL
);

CREATE TABLE IF NOT EXISTS "quoteline" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "quoteId" TEXT NOT NULL,
    "sku" TEXT NOT NULL,
    "nameEn" TEXT NOT NULL,
    "quantity" INTEGER NOT NULL,
    "unitPriceMinor" INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS "stitchingjobevent" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "jobId" TEXT NOT NULL,
    "fromStage" stitchingstage,
    "toStage" stitchingstage NOT NULL,
    "note" TEXT,
    "photoUrl" TEXT,
    "actorId" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "apikey" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "userId" TEXT NOT NULL,
    "surface" apikeysurface NOT NULL,
    "name" TEXT NOT NULL,
    "keyHash" TEXT NOT NULL UNIQUE,
    "lastFour" TEXT NOT NULL,
    "permissionKeys" TEXT[] DEFAULT '{}' NOT NULL,
    "customerScopes" TEXT[] DEFAULT '{}' NOT NULL,
    "expiresAt" TIMESTAMPTZ NOT NULL,
    "lastUsedAt" TIMESTAMPTZ,
    "rotatedAt" TIMESTAMPTZ,
    "revokedAt" TIMESTAMPTZ,
    "revokedReason" apikeyrevocationreason,
    "revokedById" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "payment" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "orderId" TEXT NOT NULL,
    "provider" TEXT NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`paymentstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "amount" INTEGER NOT NULL,
    "currency"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`TEXT' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "reference" TEXT UNIQUE,
    "providerRef" TEXT,
    "meta" JSONB,
    "receiptUrl" TEXT,
    "failureReason" TEXT,
    "approvedById" TEXT,
    "paidAt" TIMESTAMPTZ,
    "refundedAt" TIMESTAMPTZ,
    "refundAmount" INTEGER,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "purchaseorder" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "number" TEXT NOT NULL UNIQUE,
    "supplierId" TEXT NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`purchaseorderstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "currency"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`TEXT' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "subtotal" INTEGER DEFAULT 0 NOT NULL,
    "freightCost" INTEGER DEFAULT 0 NOT NULL,
    "dutyCost" INTEGER DEFAULT 0 NOT NULL,
    "otherCost" INTEGER DEFAULT 0 NOT NULL,
    "total" INTEGER DEFAULT 0 NOT NULL,
    "scopeLevel"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`scopelevel' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "originNodeId" TEXT,
    "expectedAt" TIMESTAMPTZ,
    "sentAt" TIMESTAMPTZ,
    "receivedAt" TIMESTAMPTZ,
    "cancelledAt" TIMESTAMPTZ,
    "notes" TEXT,
    "createdById" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "sourceFactInboxId" TEXT UNIQUE
);

CREATE TABLE IF NOT EXISTS "invoiceline" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "invoiceId" TEXT NOT NULL,
    "sku" TEXT NOT NULL,
    "nameEn" TEXT NOT NULL,
    "quantity" INTEGER NOT NULL,
    "unitPriceMinor" INTEGER NOT NULL,
    "lineTotalMinor" INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS "shippingmethod" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "key" TEXT NOT NULL UNIQUE,
    "courierKey" TEXT,
    "etaMinDays" INTEGER DEFAULT 2 NOT NULL,
    "etaMaxDays" INTEGER DEFAULT 5 NOT NULL,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "order" INTEGER DEFAULT 0 NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "bankstatementline" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "bankId" TEXT NOT NULL,
    "date" TIMESTAMPTZ NOT NULL,
    "description" TEXT NOT NULL,
    "reference" TEXT,
    "amountMinor" INTEGER NOT NULL,
    "type" bankstatementlinetype NOT NULL,
    "matchedTransactionId" TEXT,
    "reconciled" BOOLEAN DEFAULT FALSE NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "rolepermission" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "role" userrole NOT NULL,
    "permissionId" TEXT NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "settlementbatchline" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "batchId" TEXT NOT NULL,
    "paymentId" TEXT NOT NULL,
    "amountMinor" INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS "attendanceevent" (
    "id" UUID DEFAULT gen_random_uuid() NOT NULL,
    "userId" TEXT NOT NULL,
    "kind" attendancekind NOT NULL,
    "occurredAt" TIMESTAMPTZ NOT NULL,
    "timezone" TEXT NOT NULL,
    "orgUnitId" TEXT NOT NULL,
    "source" attendancesource NOT NULL,
    "deviceId" TEXT,
    "correctsId" TEXT,
    "externalRef" TEXT,
    "latitude" DOUBLE PRECISION,
    "longitude" DOUBLE PRECISION,
    "gpsAccuracyMeters" DOUBLE PRECISION,
    "isMockLocation" BOOLEAN DEFAULT FALSE NOT NULL,
    "breakType" attendancebreaktype,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "auctionsettlementpayment" (
    "paymentId" TEXT PRIMARY KEY,
    "settlementId" TEXT NOT NULL,
    "amountMinor" TEXT NOT NULL,
    "receivedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "category" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "slug" TEXT NOT NULL UNIQUE,
    "code" TEXT,
    "parentId" TEXT,
    "imageUrl" TEXT,
    "order" INTEGER DEFAULT 0 NOT NULL,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "retailbrand" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "organizationId" TEXT,
    "parentId" TEXT,
    "name" TEXT NOT NULL,
    "slug" TEXT NOT NULL UNIQUE,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "attendanceexception" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "eventId" TEXT NOT NULL,
    "eventOccurredAt" TIMESTAMPTZ NOT NULL,
    "userId" TEXT NOT NULL,
    "orgUnitId" TEXT NOT NULL,
    "reason" attendanceexceptionreason NOT NULL,
    "resolvedAt" TIMESTAMPTZ,
    "resolvedByUserId" TEXT,
    "note" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "categorytranslation" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "categoryId" TEXT NOT NULL,
    "locale" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "description" TEXT
);

CREATE TABLE IF NOT EXISTS "brand" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "slug" TEXT NOT NULL UNIQUE,
    "logoUrl" TEXT,
    "order" INTEGER DEFAULT 0 NOT NULL,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "stitchingjob" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "number" TEXT NOT NULL UNIQUE,
    "orderItemId" TEXT UNIQUE,
    "stitchingOrderId" TEXT,
    "garmentTypeId" TEXT NOT NULL,
    "measurementSnapshot" JSONB NOT NULL,
    "optionsSnapshot" JSONB,
    "instructions" TEXT,
    "stage"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`stitchingstage' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "assignedTailorId" TEXT,
    "priority" INTEGER DEFAULT 0 NOT NULL,
    "dueAt" TIMESTAMPTZ,
    "startedAt" TIMESTAMPTZ,
    "completedAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "bidderstrike" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "userId" TEXT NOT NULL,
    "bidderId" TEXT NOT NULL,
    "lotId" TEXT,
    "reason" TEXT NOT NULL,
    "recordedById" TEXT,
    "settlementId" TEXT UNIQUE,
    "recordedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "voidedAt" TIMESTAMPTZ,
    "voidedById" TEXT,
    "voidReason" TEXT
);

CREATE TABLE IF NOT EXISTS "pagetranslation" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "pageId" TEXT NOT NULL,
    "locale" TEXT NOT NULL,
    "title" TEXT NOT NULL,
    "body" JSONB NOT NULL,
    "metaTitle" TEXT,
    "metaDescription" TEXT,
    "ogImageUrl" TEXT
);

CREATE TABLE IF NOT EXISTS "stitchingoption" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "groupId" TEXT NOT NULL,
    "key" TEXT NOT NULL,
    "labelEn" TEXT NOT NULL,
    "labelUr" TEXT NOT NULL,
    "priceDelta" INTEGER DEFAULT 0 NOT NULL,
    "imageUrl" TEXT,
    "order" INTEGER DEFAULT 0 NOT NULL,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL
);

CREATE TABLE IF NOT EXISTS "garmenttype" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "key" TEXT NOT NULL UNIQUE,
    "labelEn" TEXT NOT NULL,
    "labelUr" TEXT NOT NULL,
    "order" INTEGER DEFAULT 0 NOT NULL,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "stitchingPrice" INTEGER DEFAULT 0 NOT NULL,
    "stitchingEnabled" BOOLEAN DEFAULT TRUE NOT NULL,
    "baseFabricGaz" DOUBLE PRECISION,
    "aliasesEn" TEXT[] DEFAULT '{}' NOT NULL,
    "aliasesUr" TEXT[] DEFAULT '{}' NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "tailorprofile" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "userId" TEXT NOT NULL UNIQUE,
    "displayName" TEXT NOT NULL,
    "phone" TEXT,
    "specialties" TEXT[] DEFAULT '{}' NOT NULL,
    "capacityPerWeek" INTEGER DEFAULT 20 NOT NULL,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "referral" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "referrerId" TEXT NOT NULL,
    "refereeId" TEXT NOT NULL,
    "code" TEXT NOT NULL,
    "rewardedAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "replicationsequencecounter" (
    "nodeId" TEXT PRIMARY KEY,
    "value" TEXT DEFAULT 0 NOT NULL
);

CREATE TABLE IF NOT EXISTS "posreceipt" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "orderId" TEXT NOT NULL UNIQUE,
    "printCount" INTEGER DEFAULT 0 NOT NULL,
    "printedAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "lifecyclechecklisttemplatetask" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "type" lifecyclechecklisttype NOT NULL,
    "taskKey" TEXT NOT NULL,
    "title" TEXT NOT NULL,
    "description" TEXT NOT NULL,
    "assignedRole" clearancerole NOT NULL,
    "requiredForCompletion" BOOLEAN DEFAULT TRUE NOT NULL,
    "prerequisiteTaskKeys" JSONB,
    "sortOrder" INTEGER DEFAULT 0 NOT NULL
);

CREATE TABLE IF NOT EXISTS "goodsreceiptitem" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "goodsReceiptId" TEXT NOT NULL,
    "purchaseOrderItemId" TEXT NOT NULL,
    "variantId" TEXT NOT NULL,
    "quantity" INTEGER NOT NULL,
    "unitCost" INTEGER NOT NULL,
    "landedUnitCost" INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS "setting" (
    "key" TEXT PRIMARY KEY,
    "value" JSONB NOT NULL,
    "description" TEXT,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "purchaserequisition" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "requestedById" TEXT NOT NULL,
    "locationId" TEXT NOT NULL,
    "justification" TEXT NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`requisitionapprovalstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "convertedPurchaseOrderId" TEXT UNIQUE,
    "rejectionNote" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "documentchunk" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "sourceId" TEXT NOT NULL,
    "organizationId" TEXT,
    "corpus" documentcorpus NOT NULL,
    "pageFrom" INTEGER NOT NULL,
    "pageTo" INTEGER NOT NULL,
    "text" TEXT NOT NULL,
    "embedding" TEXT NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "collection" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "slug" TEXT NOT NULL UNIQUE,
    "rule" JSONB,
    "sortStrategy"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`collectionsortstrategy' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "imageUrl" TEXT,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "shippingzonetranslation" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "zoneId" TEXT NOT NULL,
    "locale" TEXT NOT NULL,
    "label" TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS "optiontemplatevaluetranslation" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "valueId" TEXT NOT NULL,
    "locale" TEXT NOT NULL,
    "label" TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS "merchantpayable" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "organizationId" TEXT NOT NULL,
    "orderId" TEXT NOT NULL UNIQUE,
    "grossMinor" INTEGER NOT NULL,
    "platformFeeMinor" INTEGER NOT NULL,
    "netPayableMinor" INTEGER NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`merchantpayablestatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "scheduledPayoutAt" TIMESTAMPTZ,
    "paidAt" TIMESTAMPTZ,
    "disputedByUserId" TEXT,
    "disputeReason" TEXT,
    "resolvedByUserId" TEXT,
    "resolvedAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "settlementprovider" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "key" TEXT NOT NULL UNIQUE,
    "label" TEXT NOT NULL,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL
);

CREATE TABLE IF NOT EXISTS "citycodriskstat" (
    "city" TEXT PRIMARY KEY,
    "totalShipments" INTEGER NOT NULL,
    "rtoShipments" INTEGER NOT NULL,
    "avgOrderValue" INTEGER NOT NULL,
    "computedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "assetassignment" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "assetId" TEXT NOT NULL,
    "userId" TEXT NOT NULL,
    "issuedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "issuedById" TEXT NOT NULL,
    "returnedAt" TIMESTAMPTZ,
    "returnedById" TEXT,
    "condition" TEXT
);

CREATE TABLE IF NOT EXISTS "variantlocationstock" (
    "variantId" TEXT NOT NULL,
    "locationId" TEXT NOT NULL,
    "quantity" INTEGER DEFAULT 0 NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "shippingclasstranslation" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "classId" TEXT NOT NULL,
    "locale" TEXT NOT NULL,
    "label" TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS "interbranchinvoice" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "number" TEXT NOT NULL UNIQUE,
    "transferId" TEXT NOT NULL UNIQUE,
    "sourceRetailBrandId" TEXT NOT NULL,
    "destinationRetailBrandId" TEXT NOT NULL,
    "amountMinor" INTEGER NOT NULL,
    "issuedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "variantsalesstat" (
    "variantId" TEXT PRIMARY KEY,
    "units30" INTEGER DEFAULT 0 NOT NULL,
    "units90" INTEGER DEFAULT 0 NOT NULL,
    "lastSoldAt" TIMESTAMPTZ,
    "daysOfCover" DOUBLE PRECISION,
    "ageDays" INTEGER,
    "isSlowMover" BOOLEAN DEFAULT FALSE NOT NULL,
    "isDeadStock" BOOLEAN DEFAULT FALSE NOT NULL,
    "lastLowStockAlertAt" TIMESTAMPTZ,
    "computedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "cashmovement" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "shiftId" TEXT NOT NULL,
    "type" cashmovementtype NOT NULL,
    "amountMinor" INTEGER NOT NULL,
    "reason" TEXT,
    "actorId" TEXT NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "orgunittype" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "key" TEXT NOT NULL UNIQUE,
    "label" TEXT NOT NULL,
    "rank" INTEGER DEFAULT 0 NOT NULL,
    "isOffice" BOOLEAN DEFAULT FALSE NOT NULL
);

CREATE TABLE IF NOT EXISTS "tag" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "name" TEXT NOT NULL UNIQUE,
    "color" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "supplier" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "name" TEXT NOT NULL,
    "contactName" TEXT,
    "phone" TEXT,
    "email" TEXT,
    "address" TEXT,
    "city" TEXT,
    "country" TEXT,
    "currency"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`TEXT' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "taxId" TEXT,
    "incoterms" TEXT,
    "isInternational" BOOLEAN DEFAULT FALSE NOT NULL,
    "paymentTerms" TEXT,
    "leadTimeDays" INTEGER DEFAULT 7 NOT NULL,
    "notes" TEXT,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "partyId" TEXT UNIQUE,
    "tradingPartnerId" TEXT,
    "customFields" JSONB DEFAULT '{}' NOT NULL
);

CREATE TABLE IF NOT EXISTS "leaverequest" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "userId" TEXT NOT NULL,
    "leaveTypeId" TEXT NOT NULL,
    "startDate" TIMESTAMPTZ NOT NULL,
    "endDate" TIMESTAMPTZ NOT NULL,
    "days" DOUBLE PRECISION NOT NULL,
    "reason" TEXT,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`leaverequeststatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "decidedById" TEXT,
    "decidedAt" TIMESTAMPTZ,
    "decisionNote" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "assetdepreciationposting" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "scheduleId" TEXT NOT NULL,
    "periodStart" TIMESTAMPTZ NOT NULL,
    "periodEnd" TIMESTAMPTZ NOT NULL,
    "amountMinor" INTEGER NOT NULL,
    "postedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "collectiontranslation" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "collectionId" TEXT NOT NULL,
    "locale" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "description" TEXT,
    "metaTitle" TEXT,
    "metaDescription" TEXT
);

CREATE TABLE IF NOT EXISTS "supportticket" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "raisedByUserId" TEXT NOT NULL,
    "subject" TEXT NOT NULL,
    "category" TEXT NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`ticketstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "priority"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`ticketpriority' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "assignedToUserId" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "resolvedAt" TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS "customfielddefinition" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "entityType" TEXT NOT NULL,
    "key" TEXT NOT NULL,
    "label" TEXT NOT NULL,
    "valueType" customfieldvaluetype NOT NULL,
    "isRequired" BOOLEAN DEFAULT FALSE NOT NULL,
    "pattern" TEXT,
    "minValue" DOUBLE PRECISION,
    "maxValue" DOUBLE PRECISION,
    "options" JSONB,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "billofmaterialline" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "billOfMaterialId" TEXT NOT NULL,
    "componentVariantId" TEXT NOT NULL,
    "quantityPerUnit" DOUBLE PRECISION NOT NULL
);

CREATE TABLE IF NOT EXISTS "meeting" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "title" TEXT NOT NULL,
    "detail" TEXT,
    "startsAt" TIMESTAMPTZ NOT NULL,
    "endsAt" TIMESTAMPTZ NOT NULL,
    "timezone" TEXT NOT NULL,
    "location" TEXT,
    "audienceId" TEXT NOT NULL,
    "calledByUserId" TEXT NOT NULL,
    "orgUnitId" TEXT NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "cancelledAt" TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS "pricelist" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "key" TEXT NOT NULL UNIQUE,
    "name" TEXT NOT NULL,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "position" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "title" TEXT NOT NULL,
    "orgUnitId" TEXT NOT NULL,
    "rankId" TEXT NOT NULL,
    "reportsToId" TEXT
);

CREATE TABLE IF NOT EXISTS "rank" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "key" TEXT NOT NULL UNIQUE,
    "label" TEXT NOT NULL,
    "level" INTEGER NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS "ordernumbercounter" (
    "year" INTEGER PRIMARY KEY,
    "counter" INTEGER DEFAULT 0 NOT NULL
);

CREATE TABLE IF NOT EXISTS "bomcomponent" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "recipeId" TEXT NOT NULL,
    "rawMaterialId" TEXT NOT NULL,
    "quantityBase" INTEGER NOT NULL,
    "wastageBps" INTEGER,
    "isOptional" BOOLEAN DEFAULT FALSE NOT NULL,
    "note" TEXT,
    "order" INTEGER DEFAULT 0 NOT NULL
);

CREATE TABLE IF NOT EXISTS "newslettersubscriber" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "email" TEXT NOT NULL UNIQUE,
    "locale" TEXT NOT NULL,
    "source"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`TEXT' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "subscribedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "optedOutAt" TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS "bankfacilityrepayment" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "facilityId" TEXT NOT NULL,
    "principalMinor" INTEGER DEFAULT 0 NOT NULL,
    "markupMinor" INTEGER DEFAULT 0 NOT NULL,
    "repaymentDate" TIMESTAMPTZ NOT NULL,
    "note" TEXT,
    "createdById" TEXT NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "node" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "nodeId" TEXT NOT NULL UNIQUE,
    "role" noderole NOT NULL,
    "label" TEXT NOT NULL,
    "url" TEXT NOT NULL,
    "inboundApiKeyHash" TEXT UNIQUE,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`nodestatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "lastSeenAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "retailBrandId" TEXT
);

CREATE TABLE IF NOT EXISTS "promocode" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "discountId" TEXT NOT NULL,
    "code" TEXT NOT NULL UNIQUE,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "usageLimit" INTEGER,
    "usageCount" INTEGER DEFAULT 0 NOT NULL,
    "restrictedToUserId" TEXT,
    "expiresAt" TIMESTAMPTZ,
    "batchId" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "campaign" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "name" TEXT NOT NULL,
    "channel" campaignchannel NOT NULL,
    "segmentId" TEXT NOT NULL,
    "content" JSONB NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`campaignstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "scheduledAt" TIMESTAMPTZ,
    "sentAt" TIMESTAMPTZ,
    "createdById" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "userroleassignment" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "userId" TEXT NOT NULL,
    "roleId" TEXT NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "wishlistitem" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "userId" TEXT NOT NULL,
    "productId" TEXT NOT NULL,
    "variantId" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "orderitem" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "orderId" TEXT NOT NULL,
    "productId" TEXT,
    "variantId" TEXT,
    "kind" productkind NOT NULL,
    "nameEn" TEXT NOT NULL,
    "nameUr" TEXT,
    "sku" TEXT,
    "imageUrl" TEXT,
    "variantLabel" TEXT,
    "unitPrice" INTEGER NOT NULL,
    "unitCost" INTEGER DEFAULT 0 NOT NULL,
    "quantity" INTEGER DEFAULT 1 NOT NULL,
    "stitchingFeeUnit" INTEGER DEFAULT 0 NOT NULL,
    "discountUnit" INTEGER DEFAULT 0 NOT NULL,
    "lineTotal" INTEGER NOT NULL,
    "taxAmount" INTEGER DEFAULT 0 NOT NULL,
    "taxRateBps" INTEGER DEFAULT 0 NOT NULL,
    "stitchingServiceId" TEXT,
    "stitchingServiceName" TEXT,
    "garmentTypeKey" TEXT,
    "measurementSnapshot" JSONB,
    "optionsSnapshot" JSONB,
    "stitchingNotes" TEXT,
    "measurementValues" JSONB,
    "customDesignText" JSONB,
    "sizeToken" TEXT,
    "replenishmentReminderSentAt" TIMESTAMPTZ,
    "fulfillmentStatus"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`orderitemfulfillmentstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "awaitingSince" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "approvalchain" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "subjectType" approvalsubjecttype NOT NULL,
    "subjectId" TEXT NOT NULL,
    "currentStep" INTEGER DEFAULT 1 NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`approvalchainstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "replicationrecord" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "originNodeId" TEXT NOT NULL,
    "sequence" TEXT NOT NULL,
    "entity" TEXT NOT NULL,
    "entityId" TEXT NOT NULL,
    "dataClass" replicationclass NOT NULL,
    "direction" replicationdirection NOT NULL,
    "op" replicationop NOT NULL,
    "payload" JSONB NOT NULL,
    "schemaVersion" INTEGER NOT NULL,
    "occurredAt" TIMESTAMPTZ NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "microfrontendfragment" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "key" TEXT NOT NULL UNIQUE,
    "label" TEXT NOT NULL,
    "remoteUrl" TEXT NOT NULL,
    "mountPoints" TEXT[] NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`fragmentstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "attestedAt" TIMESTAMPTZ NOT NULL,
    "attestedByUserId" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "formmetadata" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "entityType" TEXT NOT NULL UNIQUE,
    "fields" JSONB NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "stitchingjobnumbercounter" (
    "year" INTEGER PRIMARY KEY,
    "counter" INTEGER DEFAULT 0 NOT NULL
);

CREATE TABLE IF NOT EXISTS "bomproductionrun" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "number" TEXT NOT NULL UNIQUE,
    "recipeId" TEXT NOT NULL,
    "variantId" TEXT NOT NULL,
    "runs" INTEGER DEFAULT 1 NOT NULL,
    "quantityProduced" INTEGER NOT NULL,
    "materialCost" INTEGER NOT NULL,
    "labourCost" INTEGER NOT NULL,
    "overheadCost" INTEGER NOT NULL,
    "producedUnitCost" INTEGER NOT NULL,
    "notes" TEXT,
    "operatorId" TEXT,
    "producedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "scopeLevel"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`scopelevel' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "originNodeId" TEXT,
    "locationId" TEXT NOT NULL,
    "outputLotId" TEXT
);

CREATE TABLE IF NOT EXISTS "productimagetranslation" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "imageId" TEXT NOT NULL,
    "locale" TEXT NOT NULL,
    "alt" TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS "requestoutcome" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "path" TEXT NOT NULL,
    "succeeded" BOOLEAN NOT NULL,
    "latencyMs" INTEGER NOT NULL,
    "occurredAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "walletentry" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "accountId" TEXT NOT NULL,
    "type" walletentrytype NOT NULL,
    "amountMinor" INTEGER NOT NULL,
    "orderId" TEXT,
    "sourceRef" TEXT NOT NULL,
    "actorId" TEXT,
    "reason" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "bomrecipestage" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "recipeId" TEXT NOT NULL,
    "stepNumber" INTEGER NOT NULL,
    "operationName" TEXT NOT NULL,
    "workCenterId" TEXT,
    "setupTimeMinutes" INTEGER DEFAULT 0 NOT NULL,
    "runTimeMinutesPerUnit" INTEGER DEFAULT 0 NOT NULL,
    "scrapBps" INTEGER DEFAULT 0 NOT NULL
);

CREATE TABLE IF NOT EXISTS "stitchingorder" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "number" TEXT NOT NULL UNIQUE,
    "userId" TEXT,
    "contactId" TEXT,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`stitchingorderstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "guestName" TEXT,
    "guestPhone" TEXT,
    "guestEmail" TEXT,
    "pickupAddress" JSONB,
    "deliveryAddress" JSONB,
    "fabricNotes" TEXT,
    "staffNotes" TEXT,
    "quotedFee" INTEGER,
    "fabricReceivedAt" TIMESTAMPTZ,
    "promisedAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "stitchingoptiongroup" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "key" TEXT NOT NULL UNIQUE,
    "labelEn" TEXT NOT NULL,
    "labelUr" TEXT NOT NULL,
    "garmentTypeId" TEXT,
    "area" TEXT,
    "selectionType"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`stitchingselectiontype' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "required" BOOLEAN DEFAULT FALSE NOT NULL,
    "order" INTEGER DEFAULT 0 NOT NULL,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "allowCustomText" BOOLEAN DEFAULT FALSE NOT NULL,
    "helpEn" TEXT,
    "helpUr" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "posshift" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "terminalId" TEXT NOT NULL,
    "openedByUserId" TEXT NOT NULL,
    "closedByUserId" TEXT,
    "openingCashMinor" INTEGER NOT NULL,
    "closingCashMinor" INTEGER,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`posshiftstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "openedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "closedAt" TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS "proxybid" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "lotId" TEXT NOT NULL,
    "bidderId" TEXT NOT NULL,
    "maxCipher" TEXT NOT NULL,
    "placedAfterSeq" INTEGER NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "giftcardtransaction" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "giftCardId" TEXT NOT NULL,
    "delta" INTEGER NOT NULL,
    "balanceAfter" INTEGER NOT NULL,
    "reason" TEXT NOT NULL,
    "orderId" TEXT,
    "actorId" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "banner" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "placementKey" TEXT NOT NULL,
    "imageUrl" TEXT NOT NULL,
    "linkUrl" TEXT,
    "order" INTEGER DEFAULT 0 NOT NULL,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "startsAt" TIMESTAMPTZ,
    "endsAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "partnerfactinbox" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "tradingPartnerId" TEXT NOT NULL,
    "factId" TEXT NOT NULL,
    "type" TEXT NOT NULL,
    "schemaVersion" INTEGER NOT NULL,
    "documentRef" TEXT,
    "occurredAt" TIMESTAMPTZ NOT NULL,
    "payloadHash" TEXT NOT NULL,
    "payload" JSONB,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`partnerfactstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "statusReason" TEXT,
    "appliedPurchaseOrderId" TEXT,
    "receivedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "backinstockrequest" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "userId" TEXT NOT NULL,
    "variantId" TEXT NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "lot" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "productVariantId" TEXT NOT NULL,
    "lotCode" TEXT NOT NULL,
    "expiresOn" TIMESTAMPTZ,
    "receivedOn" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "sourceReceiptId" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "stockcount" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "countNumber" TEXT NOT NULL UNIQUE,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`stockcountstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "note" TEXT,
    "startedAt" TIMESTAMPTZ,
    "completedAt" TIMESTAMPTZ,
    "cancelledAt" TIMESTAMPTZ,
    "createdBy" TEXT,
    "assignedTo" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "bomproductionline" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "runId" TEXT NOT NULL,
    "rawMaterialId" TEXT NOT NULL,
    "variantId" TEXT NOT NULL,
    "quantityBase" INTEGER NOT NULL,
    "unitCost" INTEGER NOT NULL,
    "lineCost" INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS "payrunnumbercounter" (
    "year" INTEGER PRIMARY KEY,
    "counter" INTEGER DEFAULT 0 NOT NULL
);

CREATE TABLE IF NOT EXISTS "policyquarantine" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "originNodeId" TEXT NOT NULL,
    "entity" TEXT NOT NULL,
    "entityId" TEXT NOT NULL,
    "policyKey" TEXT NOT NULL,
    "retailBrandId" TEXT,
    "payload" JSONB NOT NULL,
    "reason" TEXT NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`replicationquarantinestatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "resolvedAt" TIMESTAMPTZ,
    "resolvedById" TEXT,
    "resolvedNote" TEXT
);

CREATE TABLE IF NOT EXISTS "bankfacility" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "bankId" TEXT NOT NULL,
    "key" TEXT NOT NULL UNIQUE,
    "label" TEXT NOT NULL,
    "facilityType" bankfacilitytype NOT NULL,
    "limitMinor" INTEGER NOT NULL,
    "markupRateBps" INTEGER NOT NULL,
    "isFloatingRate" BOOLEAN DEFAULT FALSE NOT NULL,
    "securityReference" TEXT,
    "startDate" TIMESTAMPTZ NOT NULL,
    "maturityDate" TIMESTAMPTZ NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`bankfacilitystatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "asset" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "assetTag" TEXT NOT NULL UNIQUE,
    "category" TEXT NOT NULL,
    "description" TEXT NOT NULL,
    "purchaseCostMinor" INTEGER,
    "purchasedOn" TIMESTAMPTZ,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`assetstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "categorypreset" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "key" TEXT NOT NULL UNIQUE,
    "label" TEXT NOT NULL,
    "treeJson" JSONB NOT NULL,
    "description" TEXT
);

CREATE TABLE IF NOT EXISTS "transfernumbercounter" (
    "year" INTEGER PRIMARY KEY,
    "counter" INTEGER DEFAULT 0 NOT NULL
);

CREATE TABLE IF NOT EXISTS "productgarmenttype" (
    "productId" TEXT NOT NULL,
    "garmentTypeId" TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS "stockvariance" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "variantId" TEXT NOT NULL,
    "locationId" TEXT NOT NULL,
    "overshootQty" INTEGER NOT NULL,
    "stockMovementId" TEXT NOT NULL UNIQUE,
    "refType" TEXT,
    "refId" TEXT,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`stockvariancestatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "resolvedByUserId" TEXT,
    "resolvedAt" TIMESTAMPTZ,
    "resolutionNote" TEXT,
    "posShiftId" TEXT,
    "serializedUnitId" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "dottedlinereport" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "userId" TEXT NOT NULL,
    "dottedManagerUserId" TEXT NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "quote" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "number" TEXT NOT NULL UNIQUE,
    "tradingPartnerId" TEXT NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`quotestatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "currency" TEXT NOT NULL,
    "validUntil" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "payrunline" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "payRunId" TEXT NOT NULL,
    "userId" TEXT NOT NULL,
    "compensationRateId" TEXT NOT NULL,
    "grossMinor" INTEGER NOT NULL,
    "unpaidLeaveDays" DOUBLE PRECISION DEFAULT 0 NOT NULL,
    "unpaidLeaveMinor" INTEGER DEFAULT 0 NOT NULL,
    "deductions" JSONB NOT NULL,
    "deductionsMinor" INTEGER DEFAULT 0 NOT NULL,
    "netMinor" INTEGER NOT NULL,
    "taxBracketVersion" INTEGER
);

CREATE TABLE IF NOT EXISTS "debitnote" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "number" TEXT NOT NULL UNIQUE,
    "supplierId" TEXT NOT NULL,
    "supplierBillId" TEXT,
    "amountMinor" INTEGER NOT NULL,
    "reason" TEXT NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`creditdebitnotestatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "createdById" TEXT NOT NULL,
    "issuedAt" TIMESTAMPTZ,
    "voidedAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "spotcheckline" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "spotCheckId" TEXT NOT NULL,
    "variantId" TEXT NOT NULL,
    "expected" INTEGER NOT NULL,
    "counted" INTEGER,
    "variance" INTEGER
);

CREATE TABLE IF NOT EXISTS "poslinequantitysource" (
    "lineRef" TEXT PRIMARY KEY,
    "source" quantitysource NOT NULL,
    "recipeId" TEXT
);

CREATE TABLE IF NOT EXISTS "payrolltaxbracket" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "floorMinor" INTEGER NOT NULL,
    "ceilingMinor" INTEGER,
    "rateBps" INTEGER NOT NULL,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "version" INTEGER DEFAULT 1 NOT NULL
);

CREATE TABLE IF NOT EXISTS "assetdepreciationschedule" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "assetId" TEXT NOT NULL UNIQUE,
    "method" depreciationmethod NOT NULL,
    "acquisitionCostMinor" INTEGER NOT NULL,
    "salvageValueMinor" INTEGER DEFAULT 0 NOT NULL,
    "usefulLifeMonths" INTEGER NOT NULL,
    "decliningRateBps" INTEGER,
    "startDate" TIMESTAMPTZ NOT NULL,
    "createdById" TEXT NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "closedAt" TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS "bomproductionlinelot" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "productionLineId" TEXT NOT NULL,
    "lotId" TEXT NOT NULL,
    "quantityBase" INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS "supplierbill" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "supplierId" TEXT NOT NULL,
    "purchaseOrderId" TEXT,
    "reference" TEXT,
    "amount" INTEGER NOT NULL,
    "amountPaid" INTEGER DEFAULT 0 NOT NULL,
    "currency"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`TEXT' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "fxRateScaled" INTEGER DEFAULT 1000000 NOT NULL,
    "baseAmountMinor" INTEGER NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`supplierbillstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "matchReviewReason" TEXT,
    "issuedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "dueAt" TIMESTAMPTZ,
    "paidAt" TIMESTAMPTZ,
    "notes" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "bulletinacknowledgement" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "bulletinId" TEXT NOT NULL,
    "userId" TEXT NOT NULL,
    "acknowledgedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "approvalstep" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "chainId" TEXT NOT NULL,
    "stepOrder" INTEGER NOT NULL,
    "approverRoleId" TEXT,
    "approverUserId" TEXT,
    "approverContactIds" TEXT[] DEFAULT '{}' NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`approvalstepstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "decidedAt" TIMESTAMPTZ,
    "decidedById" TEXT,
    "decidedByContactId" TEXT,
    "comment" TEXT
);

CREATE TABLE IF NOT EXISTS "creditnote" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "number" TEXT NOT NULL UNIQUE,
    "customerId" TEXT NOT NULL,
    "orderId" TEXT,
    "appliedToOrderId" TEXT,
    "amountMinor" INTEGER NOT NULL,
    "reason" TEXT NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`creditdebitnotestatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "createdById" TEXT NOT NULL,
    "approvedById" TEXT,
    "issuedAt" TIMESTAMPTZ,
    "voidedAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "ssoconnection" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "protocol" ssoprotocol NOT NULL,
    "label" TEXT NOT NULL,
    "issuerUrl" TEXT NOT NULL,
    "clientId" TEXT NOT NULL,
    "credentialEncrypted" TEXT NOT NULL,
    "allowedEmailDomains" TEXT[] NOT NULL,
    "defaultRoleName" TEXT NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`ssoconnectionstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "enforceSso" BOOLEAN DEFAULT FALSE NOT NULL,
    "createdByUserId" TEXT NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "bulletin" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "authorId" TEXT NOT NULL,
    "title" TEXT NOT NULL,
    "bodyMarkdown" TEXT NOT NULL,
    "category" bulletincategory NOT NULL,
    "audienceOrgUnitId" TEXT,
    "publishedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "expiresAt" TIMESTAMPTZ,
    "requiresAcknowledgement" BOOLEAN DEFAULT FALSE NOT NULL,
    "archivedAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "orderstatusevent" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "orderId" TEXT NOT NULL,
    "fromStatus" orderstatus,
    "toStatus" orderstatus NOT NULL,
    "note" TEXT,
    "actorId" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "journalentrynumbercounter" (
    "fiscalYearId" TEXT PRIMARY KEY,
    "counter" INTEGER DEFAULT 0 NOT NULL
);

CREATE TABLE IF NOT EXISTS "storecredittransaction" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "userId" TEXT NOT NULL,
    "delta" INTEGER NOT NULL,
    "balanceAfter" INTEGER NOT NULL,
    "reason" TEXT NOT NULL,
    "orderId" TEXT,
    "actorId" TEXT,
    "note" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "serializedunit" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "productVariantId" TEXT NOT NULL,
    "serialNumber" TEXT NOT NULL UNIQUE,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`serializedunitstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "lotId" TEXT,
    "locationId" TEXT,
    "orderId" TEXT,
    "stockMovementId" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "brandtranslation" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "brandId" TEXT NOT NULL,
    "locale" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "description" TEXT
);

CREATE TABLE IF NOT EXISTS "garmentsizechart" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "garmentTypeId" TEXT NOT NULL,
    "sizeToken" TEXT NOT NULL,
    "values" JSONB NOT NULL,
    "order" INTEGER DEFAULT 0 NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "bpmnworkflowdefinition" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "subjectType" approvalsubjecttype NOT NULL UNIQUE,
    "diagramXml" TEXT NOT NULL,
    "compiledSteps" JSONB NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "leaveaccrualrun" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "leaveTypeId" TEXT NOT NULL,
    "period" TEXT NOT NULL,
    "ranAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "stocktransfer" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "number" TEXT NOT NULL UNIQUE,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`stocktransferstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "settlementMode"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`settlementmode' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "sourceLocationId" TEXT NOT NULL,
    "destinationLocationId" TEXT NOT NULL,
    "inTransitLocationId" TEXT NOT NULL,
    "createdByUserId" TEXT NOT NULL,
    "receivedByUserId" TEXT,
    "reconciledByUserId" TEXT,
    "note" TEXT,
    "discrepancyReason" TEXT,
    "dispatchedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "receivedAt" TIMESTAMPTZ,
    "reconciledAt" TIMESTAMPTZ,
    "cancelledAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "cart" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "token" TEXT NOT NULL UNIQUE,
    "userId" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "expiresAt" TIMESTAMPTZ NOT NULL,
    "reminderSentAt" TIMESTAMPTZ,
    "incentiveSentAt" TIMESTAMPTZ,
    "firstTouchSentAt" TIMESTAMPTZ,
    "incentivePromoCodeId" TEXT,
    "guestPhone" TEXT,
    "guestMarketingConsentAt" TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS "spotcheckpolicy" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "locationId" TEXT,
    "minSkusPerCheck" INTEGER DEFAULT 5 NOT NULL,
    "frequencyDays" INTEGER DEFAULT 3 NOT NULL,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "measurementprofile" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "userId" TEXT NOT NULL,
    "garmentTypeId" TEXT NOT NULL,
    "label" TEXT NOT NULL,
    "values" JSONB NOT NULL,
    "notes" TEXT,
    "isDefault" BOOLEAN DEFAULT FALSE NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "b2bpurchaseapprovalpolicy" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "b2bAccountId" TEXT NOT NULL UNIQUE,
    "thresholdMinor" INTEGER NOT NULL,
    "approverContactIds" TEXT[] NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "cartitem" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "cartId" TEXT NOT NULL,
    "productId" TEXT NOT NULL,
    "variantId" TEXT NOT NULL,
    "quantity" INTEGER DEFAULT 1 NOT NULL,
    "garmentTypeId" TEXT,
    "stitchingServiceId" TEXT,
    "measurementProfileId" TEXT,
    "stitchingNotes" TEXT,
    "measurementValues" JSONB,
    "customDesignText" JSONB,
    "sizeToken" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "markdownsuggestion" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "variantId" TEXT NOT NULL,
    "discountId" TEXT,
    "suggestedBps" INTEGER NOT NULL,
    "estimatedClearancePct" DOUBLE PRECISION NOT NULL,
    "estimatedClearanceDays" INTEGER NOT NULL,
    "marginBeforePaisa" INTEGER NOT NULL,
    "marginAfterPaisa" INTEGER NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`markdownsuggestionstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "resolvedAt" TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS "taskdependency" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "predecessorTaskId" TEXT NOT NULL,
    "successorTaskId" TEXT NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "fiscalyear" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "name" TEXT NOT NULL UNIQUE,
    "startsOn" TIMESTAMPTZ NOT NULL,
    "endsOn" TIMESTAMPTZ NOT NULL,
    "closedAt" TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS "leavebalance" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "userId" TEXT NOT NULL,
    "leaveTypeId" TEXT NOT NULL,
    "balance" DOUBLE PRECISION DEFAULT 0 NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "bankfacilitydrawdown" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "facilityId" TEXT NOT NULL,
    "amountMinor" INTEGER NOT NULL,
    "drawdownDate" TIMESTAMPTZ NOT NULL,
    "note" TEXT,
    "createdById" TEXT NOT NULL,
    "approvedById" TEXT,
    "postedAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "dunningevent" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "orderId" TEXT NOT NULL,
    "stepIndex" INTEGER NOT NULL,
    "sentAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "brandpolicy" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "retailBrandId" TEXT NOT NULL UNIQUE,
    "allowBranchProductCreation" BOOLEAN DEFAULT FALSE NOT NULL,
    "allowBranchLocalPurchasing" BOOLEAN DEFAULT FALSE NOT NULL,
    "allowInterBranchTransfers" BOOLEAN DEFAULT FALSE NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "lifecyclechecklisttask" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "checklistId" TEXT NOT NULL,
    "taskKey" TEXT NOT NULL,
    "title" TEXT NOT NULL,
    "description" TEXT NOT NULL,
    "assignedRole" clearancerole NOT NULL,
    "requiredForCompletion" BOOLEAN NOT NULL,
    "prerequisiteTaskKeys" JSONB,
    "completed" BOOLEAN DEFAULT FALSE NOT NULL,
    "completedByUserId" TEXT,
    "completedByUserRole" clearancerole,
    "completedAt" TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS "productoption" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "productId" TEXT NOT NULL,
    "key" TEXT NOT NULL,
    "display"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`optiondisplay' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "order" INTEGER DEFAULT 0 NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "manifestnumbercounter" (
    "year" INTEGER PRIMARY KEY,
    "counter" INTEGER DEFAULT 0 NOT NULL
);

CREATE TABLE IF NOT EXISTS "notificationpreference" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "userId" TEXT NOT NULL,
    "channel" notificationpreferencechannel NOT NULL,
    "category" notificationpreferencecategory NOT NULL,
    "enabled" BOOLEAN DEFAULT TRUE NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "customersupportmessage" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "ticketId" TEXT NOT NULL,
    "authorType" supportmessageauthortype NOT NULL,
    "contactId" TEXT,
    "userId" TEXT,
    "body" TEXT NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "expense" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "categoryId" TEXT NOT NULL,
    "amount" INTEGER NOT NULL,
    "currency"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`TEXT' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "description" TEXT NOT NULL,
    "incurredAt" TIMESTAMPTZ NOT NULL,
    "paidAt" TIMESTAMPTZ,
    "payee" TEXT,
    "supplierId" TEXT,
    "reference" TEXT,
    "receiptUrl" TEXT,
    "recurrence"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`expenserecurrence' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "recurringFromId" TEXT,
    "recurringPeriod" TEXT,
    "createdById" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "scopeLevel"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`scopelevel' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "originNodeId" TEXT
);

CREATE TABLE IF NOT EXISTS "walletaccount" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "organizationId" TEXT,
    "customerId" TEXT NOT NULL UNIQUE,
    "currency"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`TEXT' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`walletaccountstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "shipment" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "orderId" TEXT NOT NULL,
    "courier"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`TEXT' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "trackingNumber" TEXT,
    "trackingUrl" TEXT,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`shipmentstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "cost" INTEGER,
    "dispatchedAt" TIMESTAMPTZ,
    "deliveredAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "methodId" TEXT,
    "weightGrams" INTEGER,
    "packageCount" INTEGER DEFAULT 1 NOT NULL,
    "labelUrl" TEXT,
    "manifestId" TEXT,
    "meta" JSONB,
    "isRto" BOOLEAN DEFAULT FALSE NOT NULL,
    "rtoAt" TIMESTAMPTZ,
    "failedAttempts" INTEGER DEFAULT 0 NOT NULL,
    "lastPolledAt" TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS "posting" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "userId" TEXT NOT NULL,
    "positionId" TEXT NOT NULL,
    "isPrimary" BOOLEAN DEFAULT TRUE NOT NULL,
    "startedOn" TIMESTAMPTZ NOT NULL,
    "endedOn" TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS "task" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "reference" TEXT NOT NULL,
    "title" TEXT NOT NULL,
    "detail" TEXT,
    "positionId" TEXT NOT NULL,
    "assigneeUserId" TEXT,
    "orgUnitId" TEXT NOT NULL,
    "state"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`taskstate' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "stageId" TEXT,
    "priority" INTEGER DEFAULT 0 NOT NULL,
    "dueAt" TIMESTAMPTZ,
    "estimatedDurationDays" INTEGER,
    "sourceType" TEXT,
    "sourceId" TEXT,
    "sourceRef" TEXT,
    "createdByUserId" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "closedAt" TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS "rawmaterial" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "variantId" TEXT NOT NULL UNIQUE,
    "name" TEXT NOT NULL,
    "kind"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`TEXT' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "unit" rawmaterialunit NOT NULL,
    "unitScale" INTEGER DEFAULT 1 NOT NULL,
    "notes" TEXT,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "orgunit" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "code" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "typeId" TEXT NOT NULL,
    "parentId" TEXT,
    "path" TEXT[] NOT NULL,
    "depth" INTEGER DEFAULT 0 NOT NULL,
    "nodeId" TEXT,
    "retailBrandId" TEXT,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "closedAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "menuitemtranslation" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "menuItemId" TEXT NOT NULL,
    "locale" TEXT NOT NULL,
    "label" TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS "returnrequest" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "number" TEXT NOT NULL UNIQUE,
    "orderId" TEXT NOT NULL,
    "userId" TEXT,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`returnstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "resolution" returnresolution,
    "customerNote" TEXT,
    "staffNote" TEXT,
    "photoUrls" TEXT[] DEFAULT '{}' NOT NULL,
    "requestedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "approvedAt" TIMESTAMPTZ,
    "receivedAt" TIMESTAMPTZ,
    "inspectedAt" TIMESTAMPTZ,
    "refundedAt" TIMESTAMPTZ,
    "refundAmount" INTEGER,
    "restockingFeeMinor" INTEGER
);

CREATE TABLE IF NOT EXISTS "fiscalperiod" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "fiscalYearId" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "startsOn" TIMESTAMPTZ NOT NULL,
    "endsOn" TIMESTAMPTZ NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`fiscalperiodstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "role" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "name" TEXT NOT NULL UNIQUE,
    "description" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "payrun" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "number" TEXT NOT NULL UNIQUE,
    "periodStart" TIMESTAMPTZ NOT NULL,
    "periodEnd" TIMESTAMPTZ NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`payrunstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "scopeLevel"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`scopelevel' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "originNodeId" TEXT,
    "approvedById" TEXT,
    "approvedAt" TIMESTAMPTZ,
    "postedAt" TIMESTAMPTZ,
    "createdById" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "aiusagelog" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "feature" aifeature NOT NULL,
    "kind" aicallkind NOT NULL,
    "provider" TEXT NOT NULL,
    "model" TEXT NOT NULL,
    "inputTokens" INTEGER DEFAULT 0 NOT NULL,
    "outputTokens" INTEGER DEFAULT 0 NOT NULL,
    "costPaisa" INTEGER DEFAULT 0 NOT NULL,
    "succeeded" BOOLEAN DEFAULT TRUE NOT NULL,
    "errorMessage" TEXT,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "menuitem" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "menuKey" TEXT NOT NULL,
    "parentId" TEXT,
    "url" TEXT NOT NULL,
    "order" INTEGER DEFAULT 0 NOT NULL,
    "isActive" BOOLEAN DEFAULT TRUE NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "contacttag" (
    "contactId" TEXT NOT NULL,
    "tagId" TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS "walletadjustmentrequest" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "accountId" TEXT NOT NULL,
    "amountMinor" INTEGER NOT NULL,
    "reason" TEXT NOT NULL,
    "requestedByUserId" TEXT NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`walletadjustmentrequeststatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "walletEntryId" TEXT UNIQUE,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "decidedAt" TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS "optiontemplatevalue" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "templateId" TEXT NOT NULL,
    "key" TEXT NOT NULL,
    "swatchHex" TEXT,
    "swatchUrl" TEXT,
    "order" INTEGER DEFAULT 0 NOT NULL
);

CREATE TABLE IF NOT EXISTS "tradingpartner" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "name" TEXT NOT NULL,
    "kind" tradingpartnerkind NOT NULL,
    "outboundBaseUrl" TEXT,
    "outboundApiKeyEncrypted" TEXT,
    "inboundApiKeyHash" TEXT UNIQUE,
    "acceptedRole" partyrolename,
    "inviteTokenHash" TEXT UNIQUE,
    "invitedAt" TIMESTAMPTZ,
    "acceptedAt" TIMESTAMPTZ,
    "contactEmail" TEXT,
    "inboundKeyExpiresAt" TIMESTAMPTZ,
    "inboundKeyRotatedByUserId" TEXT,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`tradingpartnerstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "giftcard" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "code" TEXT NOT NULL UNIQUE,
    "initialValue" INTEGER NOT NULL,
    "balance" INTEGER NOT NULL,
    "status"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`giftcardstatus' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "currency"  param($m) $type = $m.Groups[1].Value.ToLower(); $upperVal = $m.Groups[2].Value; if ($enums.ContainsKey($type) -and $enums[$type].ContainsKey($upperVal)) { $lowerVal = $enums[$type][$upperVal]; $script:count++; "'`TEXT' DEFAULT '$lowerVal'::`$type" } else { $m.Value }  NOT NULL,
    "purchasedByUserId" TEXT,
    "recipientName" TEXT,
    "recipientEmail" TEXT,
    "recipientPhone" TEXT,
    "message" TEXT,
    "sourceOrderId" TEXT,
    "expiresAt" TIMESTAMPTZ,
    "issuedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS "featureflag" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "key" TEXT NOT NULL UNIQUE,
    "description" TEXT NOT NULL,
    "rolloutPercent" INTEGER DEFAULT 0 NOT NULL,
    "enabledOrgIds" TEXT[] NOT NULL,
    "killSwitch" BOOLEAN DEFAULT FALSE NOT NULL,
    "moneyOrStockPath" BOOLEAN DEFAULT FALSE NOT NULL,
    "proposedRolloutPercent" INTEGER,
    "proposedKillSwitchOff" BOOLEAN DEFAULT FALSE NOT NULL,
    "proposedById" TEXT,
    "proposedAt" TIMESTAMPTZ,
    "approvedById" TEXT,
    "approvedAt" TIMESTAMPTZ,
    "createdAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "updatedAt" TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    "removalCriteria" TEXT
);

CREATE TABLE IF NOT EXISTS "producttranslation" (
    "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    "productId" TEXT NOT NULL,
    "locale" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "description" TEXT,
    "careInstructions" TEXT,
    "metaTitle" TEXT,
    "metaDescription" TEXT,
    "focusKeyword" TEXT,
    "sourcedFromGlobalReferenceId" TEXT,
    "sourcedFromReferenceUpdatedAt" TIMESTAMPTZ,
    "searchVector" TEXT NOT NULL,
    "embedding" TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS "idx_retailbrand_organizationId" ON "retailbrand" (organizationId);
CREATE INDEX IF NOT EXISTS "idx_retailbrand_parentId" ON "retailbrand" (parentId);

CREATE INDEX IF NOT EXISTS "idx_ticketmessage_ticketId_createdAt" ON "ticketmessage" (ticketId, createdAt);

ALTER TABLE "productoptionvaluetranslation" ADD CONSTRAINT "uniq_productoptionvaluetranslation_valueId_locale" UNIQUE (valueId, locale);

CREATE INDEX IF NOT EXISTS "idx_searchquerylog_resultCount_createdAt" ON "searchquerylog" (resultCount, createdAt);
CREATE INDEX IF NOT EXISTS "idx_searchquerylog_term" ON "searchquerylog" (term);

CREATE INDEX IF NOT EXISTS "idx_compensationrate_userId_effectiveFrom" ON "compensationrate" (userId, effectiveFrom);

CREATE INDEX IF NOT EXISTS "idx_orderitem_orderId" ON "orderitem" (orderId);
CREATE INDEX IF NOT EXISTS "idx_orderitem_productId" ON "orderitem" (productId);

CREATE INDEX IF NOT EXISTS "idx_shippingrate_zoneId_methodId" ON "shippingrate" (zoneId, methodId);

CREATE INDEX IF NOT EXISTS "idx_broadcastmessage_senderId_createdAt" ON "broadcastmessage" (senderId, createdAt);

CREATE INDEX IF NOT EXISTS "idx_inboundemailreply_inboundEmailId_sentAt" ON "inboundemailreply" (inboundEmailId, sentAt);

ALTER TABLE "returnreasontranslation" ADD CONSTRAINT "uniq_returnreasontranslation_reasonId_locale" UNIQUE (reasonId, locale);

CREATE INDEX IF NOT EXISTS "idx_productcrosssell_relatedProductId" ON "productcrosssell" (relatedProductId);

ALTER TABLE "productcrosssell" ADD CONSTRAINT "uniq_productcrosssell_productId_relatedProductId" UNIQUE (productId, relatedProductId);

CREATE INDEX IF NOT EXISTS "idx_stitchingoption_groupId_order" ON "stitchingoption" (groupId, order);

ALTER TABLE "stitchingoption" ADD CONSTRAINT "uniq_stitchingoption_groupId_key" UNIQUE (groupId, key);

CREATE INDEX IF NOT EXISTS "idx_workorder_status_createdAt" ON "workorder" (status, createdAt);
CREATE INDEX IF NOT EXISTS "idx_workorder_variantId" ON "workorder" (variantId);
CREATE INDEX IF NOT EXISTS "idx_workorder_sourceRefType_sourceRefId" ON "workorder" (sourceRefType, sourceRefId);

CREATE INDEX IF NOT EXISTS "idx_auctionsettlement_auctionId_status_createdAt" ON "auctionsettlement" (auctionId, status, createdAt);
CREATE INDEX IF NOT EXISTS "idx_auctionsettlement_status_dueAt" ON "auctionsettlement" (status, dueAt);
CREATE INDEX IF NOT EXISTS "idx_auctionsettlement_orderId" ON "auctionsettlement" (orderId);

ALTER TABLE "auctionsettlement" ADD CONSTRAINT "uniq_auctionsettlement_lotId_attempt" UNIQUE (lotId, attempt);

CREATE INDEX IF NOT EXISTS "idx_whatsappconversation_contactId_lastMessageAt" ON "whatsappconversation" (contactId, lastMessageAt);
CREATE INDEX IF NOT EXISTS "idx_whatsappconversation_phoneE164" ON "whatsappconversation" (phoneE164);

CREATE INDEX IF NOT EXISTS "idx_expensecategory_isActive_order" ON "expensecategory" (isActive, order);

CREATE INDEX IF NOT EXISTS "idx_userpermission_userId" ON "userpermission" (userId);

ALTER TABLE "userpermission" ADD CONSTRAINT "uniq_userpermission_userId_permissionId" UNIQUE (userId, permissionId);

CREATE INDEX IF NOT EXISTS "idx_bidder_auctionId_status_createdAt" ON "bidder" (auctionId, status, createdAt);

ALTER TABLE "bidder" ADD CONSTRAINT "uniq_bidder_auctionId_userId" UNIQUE (auctionId, userId);
ALTER TABLE "bidder" ADD CONSTRAINT "uniq_bidder_auctionId_alias" UNIQUE (auctionId, alias);

ALTER TABLE "onsitemessagetranslation" ADD CONSTRAINT "uniq_onsitemessagetranslation_onsiteMessageId_locale" UNIQUE (onsiteMessageId, locale);

CREATE INDEX IF NOT EXISTS "idx_lifecyclechecklist_userId_type" ON "lifecyclechecklist" (userId, type);
CREATE INDEX IF NOT EXISTS "idx_lifecyclechecklist_status" ON "lifecyclechecklist" (status);

CREATE INDEX IF NOT EXISTS "idx_emailverificationtoken_userId" ON "emailverificationtoken" (userId);
CREATE INDEX IF NOT EXISTS "idx_emailverificationtoken_expiresAt" ON "emailverificationtoken" (expiresAt);

CREATE INDEX IF NOT EXISTS "idx_policyquarantine_status" ON "policyquarantine" (status);
CREATE INDEX IF NOT EXISTS "idx_policyquarantine_entity_entityId" ON "policyquarantine" (entity, entityId);
CREATE INDEX IF NOT EXISTS "idx_policyquarantine_retailBrandId" ON "policyquarantine" (retailBrandId);

CREATE INDEX IF NOT EXISTS "idx_interbranchinvoice_sourceRetailBrandId" ON "interbranchinvoice" (sourceRetailBrandId);
CREATE INDEX IF NOT EXISTS "idx_interbranchinvoice_destinationRetailBrandId" ON "interbranchinvoice" (destinationRetailBrandId);

ALTER TABLE "proxybid" ADD CONSTRAINT "uniq_proxybid_lotId_bidderId" UNIQUE (lotId, bidderId);

CREATE INDEX IF NOT EXISTS "idx_interaction_contactId_occurredAt" ON "interaction" (contactId, occurredAt);
CREATE INDEX IF NOT EXISTS "idx_interaction_leadId_occurredAt" ON "interaction" (leadId, occurredAt);

CREATE INDEX IF NOT EXISTS "idx_customersupportticket_status_assignedToUserId" ON "customersupportticket" (status, assignedToUserId);
CREATE INDEX IF NOT EXISTS "idx_customersupportticket_contactId" ON "customersupportticket" (contactId);

CREATE INDEX IF NOT EXISTS "idx_optiontemplatevalue_templateId_order" ON "optiontemplatevalue" (templateId, order);

ALTER TABLE "optiontemplatevalue" ADD CONSTRAINT "uniq_optiontemplatevalue_templateId_key" UNIQUE (templateId, key);

CREATE INDEX IF NOT EXISTS "idx_attendancedevice_orgUnitId" ON "attendancedevice" (orgUnitId);
CREATE INDEX IF NOT EXISTS "idx_attendancedevice_status" ON "attendancedevice" (status);

CREATE INDEX IF NOT EXISTS "idx_emailaccount_purpose_isActive" ON "emailaccount" (purpose, isActive);

CREATE INDEX IF NOT EXISTS "idx_instalmentplan_status" ON "instalmentplan" (status);

CREATE INDEX IF NOT EXISTS "idx_invoiceline_invoiceId" ON "invoiceline" (invoiceId);

CREATE INDEX IF NOT EXISTS "idx_adminscope_userId" ON "adminscope" (userId);

ALTER TABLE "adminscope" ADD CONSTRAINT "uniq_adminscope_userId_level_retailBrandId_branchNodeId" UNIQUE (userId, level, retailBrandId, branchNodeId);

ALTER TABLE "bannertranslation" ADD CONSTRAINT "uniq_bannertranslation_bannerId_locale" UNIQUE (bannerId, locale);

CREATE INDEX IF NOT EXISTS "idx_workcenter_isActive" ON "workcenter" (isActive);
CREATE INDEX IF NOT EXISTS "idx_workcenter_orgUnitId" ON "workcenter" (orgUnitId);

CREATE INDEX IF NOT EXISTS "idx_auction_status_startsAt" ON "auction" (status, startsAt);
CREATE INDEX IF NOT EXISTS "idx_auction_organizationId" ON "auction" (organizationId);
CREATE INDEX IF NOT EXISTS "idx_auction_outletLocationId" ON "auction" (outletLocationId);

ALTER TABLE "optiontemplatetranslation" ADD CONSTRAINT "uniq_optiontemplatetranslation_templateId_locale" UNIQUE (templateId, locale);

CREATE INDEX IF NOT EXISTS "idx_productimage_productId_order" ON "productimage" (productId, order);

CREATE INDEX IF NOT EXISTS "idx_quote_tradingPartnerId" ON "quote" (tradingPartnerId);
CREATE INDEX IF NOT EXISTS "idx_quote_status" ON "quote" (status);

CREATE INDEX IF NOT EXISTS "idx_redirect_isAutomatic" ON "redirect" (isAutomatic);

CREATE INDEX IF NOT EXISTS "idx_tasktransition_taskId_occurredAt" ON "tasktransition" (taskId, occurredAt);

CREATE INDEX IF NOT EXISTS "idx_meetinginvitation_userId_response" ON "meetinginvitation" (userId, response);

ALTER TABLE "meetinginvitation" ADD CONSTRAINT "uniq_meetinginvitation_meetingId_userId" UNIQUE (meetingId, userId);

CREATE INDEX IF NOT EXISTS "idx_instalmentpayment_planId" ON "instalmentpayment" (planId);

CREATE INDEX IF NOT EXISTS "idx_address_userId" ON "address" (userId);

ALTER TABLE "bundlecomponent" ADD CONSTRAINT "uniq_bundlecomponent_bundleId_componentVariantId" UNIQUE (bundleId, componentVariantId);

CREATE INDEX IF NOT EXISTS "idx_employeeprofile_status" ON "employeeprofile" (status);

CREATE INDEX IF NOT EXISTS "idx_goodsreceipt_purchaseOrderId" ON "goodsreceipt" (purchaseOrderId);
CREATE INDEX IF NOT EXISTS "idx_goodsreceipt_locationId" ON "goodsreceipt" (locationId);

CREATE INDEX IF NOT EXISTS "idx_audience_orgUnitId" ON "audience" (orgUnitId);

CREATE INDEX IF NOT EXISTS "idx_productvariantoptionvalue_valueId" ON "productvariantoptionvalue" (valueId);

CREATE INDEX IF NOT EXISTS "idx_passwordresettoken_userId" ON "passwordresettoken" (userId);
CREATE INDEX IF NOT EXISTS "idx_passwordresettoken_expiresAt" ON "passwordresettoken" (expiresAt);

CREATE INDEX IF NOT EXISTS "idx_documentsource_organizationId_corpus_status" ON "documentsource" (organizationId, corpus, status);
CREATE INDEX IF NOT EXISTS "idx_documentsource_status" ON "documentsource" (status);

CREATE INDEX IF NOT EXISTS "idx_manifest_courierKey_status" ON "manifest" (courierKey, status);

CREATE INDEX IF NOT EXISTS "idx_customfielddefinition_entityType" ON "customfielddefinition" (entityType);

ALTER TABLE "customfielddefinition" ADD CONSTRAINT "uniq_customfielddefinition_entityType_key" UNIQUE (entityType, key);

CREATE INDEX IF NOT EXISTS "idx_order_status_placedAt" ON "order" (status, placedAt);
CREATE INDEX IF NOT EXISTS "idx_order_userId" ON "order" (userId);
CREATE INDEX IF NOT EXISTS "idx_order_contactId" ON "order" (contactId);
CREATE INDEX IF NOT EXISTS "idx_order_guestPhone" ON "order" (guestPhone);
CREATE INDEX IF NOT EXISTS "idx_order_channel" ON "order" (channel);
CREATE INDEX IF NOT EXISTS "idx_order_shiftId" ON "order" (shiftId);
CREATE INDEX IF NOT EXISTS "idx_order_sourceCartId" ON "order" (sourceCartId);
CREATE INDEX IF NOT EXISTS "idx_order_codRiskAction_placedAt" ON "order" (codRiskAction, placedAt);

ALTER TABLE "order" ADD CONSTRAINT "uniq_order_sourceType_sourceRef" UNIQUE (sourceType, sourceRef);

CREATE INDEX IF NOT EXISTS "idx_shift_userId_startsAt" ON "shift" (userId, startsAt);
CREATE INDEX IF NOT EXISTS "idx_shift_orgUnitId_startsAt" ON "shift" (orgUnitId, startsAt);

CREATE INDEX IF NOT EXISTS "idx_productvariant_productId" ON "productvariant" (productId);
CREATE INDEX IF NOT EXISTS "idx_productvariant_stock" ON "productvariant" (stock);

CREATE INDEX IF NOT EXISTS "idx_outboxevent_status_availableAt" ON "outboxevent" (status, availableAt);
CREATE INDEX IF NOT EXISTS "idx_outboxevent_name" ON "outboxevent" (name);

CREATE INDEX IF NOT EXISTS "idx_productaisuggestion_productId_status" ON "productaisuggestion" (productId, status);
CREATE INDEX IF NOT EXISTS "idx_productaisuggestion_status_createdAt" ON "productaisuggestion" (status, createdAt);

CREATE INDEX IF NOT EXISTS "idx_purchaseorderitem_purchaseOrderId" ON "purchaseorderitem" (purchaseOrderId);
CREATE INDEX IF NOT EXISTS "idx_purchaseorderitem_variantId" ON "purchaseorderitem" (variantId);

CREATE INDEX IF NOT EXISTS "idx_assetdepreciationschedule_closedAt" ON "assetdepreciationschedule" (closedAt);

CREATE INDEX IF NOT EXISTS "idx_whatsappmessage_conversationId_createdAt" ON "whatsappmessage" (conversationId, createdAt);

ALTER TABLE "optiontemplatevaluetranslation" ADD CONSTRAINT "uniq_optiontemplatevaluetranslation_valueId_locale" UNIQUE (valueId, locale);

CREATE INDEX IF NOT EXISTS "idx_auditlog_entity_entityId" ON "auditlog" (entity, entityId);
CREATE INDEX IF NOT EXISTS "idx_auditlog_actorId" ON "auditlog" (actorId);
CREATE INDEX IF NOT EXISTS "idx_auditlog_createdAt" ON "auditlog" (createdAt);

CREATE INDEX IF NOT EXISTS "idx_bid_bidderId_acceptedAt" ON "bid" (bidderId, acceptedAt);

ALTER TABLE "bid" ADD CONSTRAINT "uniq_bid_lotId_seq" UNIQUE (lotId, seq);
ALTER TABLE "bid" ADD CONSTRAINT "uniq_bid_lotId_bidderId_idempotencyKey" UNIQUE (lotId, bidderId, idempotencyKey);

CREATE INDEX IF NOT EXISTS "idx_cartstockhold_cartId" ON "cartstockhold" (cartId);
CREATE INDEX IF NOT EXISTS "idx_cartstockhold_status_expiresAt" ON "cartstockhold" (status, expiresAt);

CREATE INDEX IF NOT EXISTS "idx_pricelistassignment_customerId" ON "pricelistassignment" (customerId);
CREATE INDEX IF NOT EXISTS "idx_pricelistassignment_tagId" ON "pricelistassignment" (tagId);

CREATE INDEX IF NOT EXISTS "idx_poslinemodifier_lineRef" ON "poslinemodifier" (lineRef);

CREATE INDEX IF NOT EXISTS "idx_inboundemail_contactId" ON "inboundemail" (contactId);
CREATE INDEX IF NOT EXISTS "idx_inboundemail_receivedAt" ON "inboundemail" (receivedAt);

ALTER TABLE "inboundemail" ADD CONSTRAINT "uniq_inboundemail_emailAccountId_messageId" UNIQUE (emailAccountId, messageId);

CREATE INDEX IF NOT EXISTS "idx_walletentry_accountId_createdAt" ON "walletentry" (accountId, createdAt);

ALTER TABLE "walletentry" ADD CONSTRAINT "uniq_walletentry_accountId_type_sourceRef" UNIQUE (accountId, type, sourceRef);

CREATE INDEX IF NOT EXISTS "idx_posparkedsale_shiftId_retrievedAt" ON "posparkedsale" (shiftId, retrievedAt);

CREATE INDEX IF NOT EXISTS "idx_contact_status" ON "contact" (status);
CREATE INDEX IF NOT EXISTS "idx_contact_ownerId" ON "contact" (ownerId);
CREATE INDEX IF NOT EXISTS "idx_contact_lastOrderAt" ON "contact" (lastOrderAt);
CREATE INDEX IF NOT EXISTS "idx_contact_lifetimeValue" ON "contact" (lifetimeValue);
CREATE INDEX IF NOT EXISTS "idx_contact_rfmRecencyScore_rfmFrequencyScore_rfmMonetaryScore" ON "contact" (rfmRecencyScore, rfmFrequencyScore, rfmMonetaryScore);

CREATE INDEX IF NOT EXISTS "idx_bomproductionlinelot_productionLineId" ON "bomproductionlinelot" (productionLineId);
CREATE INDEX IF NOT EXISTS "idx_bomproductionlinelot_lotId" ON "bomproductionlinelot" (lotId);

CREATE INDEX IF NOT EXISTS "idx_stocktransferline_transferId" ON "stocktransferline" (transferId);
CREATE INDEX IF NOT EXISTS "idx_stocktransferline_variantId" ON "stocktransferline" (variantId);

CREATE INDEX IF NOT EXISTS "idx_review_productId_status_createdAt" ON "review" (productId, status, createdAt);
CREATE INDEX IF NOT EXISTS "idx_review_userId" ON "review" (userId);

ALTER TABLE "review" ADD CONSTRAINT "uniq_review_productId_userId" UNIQUE (productId, userId);

CREATE INDEX IF NOT EXISTS "idx_garmenttype_isActive_order" ON "garmenttype" (isActive, order);

CREATE INDEX IF NOT EXISTS "idx_user_role" ON "user" (role);
CREATE INDEX IF NOT EXISTS "idx_user_createdAt" ON "user" (createdAt);
CREATE INDEX IF NOT EXISTS "idx_user_ssoExternalId" ON "user" (ssoExternalId);

CREATE INDEX IF NOT EXISTS "idx_walletadjustmentrequest_accountId_status" ON "walletadjustmentrequest" (accountId, status);

CREATE INDEX IF NOT EXISTS "idx_onsitemessage_kind_isActive" ON "onsitemessage" (kind, isActive);

CREATE INDEX IF NOT EXISTS "idx_devicetoken_userId" ON "devicetoken" (userId);

CREATE INDEX IF NOT EXISTS "idx_b2baccount_isActive" ON "b2baccount" (isActive);

ALTER TABLE "stockcountline" ADD CONSTRAINT "uniq_stockcountline_stockCountId_variantId" UNIQUE (stockCountId, variantId);

CREATE INDEX IF NOT EXISTS "idx_location_isActive" ON "location" (isActive);
CREATE INDEX IF NOT EXISTS "idx_location_retailBrandId" ON "location" (retailBrandId);

CREATE INDEX IF NOT EXISTS "idx_loyaltytransaction_userId_createdAt" ON "loyaltytransaction" (userId, createdAt);

ALTER TABLE "loyaltytransaction" ADD CONSTRAINT "uniq_loyaltytransaction_orderId_type" UNIQUE (orderId, type);

CREATE INDEX IF NOT EXISTS "idx_campaignrecipient_campaignId_status" ON "campaignrecipient" (campaignId, status);

ALTER TABLE "campaignrecipient" ADD CONSTRAINT "uniq_campaignrecipient_campaignId_contactId" UNIQUE (campaignId, contactId);

CREATE INDEX IF NOT EXISTS "idx_supplierbill_supplierId_status" ON "supplierbill" (supplierId, status);
CREATE INDEX IF NOT EXISTS "idx_supplierbill_status_dueAt" ON "supplierbill" (status, dueAt);

CREATE INDEX IF NOT EXISTS "idx_bankfacilitycovenant_facilityId" ON "bankfacilitycovenant" (facilityId);

CREATE INDEX IF NOT EXISTS "idx_assetdepreciationposting_scheduleId" ON "assetdepreciationposting" (scheduleId);

ALTER TABLE "assetdepreciationposting" ADD CONSTRAINT "uniq_assetdepreciationposting_scheduleId_periodStart" UNIQUE (scheduleId, periodStart);

ALTER TABLE "blogposttranslation" ADD CONSTRAINT "uniq_blogposttranslation_blogPostId_locale" UNIQUE (blogPostId, locale);

ALTER TABLE "settlementbatchline" ADD CONSTRAINT "uniq_settlementbatchline_batchId_paymentId" UNIQUE (batchId, paymentId);

CREATE INDEX IF NOT EXISTS "idx_attendancegeofencezone_orgUnitId_isActive" ON "attendancegeofencezone" (orgUnitId, isActive);

CREATE INDEX IF NOT EXISTS "idx_supplierleadtimeobservation_supplierId_createdAt" ON "supplierleadtimeobservation" (supplierId, createdAt);

CREATE INDEX IF NOT EXISTS "idx_taxclass_isDefault" ON "taxclass" (isDefault);

CREATE INDEX IF NOT EXISTS "idx_lead_status_createdAt" ON "lead" (status, createdAt);
CREATE INDEX IF NOT EXISTS "idx_lead_ownerId" ON "lead" (ownerId);
CREATE INDEX IF NOT EXISTS "idx_lead_contactId" ON "lead" (contactId);

CREATE INDEX IF NOT EXISTS "idx_orderdiscount_orderId" ON "orderdiscount" (orderId);

CREATE INDEX IF NOT EXISTS "idx_blogpost_status_publishedAt" ON "blogpost" (status, publishedAt);
CREATE INDEX IF NOT EXISTS "idx_blogpost_category" ON "blogpost" (category);

CREATE INDEX IF NOT EXISTS "idx_shipmentevent_shipmentId_occurredAt" ON "shipmentevent" (shipmentId, occurredAt);

CREATE INDEX IF NOT EXISTS "idx_loyaltytier_minSpend" ON "loyaltytier" (minSpend);

CREATE INDEX IF NOT EXISTS "idx_orderstatusevent_orderId_createdAt" ON "orderstatusevent" (orderId, createdAt);

CREATE INDEX IF NOT EXISTS "idx_instalmentscheduleline_planId_dueDate" ON "instalmentscheduleline" (planId, dueDate);

CREATE INDEX IF NOT EXISTS "idx_entityattribute_entityType_entityId" ON "entityattribute" (entityType, entityId);

ALTER TABLE "entityattribute" ADD CONSTRAINT "uniq_entityattribute_entityType_entityId_attributeKey" UNIQUE (entityType, entityId, attributeKey);

CREATE INDEX IF NOT EXISTS "idx_posshift_terminalId" ON "posshift" (terminalId);
CREATE INDEX IF NOT EXISTS "idx_posshift_status" ON "posshift" (status);

CREATE INDEX IF NOT EXISTS "idx_searchsynonym_locale_term" ON "searchsynonym" (locale, term);

ALTER TABLE "searchsynonym" ADD CONSTRAINT "uniq_searchsynonym_locale_term_synonym" UNIQUE (locale, term, synonym);

ALTER TABLE "partyroleassignment" ADD CONSTRAINT "uniq_partyroleassignment_partyId_role" UNIQUE (partyId, role);

CREATE INDEX IF NOT EXISTS "idx_auctionlotimage_lotId_order" ON "auctionlotimage" (lotId, order);

CREATE INDEX IF NOT EXISTS "idx_offer_createdAt" ON "offer" (createdAt);

CREATE INDEX IF NOT EXISTS "idx_invoice_tradingPartnerId" ON "invoice" (tradingPartnerId);
CREATE INDEX IF NOT EXISTS "idx_invoice_status" ON "invoice" (status);
CREATE INDEX IF NOT EXISTS "idx_invoice_orderId" ON "invoice" (orderId);

CREATE INDEX IF NOT EXISTS "idx_taxrate_taxClassId_effectiveAt" ON "taxrate" (taxClassId, effectiveAt);

CREATE INDEX IF NOT EXISTS "idx_b2borderrequest_b2bAccountId_status" ON "b2borderrequest" (b2bAccountId, status);

CREATE INDEX IF NOT EXISTS "idx_supplierproduct_variantId_isPreferred" ON "supplierproduct" (variantId, isPreferred);

ALTER TABLE "supplierproduct" ADD CONSTRAINT "uniq_supplierproduct_supplierId_variantId" UNIQUE (supplierId, variantId);

CREATE INDEX IF NOT EXISTS "idx_account_type" ON "account" (type);
CREATE INDEX IF NOT EXISTS "idx_account_parentId" ON "account" (parentId);

CREATE INDEX IF NOT EXISTS "idx_idempotencykey_createdAt" ON "idempotencykey" (createdAt);

ALTER TABLE "idempotencykey" ADD CONSTRAINT "uniq_idempotencykey_scope_key" UNIQUE (scope, key);

CREATE INDEX IF NOT EXISTS "idx_productoptionvalue_optionId_order" ON "productoptionvalue" (optionId, order);

ALTER TABLE "productoptionvalue" ADD CONSTRAINT "uniq_productoptionvalue_optionId_key" UNIQUE (optionId, key);

CREATE INDEX IF NOT EXISTS "idx_settlementbatch_status" ON "settlementbatch" (status);

ALTER TABLE "settlementbatch" ADD CONSTRAINT "uniq_settlementbatch_settlementProviderId_externalBatchRef" UNIQUE (settlementProviderId, externalBatchRef);

ALTER TABLE "productoptiontranslation" ADD CONSTRAINT "uniq_productoptiontranslation_optionId_locale" UNIQUE (optionId, locale);

ALTER TABLE "bankfacilitymarkupaccrual" ADD CONSTRAINT "uniq_bankfacilitymarkupaccrual_facilityId_periodStart" UNIQUE (facilityId, periodStart);

CREATE INDEX IF NOT EXISTS "idx_analyticsevent_type_createdAt" ON "analyticsevent" (type, createdAt);
CREATE INDEX IF NOT EXISTS "idx_analyticsevent_sessionId_createdAt" ON "analyticsevent" (sessionId, createdAt);

CREATE INDEX IF NOT EXISTS "idx_measurementfield_garmentTypeId_order" ON "measurementfield" (garmentTypeId, order);

ALTER TABLE "measurementfield" ADD CONSTRAINT "uniq_measurementfield_garmentTypeId_key" UNIQUE (garmentTypeId, key);

CREATE INDEX IF NOT EXISTS "idx_bomproductionrun_producedAt" ON "bomproductionrun" (producedAt);
CREATE INDEX IF NOT EXISTS "idx_bomproductionrun_scopeLevel" ON "bomproductionrun" (scopeLevel);
CREATE INDEX IF NOT EXISTS "idx_bomproductionrun_locationId" ON "bomproductionrun" (locationId);
CREATE INDEX IF NOT EXISTS "idx_bomproductionrun_recipeId" ON "bomproductionrun" (recipeId);
CREATE INDEX IF NOT EXISTS "idx_bomproductionrun_variantId" ON "bomproductionrun" (variantId);
CREATE INDEX IF NOT EXISTS "idx_bomproductionrun_outputLotId" ON "bomproductionrun" (outputLotId);

CREATE INDEX IF NOT EXISTS "idx_optiontemplate_order" ON "optiontemplate" (order);

CREATE INDEX IF NOT EXISTS "idx_pricelistentry_variantId" ON "pricelistentry" (variantId);

ALTER TABLE "pricelistentry" ADD CONSTRAINT "uniq_pricelistentry_priceListId_variantId_minQuantity" UNIQUE (priceListId, variantId, minQuantity);

CREATE INDEX IF NOT EXISTS "idx_lotallocation_lotId" ON "lotallocation" (lotId);
CREATE INDEX IF NOT EXISTS "idx_lotallocation_stockMovementId" ON "lotallocation" (stockMovementId);

ALTER TABLE "lotallocation" ADD CONSTRAINT "uniq_lotallocation_lotId_stockMovementId" UNIQUE (lotId, stockMovementId);

CREATE INDEX IF NOT EXISTS "idx_tradingpartner_status" ON "tradingpartner" (status);

CREATE INDEX IF NOT EXISTS "idx_auctionlot_auctionId_status" ON "auctionlot" (auctionId, status);

ALTER TABLE "auctionlot" ADD CONSTRAINT "uniq_auctionlot_auctionId_lotNo" UNIQUE (auctionId, lotNo);

CREATE INDEX IF NOT EXISTS "idx_discount_isActive_startsAt_endsAt" ON "discount" (isActive, startsAt, endsAt);

CREATE INDEX IF NOT EXISTS "idx_replicationquarantine_status" ON "replicationquarantine" (status);

CREATE INDEX IF NOT EXISTS "idx_stockadjustment_variantId_createdAt" ON "stockadjustment" (variantId, createdAt);

CREATE INDEX IF NOT EXISTS "idx_customercreditterm_isOnHold" ON "customercreditterm" (isOnHold);

CREATE INDEX IF NOT EXISTS "idx_apikey_userId_createdAt" ON "apikey" (userId, createdAt);
CREATE INDEX IF NOT EXISTS "idx_apikey_createdAt" ON "apikey" (createdAt);

CREATE INDEX IF NOT EXISTS "idx_stockmovement_variantId_createdAt" ON "stockmovement" (variantId, createdAt);
CREATE INDEX IF NOT EXISTS "idx_stockmovement_reason_createdAt" ON "stockmovement" (reason, createdAt);
CREATE INDEX IF NOT EXISTS "idx_stockmovement_refType_refId" ON "stockmovement" (refType, refId);
CREATE INDEX IF NOT EXISTS "idx_stockmovement_locationId" ON "stockmovement" (locationId);

CREATE INDEX IF NOT EXISTS "idx_returnrequestitem_returnId" ON "returnrequestitem" (returnId);
CREATE INDEX IF NOT EXISTS "idx_returnrequestitem_orderItemId" ON "returnrequestitem" (orderItemId);

CREATE INDEX IF NOT EXISTS "idx_quoteline_quoteId" ON "quoteline" (quoteId);

CREATE INDEX IF NOT EXISTS "idx_stitchingjobevent_jobId_createdAt" ON "stitchingjobevent" (jobId, createdAt);

CREATE INDEX IF NOT EXISTS "idx_stitchingorder_status" ON "stitchingorder" (status);
CREATE INDEX IF NOT EXISTS "idx_stitchingorder_userId" ON "stitchingorder" (userId);
CREATE INDEX IF NOT EXISTS "idx_stitchingorder_contactId" ON "stitchingorder" (contactId);

CREATE INDEX IF NOT EXISTS "idx_payment_orderId" ON "payment" (orderId);
CREATE INDEX IF NOT EXISTS "idx_payment_status" ON "payment" (status);
CREATE INDEX IF NOT EXISTS "idx_payment_provider_status" ON "payment" (provider, status);

CREATE INDEX IF NOT EXISTS "idx_purchaseorder_status_createdAt" ON "purchaseorder" (status, createdAt);
CREATE INDEX IF NOT EXISTS "idx_purchaseorder_supplierId" ON "purchaseorder" (supplierId);
CREATE INDEX IF NOT EXISTS "idx_purchaseorder_scopeLevel" ON "purchaseorder" (scopeLevel);

CREATE INDEX IF NOT EXISTS "idx_assetassignment_assetId" ON "assetassignment" (assetId);
CREATE INDEX IF NOT EXISTS "idx_assetassignment_userId_returnedAt" ON "assetassignment" (userId, returnedAt);

CREATE INDEX IF NOT EXISTS "idx_bankstatementline_bankId_date" ON "bankstatementline" (bankId, date);

CREATE INDEX IF NOT EXISTS "idx_rolepermission_role" ON "rolepermission" (role);

ALTER TABLE "rolepermission" ADD CONSTRAINT "uniq_rolepermission_role_permissionId" UNIQUE (role, permissionId);

CREATE INDEX IF NOT EXISTS "idx_serializedunit_productVariantId_status" ON "serializedunit" (productVariantId, status);
CREATE INDEX IF NOT EXISTS "idx_serializedunit_orderId" ON "serializedunit" (orderId);
CREATE INDEX IF NOT EXISTS "idx_serializedunit_stockMovementId" ON "serializedunit" (stockMovementId);

CREATE INDEX IF NOT EXISTS "idx_attendanceevent_userId_occurredAt" ON "attendanceevent" (userId, occurredAt);
CREATE INDEX IF NOT EXISTS "idx_attendanceevent_orgUnitId_occurredAt" ON "attendanceevent" (orgUnitId, occurredAt);
CREATE INDEX IF NOT EXISTS "idx_attendanceevent_id" ON "attendanceevent" (id);

ALTER TABLE "attendanceevent" ADD CONSTRAINT "uniq_attendanceevent_userId_source_externalRef_occurredAt" UNIQUE (userId, source, externalRef, occurredAt);

CREATE INDEX IF NOT EXISTS "idx_auctionsettlementpayment_settlementId" ON "auctionsettlementpayment" (settlementId);

CREATE INDEX IF NOT EXISTS "idx_category_parentId" ON "category" (parentId);
CREATE INDEX IF NOT EXISTS "idx_category_isActive" ON "category" (isActive);

ALTER TABLE "productimagetranslation" ADD CONSTRAINT "uniq_productimagetranslation_imageId_locale" UNIQUE (imageId, locale);

CREATE INDEX IF NOT EXISTS "idx_orgunit_parentId" ON "orgunit" (parentId);
CREATE INDEX IF NOT EXISTS "idx_orgunit_path" ON "orgunit" (path);
CREATE INDEX IF NOT EXISTS "idx_orgunit_nodeId" ON "orgunit" (nodeId);

ALTER TABLE "orgunit" ADD CONSTRAINT "uniq_orgunit_retailBrandId_code" UNIQUE (retailBrandId, code);

ALTER TABLE "categorytranslation" ADD CONSTRAINT "uniq_categorytranslation_categoryId_locale" UNIQUE (categoryId, locale);

CREATE INDEX IF NOT EXISTS "idx_brand_isActive" ON "brand" (isActive);

CREATE INDEX IF NOT EXISTS "idx_stitchingjob_stage_dueAt" ON "stitchingjob" (stage, dueAt);
CREATE INDEX IF NOT EXISTS "idx_stitchingjob_assignedTailorId_stage" ON "stitchingjob" (assignedTailorId, stage);
CREATE INDEX IF NOT EXISTS "idx_stitchingjob_stitchingOrderId" ON "stitchingjob" (stitchingOrderId);

CREATE INDEX IF NOT EXISTS "idx_bidderstrike_userId_recordedAt" ON "bidderstrike" (userId, recordedAt);
CREATE INDEX IF NOT EXISTS "idx_bidderstrike_bidderId" ON "bidderstrike" (bidderId);

ALTER TABLE "pagetranslation" ADD CONSTRAINT "uniq_pagetranslation_pageId_locale" UNIQUE (pageId, locale);

CREATE INDEX IF NOT EXISTS "idx_page_status" ON "page" (status);
CREATE INDEX IF NOT EXISTS "idx_page_publishedAt" ON "page" (publishedAt);

CREATE INDEX IF NOT EXISTS "idx_followuptask_isDone_dueAt" ON "followuptask" (isDone, dueAt);
CREATE INDEX IF NOT EXISTS "idx_followuptask_assigneeId_isDone" ON "followuptask" (assigneeId, isDone);

CREATE INDEX IF NOT EXISTS "idx_payrolltaxbracket_isActive_floorMinor" ON "payrolltaxbracket" (isActive, floorMinor);
CREATE INDEX IF NOT EXISTS "idx_payrolltaxbracket_version" ON "payrolltaxbracket" (version);

ALTER TABLE "lifecyclechecklisttemplatetask" ADD CONSTRAINT "uniq_lifecyclechecklisttemplatetask_type_taskKey" UNIQUE (type, taskKey);

CREATE INDEX IF NOT EXISTS "idx_purchaserequisition_status_createdAt" ON "purchaserequisition" (status, createdAt);
CREATE INDEX IF NOT EXISTS "idx_purchaserequisition_requestedById_createdAt" ON "purchaserequisition" (requestedById, createdAt);
CREATE INDEX IF NOT EXISTS "idx_purchaserequisition_locationId" ON "purchaserequisition" (locationId);

CREATE INDEX IF NOT EXISTS "idx_documentchunk_organizationId_corpus_sourceId" ON "documentchunk" (organizationId, corpus, sourceId);
CREATE INDEX IF NOT EXISTS "idx_documentchunk_sourceId" ON "documentchunk" (sourceId);

CREATE INDEX IF NOT EXISTS "idx_collection_isActive" ON "collection" (isActive);

CREATE INDEX IF NOT EXISTS "idx_merchantpayable_organizationId_status" ON "merchantpayable" (organizationId, status);

ALTER TABLE "shippingzonetranslation" ADD CONSTRAINT "uniq_shippingzonetranslation_zoneId_locale" UNIQUE (zoneId, locale);

CREATE INDEX IF NOT EXISTS "idx_shipmentitem_shipmentId" ON "shipmentitem" (shipmentId);
CREATE INDEX IF NOT EXISTS "idx_shipmentitem_orderItemId" ON "shipmentitem" (orderItemId);

CREATE INDEX IF NOT EXISTS "idx_producttranslation_name" ON "producttranslation" (name);

ALTER TABLE "producttranslation" ADD CONSTRAINT "uniq_producttranslation_productId_locale" UNIQUE (productId, locale);

ALTER TABLE "shippingclasstranslation" ADD CONSTRAINT "uniq_shippingclasstranslation_classId_locale" UNIQUE (classId, locale);

CREATE INDEX IF NOT EXISTS "idx_newslettersubscriber_optedOutAt" ON "newslettersubscriber" (optedOutAt);

CREATE INDEX IF NOT EXISTS "idx_cashmovement_shiftId" ON "cashmovement" (shiftId);

CREATE INDEX IF NOT EXISTS "idx_posterminal_locationId" ON "posterminal" (locationId);
CREATE INDEX IF NOT EXISTS "idx_posterminal_status" ON "posterminal" (status);

CREATE INDEX IF NOT EXISTS "idx_supplier_isActive_name" ON "supplier" (isActive, name);
CREATE INDEX IF NOT EXISTS "idx_supplier_tradingPartnerId" ON "supplier" (tradingPartnerId);

CREATE INDEX IF NOT EXISTS "idx_leaverequest_userId_status" ON "leaverequest" (userId, status);

ALTER TABLE "collectiontranslation" ADD CONSTRAINT "uniq_collectiontranslation_collectionId_locale" UNIQUE (collectionId, locale);

CREATE INDEX IF NOT EXISTS "idx_rolegrant_roleId" ON "rolegrant" (roleId);

ALTER TABLE "rolegrant" ADD CONSTRAINT "uniq_rolegrant_roleId_permissionId" UNIQUE (roleId, permissionId);

CREATE INDEX IF NOT EXISTS "idx_billofmaterialline_componentVariantId" ON "billofmaterialline" (componentVariantId);

ALTER TABLE "billofmaterialline" ADD CONSTRAINT "uniq_billofmaterialline_billOfMaterialId_componentVariantId" UNIQUE (billOfMaterialId, componentVariantId);

CREATE INDEX IF NOT EXISTS "idx_meeting_orgUnitId_startsAt" ON "meeting" (orgUnitId, startsAt);

CREATE INDEX IF NOT EXISTS "idx_position_orgUnitId" ON "position" (orgUnitId);
CREATE INDEX IF NOT EXISTS "idx_position_reportsToId" ON "position" (reportsToId);

CREATE INDEX IF NOT EXISTS "idx_aiusagelog_feature_createdAt" ON "aiusagelog" (feature, createdAt);
CREATE INDEX IF NOT EXISTS "idx_aiusagelog_createdAt" ON "aiusagelog" (createdAt);

CREATE INDEX IF NOT EXISTS "idx_bomcomponent_rawMaterialId" ON "bomcomponent" (rawMaterialId);

ALTER TABLE "bomcomponent" ADD CONSTRAINT "uniq_bomcomponent_recipeId_rawMaterialId" UNIQUE (recipeId, rawMaterialId);

CREATE INDEX IF NOT EXISTS "idx_approvalstep_approverUserId_status" ON "approvalstep" (approverUserId, status);

ALTER TABLE "approvalstep" ADD CONSTRAINT "uniq_approvalstep_chainId_stepOrder" UNIQUE (chainId, stepOrder);

CREATE INDEX IF NOT EXISTS "idx_node_role" ON "node" (role);
CREATE INDEX IF NOT EXISTS "idx_node_status" ON "node" (status);
CREATE INDEX IF NOT EXISTS "idx_node_retailBrandId" ON "node" (retailBrandId);

CREATE INDEX IF NOT EXISTS "idx_promocode_discountId" ON "promocode" (discountId);
CREATE INDEX IF NOT EXISTS "idx_promocode_batchId" ON "promocode" (batchId);

CREATE INDEX IF NOT EXISTS "idx_campaign_status_scheduledAt" ON "campaign" (status, scheduledAt);

CREATE INDEX IF NOT EXISTS "idx_userroleassignment_userId" ON "userroleassignment" (userId);
CREATE INDEX IF NOT EXISTS "idx_userroleassignment_roleId" ON "userroleassignment" (roleId);

ALTER TABLE "userroleassignment" ADD CONSTRAINT "uniq_userroleassignment_userId_roleId" UNIQUE (userId, roleId);

CREATE INDEX IF NOT EXISTS "idx_wishlistitem_userId_createdAt" ON "wishlistitem" (userId, createdAt);
CREATE INDEX IF NOT EXISTS "idx_wishlistitem_productId" ON "wishlistitem" (productId);

ALTER TABLE "wishlistitem" ADD CONSTRAINT "uniq_wishlistitem_userId_productId_variantId" UNIQUE (userId, productId, variantId);

CREATE INDEX IF NOT EXISTS "idx_lot_expiresOn" ON "lot" (expiresOn);

ALTER TABLE "lot" ADD CONSTRAINT "uniq_lot_productVariantId_lotCode" UNIQUE (productVariantId, lotCode);

CREATE INDEX IF NOT EXISTS "idx_approvalchain_subjectType_status" ON "approvalchain" (subjectType, status);

ALTER TABLE "approvalchain" ADD CONSTRAINT "uniq_approvalchain_subjectType_subjectId" UNIQUE (subjectType, subjectId);

CREATE INDEX IF NOT EXISTS "idx_replicationrecord_entity_entityId" ON "replicationrecord" (entity, entityId);

ALTER TABLE "replicationrecord" ADD CONSTRAINT "uniq_replicationrecord_originNodeId_sequence" UNIQUE (originNodeId, sequence);

CREATE INDEX IF NOT EXISTS "idx_requestoutcome_path_occurredAt" ON "requestoutcome" (path, occurredAt);

CREATE INDEX IF NOT EXISTS "idx_requisitionline_requisitionId" ON "requisitionline" (requisitionId);
CREATE INDEX IF NOT EXISTS "idx_requisitionline_variantId" ON "requisitionline" (variantId);

CREATE INDEX IF NOT EXISTS "idx_collectionproduct_productId" ON "collectionproduct" (productId);

CREATE INDEX IF NOT EXISTS "idx_product_status_kind" ON "product" (status, kind);
CREATE INDEX IF NOT EXISTS "idx_product_categoryId" ON "product" (categoryId);
CREATE INDEX IF NOT EXISTS "idx_product_featuredRank" ON "product" (featuredRank);
CREATE INDEX IF NOT EXISTS "idx_product_createdAt" ON "product" (createdAt);

CREATE INDEX IF NOT EXISTS "idx_cart_userId" ON "cart" (userId);
CREATE INDEX IF NOT EXISTS "idx_cart_expiresAt" ON "cart" (expiresAt);
CREATE INDEX IF NOT EXISTS "idx_cart_guestPhone" ON "cart" (guestPhone);

CREATE INDEX IF NOT EXISTS "idx_stitchingoptiongroup_garmentTypeId_order" ON "stitchingoptiongroup" (garmentTypeId, order);
CREATE INDEX IF NOT EXISTS "idx_stitchingoptiongroup_area_order" ON "stitchingoptiongroup" (area, order);

CREATE INDEX IF NOT EXISTS "idx_shipment_orderId" ON "shipment" (orderId);
CREATE INDEX IF NOT EXISTS "idx_shipment_trackingNumber" ON "shipment" (trackingNumber);
CREATE INDEX IF NOT EXISTS "idx_shipment_status" ON "shipment" (status);
CREATE INDEX IF NOT EXISTS "idx_shipment_manifestId" ON "shipment" (manifestId);

CREATE INDEX IF NOT EXISTS "idx_giftcardtransaction_giftCardId_createdAt" ON "giftcardtransaction" (giftCardId, createdAt);
CREATE INDEX IF NOT EXISTS "idx_giftcardtransaction_orderId" ON "giftcardtransaction" (orderId);

CREATE INDEX IF NOT EXISTS "idx_partnerfactinbox_tradingPartnerId_receivedAt" ON "partnerfactinbox" (tradingPartnerId, receivedAt);

ALTER TABLE "partnerfactinbox" ADD CONSTRAINT "uniq_partnerfactinbox_tradingPartnerId_factId" UNIQUE (tradingPartnerId, factId);

CREATE INDEX IF NOT EXISTS "idx_backinstockrequest_variantId" ON "backinstockrequest" (variantId);

ALTER TABLE "backinstockrequest" ADD CONSTRAINT "uniq_backinstockrequest_userId_variantId" UNIQUE (userId, variantId);

CREATE INDEX IF NOT EXISTS "idx_stockcount_status" ON "stockcount" (status);

CREATE INDEX IF NOT EXISTS "idx_bomproductionline_runId" ON "bomproductionline" (runId);
CREATE INDEX IF NOT EXISTS "idx_bomproductionline_rawMaterialId" ON "bomproductionline" (rawMaterialId);

CREATE INDEX IF NOT EXISTS "idx_productoption_productId_order" ON "productoption" (productId, order);

ALTER TABLE "productoption" ADD CONSTRAINT "uniq_productoption_productId_key" UNIQUE (productId, key);

CREATE INDEX IF NOT EXISTS "idx_bankfacility_status" ON "bankfacility" (status);
CREATE INDEX IF NOT EXISTS "idx_bankfacility_maturityDate" ON "bankfacility" (maturityDate);

CREATE INDEX IF NOT EXISTS "idx_asset_status" ON "asset" (status);

CREATE INDEX IF NOT EXISTS "idx_bomrecipestage_workCenterId" ON "bomrecipestage" (workCenterId);

ALTER TABLE "bomrecipestage" ADD CONSTRAINT "uniq_bomrecipestage_recipeId_stepNumber" UNIQUE (recipeId, stepNumber);

CREATE INDEX IF NOT EXISTS "idx_party_phone" ON "party" (phone);
CREATE INDEX IF NOT EXISTS "idx_party_taxId" ON "party" (taxId);

CREATE INDEX IF NOT EXISTS "idx_orgunitgrant_userId_expiresAt" ON "orgunitgrant" (userId, expiresAt);
CREATE INDEX IF NOT EXISTS "idx_orgunitgrant_orgUnitId" ON "orgunitgrant" (orgUnitId);

CREATE INDEX IF NOT EXISTS "idx_stockvariance_status" ON "stockvariance" (status);
CREATE INDEX IF NOT EXISTS "idx_stockvariance_variantId" ON "stockvariance" (variantId);
CREATE INDEX IF NOT EXISTS "idx_stockvariance_locationId" ON "stockvariance" (locationId);
CREATE INDEX IF NOT EXISTS "idx_stockvariance_posShiftId" ON "stockvariance" (posShiftId);
CREATE INDEX IF NOT EXISTS "idx_stockvariance_serializedUnitId" ON "stockvariance" (serializedUnitId);

CREATE INDEX IF NOT EXISTS "idx_dottedlinereport_dottedManagerUserId" ON "dottedlinereport" (dottedManagerUserId);

ALTER TABLE "dottedlinereport" ADD CONSTRAINT "uniq_dottedlinereport_userId_dottedManagerUserId" UNIQUE (userId, dottedManagerUserId);

CREATE INDEX IF NOT EXISTS "idx_posting_userId_endedOn" ON "posting" (userId, endedOn);
CREATE INDEX IF NOT EXISTS "idx_posting_positionId_endedOn" ON "posting" (positionId, endedOn);

ALTER TABLE "payrunline" ADD CONSTRAINT "uniq_payrunline_payRunId_userId" UNIQUE (payRunId, userId);

CREATE INDEX IF NOT EXISTS "idx_stocktransfer_status" ON "stocktransfer" (status);
CREATE INDEX IF NOT EXISTS "idx_stocktransfer_sourceLocationId" ON "stocktransfer" (sourceLocationId);
CREATE INDEX IF NOT EXISTS "idx_stocktransfer_destinationLocationId" ON "stocktransfer" (destinationLocationId);

CREATE INDEX IF NOT EXISTS "idx_debitnote_supplierId_status" ON "debitnote" (supplierId, status);

ALTER TABLE "spotcheckline" ADD CONSTRAINT "uniq_spotcheckline_spotCheckId_variantId" UNIQUE (spotCheckId, variantId);

CREATE INDEX IF NOT EXISTS "idx_notificationpreference_userId" ON "notificationpreference" (userId);

ALTER TABLE "notificationpreference" ADD CONSTRAINT "uniq_notificationpreference_userId_channel_category" UNIQUE (userId, channel, category);

CREATE INDEX IF NOT EXISTS "idx_banner_placementKey_isActive_order" ON "banner" (placementKey, isActive, order);

ALTER TABLE "shippingmethodtranslation" ADD CONSTRAINT "uniq_shippingmethodtranslation_methodId_locale" UNIQUE (methodId, locale);

CREATE INDEX IF NOT EXISTS "idx_markdownsuggestion_variantId_status" ON "markdownsuggestion" (variantId, status);
CREATE INDEX IF NOT EXISTS "idx_markdownsuggestion_status_createdAt" ON "markdownsuggestion" (status, createdAt);

CREATE INDEX IF NOT EXISTS "idx_garmentsizechart_garmentTypeId_order" ON "garmentsizechart" (garmentTypeId, order);

ALTER TABLE "garmentsizechart" ADD CONSTRAINT "uniq_garmentsizechart_garmentTypeId_sizeToken" UNIQUE (garmentTypeId, sizeToken);

ALTER TABLE "bulletinacknowledgement" ADD CONSTRAINT "uniq_bulletinacknowledgement_bulletinId_userId" UNIQUE (bulletinId, userId);

CREATE INDEX IF NOT EXISTS "idx_goodsreceiptitem_goodsReceiptId" ON "goodsreceiptitem" (goodsReceiptId);
CREATE INDEX IF NOT EXISTS "idx_goodsreceiptitem_variantId" ON "goodsreceiptitem" (variantId);

CREATE INDEX IF NOT EXISTS "idx_creditnote_customerId_status" ON "creditnote" (customerId, status);
CREATE INDEX IF NOT EXISTS "idx_creditnote_orderId" ON "creditnote" (orderId);

CREATE INDEX IF NOT EXISTS "idx_ssoconnection_status" ON "ssoconnection" (status);

CREATE INDEX IF NOT EXISTS "idx_bulletin_audienceOrgUnitId_publishedAt" ON "bulletin" (audienceOrgUnitId, publishedAt);

ALTER TABLE "brandtranslation" ADD CONSTRAINT "uniq_brandtranslation_brandId_locale" UNIQUE (brandId, locale);

CREATE INDEX IF NOT EXISTS "idx_customersupportmessage_ticketId_createdAt" ON "customersupportmessage" (ticketId, createdAt);

CREATE INDEX IF NOT EXISTS "idx_storecredittransaction_userId_createdAt" ON "storecredittransaction" (userId, createdAt);
CREATE INDEX IF NOT EXISTS "idx_storecredittransaction_orderId" ON "storecredittransaction" (orderId);

ALTER TABLE "leaveaccrualrun" ADD CONSTRAINT "uniq_leaveaccrualrun_leaveTypeId_period" UNIQUE (leaveTypeId, period);

CREATE INDEX IF NOT EXISTS "idx_escalationfire_taskId" ON "escalationfire" (taskId);

ALTER TABLE "escalationfire" ADD CONSTRAINT "uniq_escalationfire_taskId_fireNumber" UNIQUE (taskId, fireNumber);

CREATE INDEX IF NOT EXISTS "idx_posreceipt_orderId" ON "posreceipt" (orderId);

CREATE INDEX IF NOT EXISTS "idx_spotcheckpolicy_locationId_isActive" ON "spotcheckpolicy" (locationId, isActive);

CREATE INDEX IF NOT EXISTS "idx_measurementprofile_userId" ON "measurementprofile" (userId);

ALTER TABLE "measurementprofile" ADD CONSTRAINT "uniq_measurementprofile_userId_garmentTypeId_label" UNIQUE (userId, garmentTypeId, label);

CREATE INDEX IF NOT EXISTS "idx_taskdependency_successorTaskId" ON "taskdependency" (successorTaskId);

ALTER TABLE "taskdependency" ADD CONSTRAINT "uniq_taskdependency_predecessorTaskId_successorTaskId" UNIQUE (predecessorTaskId, successorTaskId);

CREATE INDEX IF NOT EXISTS "idx_cartitem_cartId" ON "cartitem" (cartId);

CREATE INDEX IF NOT EXISTS "idx_fiscalyear_startsOn" ON "fiscalyear" (startsOn);

CREATE INDEX IF NOT EXISTS "idx_dunningevent_orderId" ON "dunningevent" (orderId);

ALTER TABLE "dunningevent" ADD CONSTRAINT "uniq_dunningevent_orderId_stepIndex" UNIQUE (orderId, stepIndex);

CREATE INDEX IF NOT EXISTS "idx_journalline_entryId" ON "journalline" (entryId);
CREATE INDEX IF NOT EXISTS "idx_journalline_accountId" ON "journalline" (accountId);
CREATE INDEX IF NOT EXISTS "idx_journalline_retailBrandId" ON "journalline" (retailBrandId);
CREATE INDEX IF NOT EXISTS "idx_journalline_branchNodeId" ON "journalline" (branchNodeId);
CREATE INDEX IF NOT EXISTS "idx_journalline_counterpartyBrandId" ON "journalline" (counterpartyBrandId);

ALTER TABLE "lifecyclechecklisttask" ADD CONSTRAINT "uniq_lifecyclechecklisttask_checklistId_taskKey" UNIQUE (checklistId, taskKey);

ALTER TABLE "leavebalance" ADD CONSTRAINT "uniq_leavebalance_userId_leaveTypeId" UNIQUE (userId, leaveTypeId);

CREATE INDEX IF NOT EXISTS "idx_attendanceexception_resolvedAt" ON "attendanceexception" (resolvedAt);
CREATE INDEX IF NOT EXISTS "idx_attendanceexception_userId" ON "attendanceexception" (userId);
CREATE INDEX IF NOT EXISTS "idx_attendanceexception_orgUnitId" ON "attendanceexception" (orgUnitId);

CREATE INDEX IF NOT EXISTS "idx_expense_incurredAt" ON "expense" (incurredAt);
CREATE INDEX IF NOT EXISTS "idx_expense_categoryId_incurredAt" ON "expense" (categoryId, incurredAt);
CREATE INDEX IF NOT EXISTS "idx_expense_paidAt" ON "expense" (paidAt);
CREATE INDEX IF NOT EXISTS "idx_expense_supplierId" ON "expense" (supplierId);
CREATE INDEX IF NOT EXISTS "idx_expense_scopeLevel" ON "expense" (scopeLevel);

ALTER TABLE "expense" ADD CONSTRAINT "uniq_expense_recurringFromId_recurringPeriod" UNIQUE (recurringFromId, recurringPeriod);

CREATE INDEX IF NOT EXISTS "idx_walletaccount_organizationId" ON "walletaccount" (organizationId);

CREATE INDEX IF NOT EXISTS "idx_lotstock_locationId" ON "lotstock" (locationId);

ALTER TABLE "lotstock" ADD CONSTRAINT "uniq_lotstock_lotId_locationId" UNIQUE (lotId, locationId);

CREATE INDEX IF NOT EXISTS "idx_spotcheck_locationId_status" ON "spotcheck" (locationId, status);
CREATE INDEX IF NOT EXISTS "idx_spotcheck_policyId_scheduledFor" ON "spotcheck" (policyId, scheduledFor);

CREATE INDEX IF NOT EXISTS "idx_task_positionId_state" ON "task" (positionId, state);
CREATE INDEX IF NOT EXISTS "idx_task_assigneeUserId_state" ON "task" (assigneeUserId, state);
CREATE INDEX IF NOT EXISTS "idx_task_orgUnitId_state" ON "task" (orgUnitId, state);
CREATE INDEX IF NOT EXISTS "idx_task_dueAt" ON "task" (dueAt);

ALTER TABLE "task" ADD CONSTRAINT "uniq_task_sourceType_sourceRef" UNIQUE (sourceType, sourceRef);

CREATE INDEX IF NOT EXISTS "idx_rawmaterial_isActive_kind" ON "rawmaterial" (isActive, kind);
CREATE INDEX IF NOT EXISTS "idx_rawmaterial_name" ON "rawmaterial" (name);

CREATE INDEX IF NOT EXISTS "idx_referral_referrerId" ON "referral" (referrerId);

ALTER TABLE "referral" ADD CONSTRAINT "uniq_referral_refereeId" UNIQUE (refereeId);

ALTER TABLE "menuitemtranslation" ADD CONSTRAINT "uniq_menuitemtranslation_menuItemId_locale" UNIQUE (menuItemId, locale);

CREATE INDEX IF NOT EXISTS "idx_fiscalperiod_status" ON "fiscalperiod" (status);
CREATE INDEX IF NOT EXISTS "idx_fiscalperiod_startsOn_endsOn" ON "fiscalperiod" (startsOn, endsOn);

ALTER TABLE "fiscalperiod" ADD CONSTRAINT "uniq_fiscalperiod_fiscalYearId_name" UNIQUE (fiscalYearId, name);

CREATE INDEX IF NOT EXISTS "idx_payrun_status_periodStart" ON "payrun" (status, periodStart);

CREATE INDEX IF NOT EXISTS "idx_journalentry_periodId" ON "journalentry" (periodId);
CREATE INDEX IF NOT EXISTS "idx_journalentry_source_sourceRef" ON "journalentry" (source, sourceRef);
CREATE INDEX IF NOT EXISTS "idx_journalentry_status" ON "journalentry" (status);

CREATE INDEX IF NOT EXISTS "idx_menuitem_menuKey_parentId_order" ON "menuitem" (menuKey, parentId, order);

CREATE INDEX IF NOT EXISTS "idx_returnrequest_status_requestedAt" ON "returnrequest" (status, requestedAt);
CREATE INDEX IF NOT EXISTS "idx_returnrequest_orderId" ON "returnrequest" (orderId);

CREATE INDEX IF NOT EXISTS "idx_supportticket_status_assignedToUserId" ON "supportticket" (status, assignedToUserId);
CREATE INDEX IF NOT EXISTS "idx_supportticket_raisedByUserId" ON "supportticket" (raisedByUserId);

CREATE INDEX IF NOT EXISTS "idx_refreshtoken_userId" ON "refreshtoken" (userId);
CREATE INDEX IF NOT EXISTS "idx_refreshtoken_expiresAt" ON "refreshtoken" (expiresAt);
CREATE INDEX IF NOT EXISTS "idx_refreshtoken_familyId" ON "refreshtoken" (familyId);

CREATE INDEX IF NOT EXISTS "idx_giftcard_status" ON "giftcard" (status);
CREATE INDEX IF NOT EXISTS "idx_giftcard_purchasedByUserId" ON "giftcard" (purchasedByUserId);

CREATE INDEX IF NOT EXISTS "idx_featureflag_createdAt" ON "featureflag" (createdAt);

CREATE INDEX IF NOT EXISTS "idx_bomrecipe_status" ON "bomrecipe" (status);
CREATE INDEX IF NOT EXISTS "idx_bomrecipe_costComplete" ON "bomrecipe" (costComplete);

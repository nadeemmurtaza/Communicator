-- Migration 0009: Add index on brands.industry_id and seed additional industries
-- Note: industry_id column already added by migration 0007, do not duplicate ALTER TABLE

CREATE INDEX IF NOT EXISTS idx_brands_industry_id ON brands(industry_id);

-- Seed additional industries not already present in migration 0007
-- Migration 0007 seeds: ind_healthcare, ind_ecommerce, ind_education, ind_finance,
--   ind_realestate, ind_restaurant, ind_retail, ind_technology, ind_travel,
--   ind_fitness, ind_legal, ind_automotive, ind_general
-- These are the new/unique industries:
INSERT INTO industries (id, name, description, icon) VALUES
    ('ind_food', 'Food & Beverage', 'Restaurants, cafes, and food delivery', '🍔'),
    ('ind_entertainment', 'Entertainment', 'Media, gaming, and entertainment', '🎬'),
    ('ind_government', 'Government', 'Public sector and government services', '🏛️'),
    ('ind_nonprofit', 'Non-Profit', 'NGOs and charitable organizations', '❤️'),
    ('ind_logistics', 'Logistics', 'Shipping, delivery, and supply chain', '📦'),
    ('ind_other', 'Other', 'Other industries', '📋');

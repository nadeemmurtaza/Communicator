#![allow(dead_code)]

//! Channel commerce templates: render catalog, cart, and order flows for each channel.

use serde::{Deserialize, Serialize};

use crate::models::{Product, CommerceOrder};
use crate::services::commerce_cart::CartItemWithProduct;
use crate::services::commerce_options::ProductOptionWithValues;

// ---------------------------------------------------------------------------
// Template Renderer
// ---------------------------------------------------------------------------

pub struct ChannelCommerceTemplates;

impl ChannelCommerceTemplates {
    // ---------------------------------------------------------------------------
    // WhatsApp Templates
    // ---------------------------------------------------------------------------

    /// Render a product card for WhatsApp catalog messages
    pub fn whatsapp_product_card(product: &Product) -> String {
        Self::whatsapp_product_card_with_options(product, &[])
    }

    /// Render a product card for WhatsApp with options/add-ons
    pub fn whatsapp_product_card_with_options(
        product: &Product,
        options: &[ProductOptionWithValues],
    ) -> String {
        let price = if product.compare_at_price.is_some() {
            format!(
                "~${:.2}~ ${:.2}",
                product.compare_at_price.unwrap(),
                product.price
            )
        } else {
            format!("${:.2}", product.price)
        };

        let stock = if product.track_inventory {
            if product.inventory_count > 0 {
                format!("✅ In stock ({})", product.inventory_count)
            } else {
                "❌ Out of stock".to_string()
            }
        } else {
            "✅ Available".to_string()
        };

        let mut card = format!(
            "🛍️ *{}*\n{}\n\n💰 {}\n{}",
            product.name,
            product.description.as_deref().unwrap_or("No description"),
            price,
            stock
        );

        // Append options section if product has add-ons
        if !options.is_empty() {
            card.push_str("\n\n🎛️ *Customize your order:*");
            for opt in options {
                let req_tag = if opt.option.required { " (required)" } else { "" };
                card.push_str(&format!("\n\n*{}{}*:", opt.option.name, req_tag));
                if !opt.values.is_empty() {
                    for val in &opt.values {
                        let price_str = if val.price_adjustment != 0.0 {
                            format!("+${:.2}", val.price_adjustment)
                        } else {
                            "".to_string()
                        };
                        card.push_str(&format!("\n • {} {}", val.name, price_str));
                    }
                } else {
                    let price_str = if opt.option.price_adjustment != 0.0 {
                        format!("+${:.2}", opt.option.price_adjustment)
                    } else {
                        "".to_string()
                    };
                    card.push_str(&format!("\n • Add {} {}", opt.option.name, price_str));
                }
            }
            card.push_str("\n\n👆 Reply with the options you want, or 'ADD' to add to cart.");
        }

        card
    }

    /// Generate WhatsApp interactive list message for product options
    pub fn whatsapp_product_options_list(
        product: &Product,
        options: &[ProductOptionWithValues],
    ) -> serde_json::Value {
        // Build list items from all option values
        let mut items = Vec::new();
        for opt in options {
            if !opt.values.is_empty() {
                for val in &opt.values {
                    let price_str = if val.price_adjustment != 0.0 {
                        format!("+${:.2}", val.price_adjustment)
                    } else {
                        "".to_string()
                    };
                    items.push(serde_json::json!({
                        "id": val.id.clone(),
                        "title": format!("{} {}", val.name, price_str),
                        "description": format!("{} — {}", opt.option.name, product.name),
                    }));
                }
            } else {
                let price_str = if opt.option.price_adjustment != 0.0 {
                    format!("+${:.2}", opt.option.price_adjustment)
                } else {
                    "".to_string()
                };
                items.push(serde_json::json!({
                    "id": opt.option.id.clone(),
                    "title": format!("{} {}", opt.option.name, price_str),
                    "description": product.name.clone(),
                }));
            }
        }

        serde_json::json!({
            "type": "list",
            "header": { "type": "text", "text": format!("🛍️ {}", product.name) },
            "body": {
                "text": format!(
                    "{}\n\n💰 ${:.2}\n\nChoose your options below:",
                    product.description.as_deref().unwrap_or(""),
                    product.price
                )
            },
            "footer": { "text": "Tap an option to add it to your order" },
            "action": {
                "button": "Choose options",
                "sections": [{
                    "title": "Available options",
                    "rows": items,
                }]
            }
        })
    }

    /// Render a cart summary for WhatsApp
    pub fn whatsapp_cart_summary(
        items: &[CartItemWithProduct],
        total: f64,
        currency: &str,
    ) -> String {
        let lines: Vec<String> = items
            .iter()
            .map(|item| {
                let name = item.product_name.as_deref().unwrap_or("Product");
                format!("• {} x{} — {:.2} {}", name, item.quantity, item.unit_price, currency)
            })
            .collect();

        format!(
            "🛒 *Your Cart*\n\n{}\n\n*Total: {:.2} {}*\n\nReply with 'CHECKOUT' to place your order.",
            lines.join("\n"),
            total,
            currency
        )
    }

    /// Render an order confirmation for WhatsApp
    pub fn whatsapp_order_confirmation(order: &CommerceOrder) -> String {
        let address = order
            .shipping_address
            .as_ref()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
            .and_then(|v| v.get("address").and_then(|a| a.as_str()).map(|s| s.to_string()))
            .unwrap_or_else(|| "Not provided".to_string());

        format!(
            "✅ *Order Confirmed!*\n\nOrder: {}\nTotal: {:.2} {}\nStatus: {}\n\n📦 Shipping to:\n{}\n\nThank you for your order!",
            order.order_number,
            order.total_amount,
            order.currency,
            order.status,
            address
        )
    }

    // ---------------------------------------------------------------------------
    // Instagram Templates
    // ---------------------------------------------------------------------------

    /// Render a product carousel card for Instagram
    pub fn instagram_product_card(product: &Product) -> String {
        Self::instagram_product_card_with_options(product, &[])
    }

    /// Render a product carousel card for Instagram with options
    pub fn instagram_product_card_with_options(
        product: &Product,
        options: &[ProductOptionWithValues],
    ) -> String {
        let emoji = if product.price > 100.0 { "✨" } else { "🛍️" };
        let mut card = format!(
            "{} {} — ${:.2}\n{}",
            emoji,
            product.name,
            product.price,
            product.description.as_deref().unwrap_or("")
        );

        if !options.is_empty() {
            card.push_str("\n\n✨ Customize:");
            for opt in options {
                let req_tag = if opt.option.required { " *" } else { "" };
                card.push_str(&format!("\n🔹{}{}", opt.option.name, req_tag));
                for val in &opt.values {
                    let price_str = if val.price_adjustment != 0.0 {
                        format!(" (+${:.2})", val.price_adjustment)
                    } else {
                        "".to_string()
                    };
                    card.push_str(&format!("\n   •{}{}", val.name, price_str));
                }
            }
            card.push_str("\n\n💬 Reply with your choices to order!");
        }

        card
    }

    /// Generate Instagram reply buttons for product options
    pub fn instagram_product_options_buttons(
        _product: &Product,
        options: &[ProductOptionWithValues],
    ) -> Vec<serde_json::Value> {
        let mut buttons = Vec::new();
        for opt in options {
            if !opt.values.is_empty() {
                for val in &opt.values {
                    let price_str = if val.price_adjustment != 0.0 {
                        format!(" +${:.2}", val.price_adjustment)
                    } else {
                        "".to_string()
                    };
                    buttons.push(serde_json::json!({
                        "type": "reply",
                        "reply": {
                            "id": val.id.clone(),
                            "title": format!("{}{}", val.name, price_str),
                            "description": opt.option.name.clone(),
                        }
                    }));
                }
            } else {
                let price_str = if opt.option.price_adjustment != 0.0 {
                    format!(" +${:.2}", opt.option.price_adjustment)
                } else {
                    "".to_string()
                };
                buttons.push(serde_json::json!({
                    "type": "reply",
                    "reply": {
                        "id": opt.option.id.clone(),
                        "title": format!("{}{}", opt.option.name, price_str),
                        "description": "Add-on".to_string(),
                    }
                }));
            }
        }
        buttons
    }

    /// Render a DM cart flow for Instagram
    pub fn instagram_cart_summary(items: &[CartItemWithProduct], total: f64) -> String {
        let lines: Vec<String> = items
            .iter()
            .map(|item| {
                let name = item.product_name.as_deref().unwrap_or("Product");
                format!("{} x{} = ${:.2}", name, item.quantity, item.quantity as f64 * item.unit_price)
            })
            .collect();
        format!(
            "🛒 Cart Summary\n{}\n\nTotal: ${:.2}",
            lines.join("\n"),
            total
        )
    }

    // ---------------------------------------------------------------------------
    // Telegram Templates
    // ---------------------------------------------------------------------------

    /// Render catalog navigation for Telegram inline keyboard
    pub fn telegram_catalog_navigation(products: &[Product]) -> serde_json::Value {
        Self::telegram_catalog_navigation_with_options(products, &[])
    }

    /// Render catalog navigation for Telegram with options indicator
    pub fn telegram_catalog_navigation_with_options(
        products: &[Product],
        _options_map: &std::collections::HashMap<String, Vec<ProductOptionWithValues>>,
    ) -> serde_json::Value {
        let buttons: Vec<serde_json::Value> = products.iter().take(10).map(|p| {
            let has_options = _options_map.contains_key(&p.id);
            let options_badge = if has_options { " ⚙️" } else { "" };
            serde_json::json!({
                "text": format!("{}{} — ${:.2}", p.name, options_badge, p.price),
                "callback_data": format!("product_{}", p.id)
            })
        }).collect();

        serde_json::json!({
            "inline_keyboard": [buttons.chunks(2).map(|chunk| chunk.to_vec()).collect::<Vec<_>>()]
        })
    }

    /// Render Telegram inline keyboard for product options
    pub fn telegram_product_options_keyboard(
        product: &Product,
        options: &[ProductOptionWithValues],
    ) -> serde_json::Value {
        let mut rows: Vec<Vec<serde_json::Value>> = Vec::new();

        for opt in options {
            if !opt.values.is_empty() {
                let req_tag = if opt.option.required { " *" } else { "" };
                let mut row = vec![serde_json::json!({
                    "text": format!("🔹{}{}", opt.option.name, req_tag),
                    "callback_data": format!("option_{}", opt.option.id),
                })];
                for val in &opt.values {
                    let price_str = if val.price_adjustment != 0.0 {
                        format!(" +${:.2}", val.price_adjustment)
                    } else {
                        "".to_string()
                    };
                    row.push(serde_json::json!({
                        "text": format!("{}{}", val.name, price_str),
                        "callback_data": format!("value_{}", val.id),
                    }));
                }
                rows.push(row);
            } else {
                let price_str = if opt.option.price_adjustment != 0.0 {
                    format!(" +${:.2}", opt.option.price_adjustment)
                } else {
                    "".to_string()
                };
                rows.push(vec![serde_json::json!({
                    "text": format!("{}{}", opt.option.name, price_str),
                    "callback_data": format!("option_{}", opt.option.id),
                })]);
            }
        }

        // Add confirm button
        rows.push(vec![
            serde_json::json!({
                "text": "✅ Add to cart",
                "callback_data": format!("add_to_cart_{}", product.id),
            }),
            serde_json::json!({
                "text": "❌ Cancel",
                "callback_data": "cancel",
            }),
        ]);

        serde_json::json!({
            "inline_keyboard": rows
        })
    }

    /// Render cart buttons for Telegram
    pub fn telegram_cart_buttons(items: &[CartItemWithProduct], total: f64) -> serde_json::Value {
        serde_json::json!({
            "inline_keyboard": [
                [
                    {"text": format!("🛒 {} items — ${:.2}", items.len(), total), "callback_data": "view_cart"},
                    {"text": "💳 Checkout", "callback_data": "checkout"}
                ]
            ]
        })
    }

    // ---------------------------------------------------------------------------
    // Email Templates
    // ---------------------------------------------------------------------------

    /// Render HTML catalog email
    pub fn email_catalog_html(products: &[Product]) -> String {
        Self::email_catalog_html_with_options(products, &std::collections::HashMap::new())
    }

    /// Render HTML catalog email with product options
    pub fn email_catalog_html_with_options(
        products: &[Product],
        options_map: &std::collections::HashMap<String, Vec<ProductOptionWithValues>>,
    ) -> String {
        let product_rows: Vec<String> = products.iter().map(|p| {
            let img = p.images.as_ref()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
                .and_then(|v| v.as_array().and_then(|arr| arr.first().and_then(|f| f.as_str()).map(|s| s.to_string())))
                .unwrap_or_default();

            let image_html = if !img.is_empty() {
                format!("<td><img src=\"{}\" width=\"100\" alt=\"{}\"></td>", img, p.name)
            } else {
                "<td>📦</td>".to_string()
            };

            // Build options HTML if this product has options
            let options_html = if let Some(options) = options_map.get(&p.id) {
                let mut html = "<div style='margin-top:8px; padding:8px; background:#f9f9f9; border-radius:4px;'>".to_string();
                html.push_str("<strong style='font-size:12px;'>⚙️ Customize:</strong>");
                for opt in options {
                    html.push_str(&format!("<div style='margin-top:4px;'><strong>{}</strong>", opt.option.name));
                    if opt.option.required {
                        html.push_str(" <em style='color:red;'>(required)</em>");
                    }
                    if !opt.values.is_empty() {
                        html.push_str(": ");
                        for (i, val) in opt.values.iter().enumerate() {
                            if i > 0 { html.push_str(", "); }
                            let price_str = if val.price_adjustment != 0.0 {
                                format!(" (+${:.2})", val.price_adjustment)
                            } else {
                                "".to_string()
                            };
                            html.push_str(&format!("<label style='cursor:pointer;'><input type='checkbox' name='{}' value='{}'> {}</label>",
                                opt.option.id, val.id, val.name));
                            html.push_str(&price_str);
                        }
                    } else {
                        let price_str = if opt.option.price_adjustment != 0.0 {
                            format!(" (+${:.2})", opt.option.price_adjustment)
                        } else {
                            "".to_string()
                        };
                        html.push_str(&format!(": <label><input type='checkbox' name='{}'> Add {}</label>{}",
                            opt.option.id, opt.option.name, price_str));
                    }
                    html.push_str("</div>");
                }
                html.push_str("</div>");
                html
            } else {
                "".to_string()
            };

            format!(
                "<tr>
                    {}
                    <td>
                        <h3>{}</h3>
                        <p>{}</p>
                        <strong>${:.2}</strong>
                        {}
                    </td>
                </tr>",
                image_html,
                p.name,
                p.description.as_deref().unwrap_or(""),
                p.price,
                options_html
            )
        }).collect();

        format!(
            "<html><body>
                <h1>Our Products</h1>
                <table style=\"width:100%%; border-collapse: collapse;\">
                    {}
                </table>
            </body></html>",
            product_rows.join("")
        )
    }

    /// Render HTML order confirmation email
    pub fn email_order_confirmation(order: &CommerceOrder) -> String {
        format!(
            "<html><body>
                <h1>Order Confirmed</h1>
                <p>Thank you for your order!</p>
                <h2>Order {}: ${:.2} {}</h2>
                <p>Status: {}</p>
                <p>We'll notify you when your order ships.</p>
            </body></html>",
            order.order_number,
            order.total_amount,
            order.currency,
            order.status
        )
    }
}

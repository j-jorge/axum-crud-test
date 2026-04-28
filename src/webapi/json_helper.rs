// SPDX-License-Identifier: AGPL-3.0-only
use crate::business;

pub fn to_json(
  products: &[business::shop::ShopProduct],
) -> business::result::Result<Vec<serde_json::value::Value>> {
  let mut json_products: Vec<serde_json::value::Value> = Vec::with_capacity(4);

  for p in products.iter() {
    let mut m: std::collections::HashMap<&str, serde_json::value::Value> =
      std::collections::HashMap::new();
    m.insert("product-id", serde_json::to_value(&p.product_id)?);
    m.insert("coins", serde_json::to_value(p.coins)?);

    json_products.push(serde_json::to_value(m)?);
  }

  return Ok(json_products);
}

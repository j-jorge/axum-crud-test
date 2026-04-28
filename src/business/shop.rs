// SPDX-License-Identifier: AGPL-3.0-only
use super::*;

// The shop lists all products that can be purchased via the store.

pub async fn run_migration(
  transaction: &deadpool_postgres::Transaction<'_>,
  to_version: i32,
) -> result::Result<()> {
  if to_version == 1 {
    // id: should match a product ID from the store.
    // coins: the amount of coins acquired by this purchase.
    transaction
      .batch_execute(
        "create table shop \
         (id text primary key, coins integer)",
      )
      .await?;
  }

  return Ok(());
}

pub struct ShopProduct {
  pub product_id: String,
  pub coins: i32,
}

pub struct Shop {
  m_db: deadpool_postgres::Pool,
}

impl Shop {
  pub fn new(db: deadpool_postgres::Pool) -> Shop {
    let result = Shop { m_db: db };

    return result;
  }

  /// Adds a new product with the given reward in coins, or update the
  /// reward of a product if the ID already exists.
  pub async fn update(
    &self,
    product_id: &str,
    coins: i32,
  ) -> result::Result<()> {
    if coins < 0 {
      return Err(error::Error::InvalidParameter);
    }

    self
      .m_db
      .get()
      .await?
      .execute(
        "insert into shop \
           values ($1, $2) \
           on conflict (id) \
           do update set coins = $2",
        &[&product_id, &coins],
      )
      .await?;

    return Ok(());
  }

  /// Returns a vector of shop products.
  pub async fn list(&self) -> result::Result<Vec<ShopProduct>> {
    return Ok(
      self
        .m_db
        .get()
        .await?
        .query("select * from shop", &[])
        .await?
        .into_iter()
        .map(|row| ShopProduct {
          product_id: row.get(0),
          coins: row.get(1),
        })
        .collect(),
    );
  }
}

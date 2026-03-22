use super::*;

// A game feature is just an ID and an associated cost in coins.

pub async fn run_migration(
  transaction: &deadpool_postgres::Transaction<'_>,
  to_version: i32
) -> result::Result<()>
{
  if to_version == 1
  {
    transaction
      .batch_execute(
        "create table game_features \
         (id text primary key, cost_in_coins integer)"
      )
      .await?;
  }

  return Ok(());
}

pub struct GameFeatures
{
  m_db: deadpool_postgres::Pool
}

impl GameFeatures
{
  pub fn new(db: deadpool_postgres::Pool) -> GameFeatures
  {
    let result = GameFeatures {
      m_db: db
    };

    return result;
  }

  /// Adds a new game feature with the given cost in coins, or update the
  /// price of a game feature if the ID already exists.
  pub async fn update(&self, id: &str, cost_in_coins: i32)
  -> result::Result<()>
  {
    if cost_in_coins < 0
    {
      return Err(error::Error::InvalidParameter);
    }

    self
      .m_db
      .get()
      .await?
      .execute(
        "insert into game_features \
           values ($1, $2) \
           on conflict (id) \
           do update set cost_in_coins = $2",
        &[&id, &cost_in_coins]
      )
      .await?;

    return Ok(());
  }

  /// Returns a map of game feature IDs as keys and their cost as coins as
  /// values.
  pub async fn list(
    &self
  ) -> result::Result<std::collections::HashMap<String, i32>>
  {
    return Ok(
      self
        .m_db
        .get()
        .await?
        .query("select * from game_features", &[])
        .await?
        .into_iter()
        .map(|row| (row.get(0), row.get(1)))
        .collect()
    );
  }
}

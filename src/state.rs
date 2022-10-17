use cw_storage_plus::Item;
use cosmwasm_std::{Coin, Addr};

pub const COUNTER: Item<u64> = Item::new("counter");
pub const MINIMAL_DONATION: Item<Coin> = Item::new("minimal_donation");
pub const OWNER: Item<Addr> = Item::new("owner");
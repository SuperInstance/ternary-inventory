# ternary-inventory

**Items, equipment, loot tables, and trading for ternary-valued game systems.**

Every game needs items. Every item has properties. When your properties are ternary — `{-1 = cursed, 0 = mundane, +1 = enchanted}` — the entire inventory system becomes beautifully simple. No floating-point stats. No power creep calculations. Just three states per property, and every item tells you exactly what it is.

This crate provides a complete item system: creation, inventory management, equipment slots, loot tables with drop probabilities, and trade offers between agents.

## What's Inside

- **`Item`** — id, name, weight, ternary value (`Negative`/`Zero`/`Positive`), and tags
- **`Inventory`** — capacity-bounded collection of items. Weight limits, item stacking, find by tag/value
- **`EquipmentSlot`** — bind items to named slots (head, hands, weapon, etc.)
- **`LootTable`** — probabilistic drops. Define items with drop weights, roll for loot
- **`TradeOffer`** — structured exchange between two inventories with validation
- **`craft(recipe, inventory)`** — consume materials, produce new items

## Quick Example

```rust
use ternary_inventory::*;

// Create items
let sword = Item::new(1, "Rusty Sword", 3.0, Ternary::Negative)
    .with_tag("weapon")
    .with_tag("metal");

let ring = Item::new(2, "Lucky Ring", 0.1, Ternary::Positive)
    .with_tag("accessory")
    .with_tag("magic");

// Manage inventory
let mut bag = Inventory::new(50.0); // 50 kg capacity
bag.add(sword.clone()).unwrap();
bag.add(ring.clone()).unwrap();

// Equip
let mut equip = EquipmentSlots::new();
equip.equip("weapon", sword.clone());
equip.equip("ring", ring.clone());

// Loot table
let mut loot = LootTable::new();
loot.add(Item::new(3, "Gold Coin", 0.01, Ternary::Zero), 50.0); // common
loot.add(Item::new(4, "Enchanted Gem", 0.1, Ternary::Positive), 5.0); // rare

let drops = loot.roll(3); // roll 3 times
```

## The Insight

**Three-valued items are enough.** Most RPG systems have 0-100 stat ranges that nobody reads. Ternary values force design clarity: an item is either *bad*, *neutral*, or *good*. Players understand that instantly. "This sword is cursed (-1). This ring is enchanted (+1). That potion is mundane (0)." No math required.

**Use cases:**
- **Roguelike games** — lightweight item system with clear ternary properties
- **Board game engines** — item cards with ternary stats
- **Economic simulations** — goods with quality ratings (negative/neutral/positive)
- **Asset management** — tag items as liabilities/neutral/assets
- **Educational games** — teach categorization with ternary-valued objects

## See Also

- **ternary-auction** — buy and sell these items through auction mechanisms
- **ternary-trust** — trust between trading partners affects deal quality
- **ternary-consensus** — group decisions about inventory management
- **ternary-shipyard** — construction systems that consume inventory items

## Install

```bash
cargo add ternary-inventory
```

## License

MIT

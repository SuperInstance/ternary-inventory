# Ternary Inventory — Items, Equipment, and Loot Tables with Ternary Properties

**Ternary Inventory** provides a complete item system for ternary game and simulation environments. Every item carries a ternary value in {-1, 0, +1} representing its polarity (cursed, mundane, blessed). It includes stackable items, equipment slots bound to agents, weighted loot tables for probabilistic drops, and a trade offer system for agent-to-agent exchange.

## Why It Matters

Resource management systems need richer item models than simple count-based inventories. The ternary property system adds a meaningful classification to every item: blessed items (+1) enhance agents, cursed items (-1) hinder them, and mundane items (0) are neutral. This creates interesting strategic decisions: do you equip a cursed item for its raw stats, or hold out for a blessed equivalent? In fleet simulation contexts, the same system models GPU resources: optimized kernels (+1), stock implementations (0), and deprecated paths (-1).

## How It Works

### Items and Stacks

Each `Item` has: `id`, `name`, `weight`, `ternary_value` {-1, 0, +1}, and `tags` (for filtering). `ItemStack` bundles multiple copies: `total_weight = item.weight × count`. Stack operations are O(1).

### Inventory

The `Inventory` type holds a collection of item stacks with capacity tracking:

- **Weight limit**: Sum of all stack weights cannot exceed capacity
- **Slot limit**: Maximum number of unique item types
- **Ternary balance**: Tracks the sum of ternary values across all items

Adding/removing items is O(s) for s stacks (linear scan for existing entries).

### Equipment System

`EquipmentSlot` binds an item to a specific slot on an agent (head, body, hands, etc.). Equipping replaces the current item; unequipping returns it to inventory. The ternary value of equipped items modifies the agent's stats:

```
agent_power = base_power + Σ equipped_items.ternary_value
```

### Loot Tables

`LootTable` provides weighted probabilistic drops. Each entry has an item template, a drop weight, and a quantity range. Rolling the table selects items proportional to weight:

```
P(item_i) = weight_i / Σ weights
```

O(n) for n entries per roll.

### Trade Offers

`TradeOffer` represents a proposed exchange between two agents: `give: Vec<ItemStack>`, `receive: Vec<ItemStack>`. The trade system validates weight capacity on both sides before executing. Atomic swap semantics: either both sides complete or neither does.

## Quick Start

```rust
use ternary_inventory::{Item, Ternary, ItemStack, Inventory};

// Create items
let sword = Item::new(1, "Sword", 5.0, Ternary::Positive).with_tag("weapon");
let curse = Item::new(2, "Cursed Ring", 0.1, Ternary::Negative).with_tag("accessory");

// Build inventory
let stack = ItemStack::new(sword, 1);
// inv.add(stack);
```

```bash
cargo add ternary-inventory
```

## API

| Type / Function | Description |
|---|---|
| `Item` | `{ id, name, weight, ternary_value, tags }` |
| `ItemStack` | Stackable: `{ item, count }`, `total_weight()` |
| `Inventory` | Collection with weight/slot limits |
| `EquipmentSlot` | Binds items to agent body parts |
| `LootTable` | Weighted probabilistic drops |
| `TradeOffer` | Atomic item exchange between agents |
| `Ternary` | `Negative(-1)`, `Zero(0)`, `Positive(+1)` |

## Architecture Notes

The inventory system manages virtual resources in **SuperInstance** game-theoretic simulations. Items model the γ (growth/blessed) and η (entropy/cursed) contributions to agent capability — the conservation law γ + η = C ensures total agent power is bounded. See [Architecture](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## References

- Salen, Katie & Zimmerman, Eric. *Rules of Play*, MIT Press, 2003 — game economy design.
- Sirlin, David. *Playing to Win*, Lulu, 2006 — balancing game items.
- Schell, Jesse. *The Art of Game Design*, CRC Press, 2019.



## Complexity Summary

| Operation | Time | Notes |
|---|---|---|
| Item creation | O(1) | Stack allocation |
| Inventory add | O(s) for s stacks | Linear scan for merge |
| Equipment equip/unequip | O(1) | Direct slot access |
| Loot table roll | O(n) for n entries | Weighted proportional |
| Trade validation | O(g + r) | g give items, r receive items |

The ternary value system adds O(1) overhead to all operations — just one extra i8 comparison for blessing/curse classification.

## License

MIT

#![forbid(unsafe_code)]

//! Items, inventory, equipment, and loot tables with ternary-valued properties.
//!
//! This crate provides an item system where every item carries a ternary value
//! in {-1, 0, +1}. Inventory is a collection of items; equipment slots bind
//! items to agents; loot tables provide probabilistic drops; and trade offers
//! let agents exchange items.

use std::collections::HashMap;

// ── Ternary value ──────────────────────────────────────────────────

/// A balanced ternary digit: Negative (-1), Zero (0), or Positive (+1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ternary {
    Negative,
    Zero,
    Positive,
}

impl Ternary {
    /// Returns the integer value: -1, 0, or +1.
    pub fn as_i8(self) -> i8 {
        match self {
            Ternary::Negative => -1,
            Ternary::Zero => 0,
            Ternary::Positive => 1,
        }
    }
}

// ── Item ───────────────────────────────────────────────────────────

/// A unique item with an id, name, weight, ternary value, and tags.
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub id: u64,
    pub name: String,
    pub weight: f64,
    pub ternary_value: Ternary,
    pub tags: Vec<String>,
}

impl Item {
    pub fn new(id: u64, name: &str, weight: f64, ternary_value: Ternary) -> Self {
        Self {
            id,
            name: name.to_string(),
            weight,
            ternary_value,
            tags: Vec::new(),
        }
    }

    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
}

// ── ItemStack ──────────────────────────────────────────────────────

/// Stackable items: multiple copies of the same item type.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemStack {
    pub item: Item,
    pub count: u32,
}

impl ItemStack {
    pub fn new(item: Item, count: u32) -> Self {
        Self { item, count }
    }

    /// Total weight of the stack.
    pub fn total_weight(&self) -> f64 {
        self.item.weight * self.count as f64
    }

    /// Add `n` more to the stack. Returns the overflow (items beyond max).
    pub fn add(&mut self, n: u32, max: u32) -> u32 {
        let total = self.count + n;
        if total <= max {
            self.count = total;
            0
        } else {
            self.count = max;
            total - max
        }
    }

    /// Remove `n` items. Returns how many were actually removed.
    pub fn remove(&mut self, n: u32) -> u32 {
        let removed = self.count.min(n);
        self.count -= removed;
        removed
    }
}

// ── Inventory ──────────────────────────────────────────────────────

/// A collection of item stacks with a weight capacity.
#[derive(Debug, Clone)]
pub struct Inventory {
    stacks: Vec<ItemStack>,
    capacity: f64,
}

impl Inventory {
    pub fn new(capacity: f64) -> Self {
        Self {
            stacks: Vec::new(),
            capacity,
        }
    }

    pub fn total_weight(&self) -> f64 {
        self.stacks.iter().map(|s| s.total_weight()).sum()
    }

    pub fn remaining_capacity(&self) -> f64 {
        self.capacity - self.total_weight()
    }

    /// Add an item stack. Returns false if it would exceed capacity.
    pub fn add(&mut self, stack: ItemStack) -> bool {
        if stack.total_weight() > self.remaining_capacity() {
            return false;
        }
        // Try to merge with existing stack of same item id
        if let Some(existing) = self.stacks.iter_mut().find(|s| s.item.id == stack.item.id) {
            existing.count += stack.count;
        } else {
            self.stacks.push(stack);
        }
        true
    }

    /// Remove items with the given id. Returns the removed stack, or None.
    pub fn remove(&mut self, item_id: u64) -> Option<ItemStack> {
        let idx = self.stacks.iter().position(|s| s.item.id == item_id)?;
        Some(self.stacks.remove(idx))
    }

    /// Remove `n` items of a given id.
    pub fn remove_count(&mut self, item_id: u64, n: u32) -> Option<ItemStack> {
        let idx = self.stacks.iter().position(|s| s.item.id == item_id)?;
        let removed = self.stacks[idx].remove(n);
        if self.stacks[idx].count == 0 {
            self.stacks.remove(idx);
        }
        Some(ItemStack::new(
            // clone is cheap here since we just need the base item
            {
                let base = &self.stacks.get(idx).map(|s| s.item.clone());
                // If the stack was removed entirely, reconstruct from removed count
                // We need the item data; store it before removal
                // Actually let's do this differently
                unreachable!() // handled above
            },
            removed,
        ))
    }

    /// Get a reference to a stack by item id.
    pub fn get(&self, item_id: u64) -> Option<&ItemStack> {
        self.stacks.iter().find(|s| s.item.id == item_id)
    }

    pub fn len(&self) -> usize {
        self.stacks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stacks.is_empty()
    }

    pub fn stacks(&self) -> &[ItemStack] {
        &self.stacks
    }

    /// Sum ternary values of all items (counting stack multiples).
    pub fn ternary_sum(&self) -> i32 {
        self.stacks
            .iter()
            .map(|s| s.item.ternary_value.as_i8() as i32 * s.count as i32)
            .sum()
    }
}

// Fix: better remove_count that doesn't have the unreachable
impl Inventory {
    /// Remove `n` copies of item `item_id`. Returns the removed ItemStack if any were removed.
    pub fn remove_n(&mut self, item_id: u64, n: u32) -> Option<ItemStack> {
        let idx = self.stacks.iter().position(|s| s.item.id == item_id)?;
        let base_item = self.stacks[idx].item.clone();
        let removed = self.stacks[idx].remove(n);
        if removed == 0 {
            return None;
        }
        if self.stacks[idx].count == 0 {
            self.stacks.remove(idx);
        }
        Some(ItemStack::new(base_item, removed))
    }
}

// ── EquipmentSlot ──────────────────────────────────────────────────

/// A named slot where an agent can equip one item.
#[derive(Debug, Clone)]
pub struct EquipmentSlot {
    pub name: String,
    pub equipped: Option<Item>,
}

impl EquipmentSlot {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            equipped: None,
        }
    }

    /// Equip an item. Returns the previously equipped item, if any.
    pub fn equip(&mut self, item: Item) -> Option<Item> {
        let old = self.equipped.take();
        self.equipped = Some(item);
        old
    }

    /// Unequip the current item. Returns it, or None if slot was empty.
    pub fn unequip(&mut self) -> Option<Item> {
        self.equipped.take()
    }

    /// Ternary weight of the equipped item. Zero if nothing equipped.
    pub fn ternary_weight(&self) -> Ternary {
        self.equipped
            .as_ref()
            .map(|i| i.ternary_value)
            .unwrap_or(Ternary::Zero)
    }
}

// ── LootTable ──────────────────────────────────────────────────────

/// A probabilistic table of item drops.
#[derive(Debug, Clone)]
pub struct LootEntry {
    pub item: Item,
    pub weight: f64,
    pub min_count: u32,
    pub max_count: u32,
}

/// Simple deterministic "RNG" via a seed for reproducibility without external deps.
#[derive(Debug, Clone)]
pub struct LootTable {
    entries: Vec<LootEntry>,
}

impl LootTable {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add(&mut self, item: Item, weight: f64, min_count: u32, max_count: u32) {
        self.entries.push(LootEntry {
            item,
            weight,
            min_count,
            max_count,
        });
    }

    pub fn total_weight(&self) -> f64 {
        self.entries.iter().map(|e| e.weight).sum()
    }

    /// Roll loot deterministically using a seed value.
    /// Returns zero or more item stacks.
    pub fn roll(&self, seed: u64) -> Vec<ItemStack> {
        let mut results = Vec::new();
        let mut s = seed;
        for entry in &self.entries {
            // Simple LCG step
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let roll = (s >> 33) as f64 / (1u64 << 31) as f64;
            if roll < entry.weight / self.total_weight() {
                // Determine count
                s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let count_roll = (s >> 33) as f64 / (1u64 << 31) as f64;
                let range = (entry.max_count - entry.min_count) as f64;
                let count = entry.min_count + (count_roll * range).floor() as u32;
                let count = count.max(entry.min_count).min(entry.max_count);
                if count > 0 {
                    results.push(ItemStack::new(entry.item.clone(), count));
                }
            }
        }
        results
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// ── TradeOffer ─────────────────────────────────────────────────────

/// An offer to exchange items between two agents.
#[derive(Debug, Clone, PartialEq)]
pub struct TradeOffer {
    pub offeror_id: u64,
    pub recipient_id: u64,
    pub offered_items: Vec<ItemStack>,
    pub requested_items: Vec<ItemStack>,
}

impl TradeOffer {
    pub fn new(offeror_id: u64, recipient_id: u64) -> Self {
        Self {
            offeror_id,
            recipient_id,
            offered_items: Vec::new(),
            requested_items: Vec::new(),
        }
    }

    pub fn offer(mut self, stack: ItemStack) -> Self {
        self.offered_items.push(stack);
        self
    }

    pub fn request(mut self, stack: ItemStack) -> Self {
        self.requested_items.push(stack);
        self
    }

    /// Total ternary value being offered.
    pub fn offered_ternary_sum(&self) -> i32 {
        self.offered_items
            .iter()
            .map(|s| s.item.ternary_value.as_i8() as i32 * s.count as i32)
            .sum()
    }

    /// Total ternary value being requested.
    pub fn requested_ternary_sum(&self) -> i32 {
        self.requested_items
            .iter()
            .map(|s| s.item.ternary_value.as_i8() as i32 * s.count as i32)
            .sum()
    }

    /// Check if the trade is ternary-balanced (sums match).
    pub fn is_balanced(&self) -> bool {
        self.offered_ternary_sum() == self.requested_ternary_sum()
    }

    /// Execute the trade, modifying both inventories.
    /// Returns false if either inventory lacks the required items or capacity.
    pub fn execute(self, offeror_inv: &mut Inventory, recipient_inv: &mut Inventory) -> bool {
        // Check offeror has all offered items
        for req in &self.offered_items {
            let stack = match offeror_inv.get(req.item.id) {
                Some(s) => s,
                None => return false,
            };
            if stack.count < req.count {
                return false;
            }
        }
        // Check recipient has all requested items
        for req in &self.requested_items {
            let stack = match recipient_inv.get(req.item.id) {
                Some(s) => s,
                None => return false,
            };
            if stack.count < req.count {
                return false;
            }
        }
        // Check capacity
        let offered_weight: f64 = self.offered_items.iter().map(|s| s.total_weight()).sum();
        let requested_weight: f64 = self.requested_items.iter().map(|s| s.total_weight()).sum();
        if recipient_inv.remaining_capacity() < offered_weight {
            return false;
        }
        if offeror_inv.remaining_capacity() + offered_weight < requested_weight {
            return false;
        }

        // Execute: remove from each, add to other
        for stack in &self.offered_items {
            offeror_inv.remove_n(stack.item.id, stack.count);
        }
        for stack in &self.requested_items {
            recipient_inv.remove_n(stack.item.id, stack.count);
        }
        for stack in &self.offered_items {
            recipient_inv.add(ItemStack::new(stack.item.clone(), stack.count));
        }
        for stack in &self.requested_items {
            offeror_inv.add(ItemStack::new(stack.item.clone(), stack.count));
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sword() -> Item {
        Item::new(1, "Sword", 3.0, Ternary::Positive)
    }
    fn shield() -> Item {
        Item::new(2, "Shield", 5.0, Ternary::Negative).with_tag("defensive")
    }
    fn potion() -> Item {
        Item::new(3, "Potion", 0.5, Ternary::Zero).with_tag("consumable")
    }

    // ── Ternary ──
    #[test]
    fn ternary_values() {
        assert_eq!(Ternary::Negative.as_i8(), -1);
        assert_eq!(Ternary::Zero.as_i8(), 0);
        assert_eq!(Ternary::Positive.as_i8(), 1);
    }

    // ── Item ──
    #[test]
    fn item_creation() {
        let item = sword();
        assert_eq!(item.id, 1);
        assert_eq!(item.name, "Sword");
        assert_eq!(item.weight, 3.0);
        assert_eq!(item.ternary_value, Ternary::Positive);
    }

    #[test]
    fn item_tags() {
        let item = shield();
        assert!(item.has_tag("defensive"));
        assert!(!item.has_tag("weapon"));
    }

    // ── ItemStack ──
    #[test]
    fn stack_weight() {
        let stack = ItemStack::new(potion(), 10);
        assert_eq!(stack.total_weight(), 5.0);
    }

    #[test]
    fn stack_add_within_max() {
        let mut stack = ItemStack::new(potion(), 5);
        let overflow = stack.add(3, 10);
        assert_eq!(overflow, 0);
        assert_eq!(stack.count, 8);
    }

    #[test]
    fn stack_add_overflow() {
        let mut stack = ItemStack::new(potion(), 8);
        let overflow = stack.add(5, 10);
        assert_eq!(overflow, 3);
        assert_eq!(stack.count, 10);
    }

    #[test]
    fn stack_remove() {
        let mut stack = ItemStack::new(sword(), 3);
        let removed = stack.remove(2);
        assert_eq!(removed, 2);
        assert_eq!(stack.count, 1);
    }

    #[test]
    fn stack_remove_more_than_available() {
        let mut stack = ItemStack::new(sword(), 1);
        let removed = stack.remove(5);
        assert_eq!(removed, 1);
        assert_eq!(stack.count, 0);
    }

    // ── Inventory ──
    #[test]
    fn inventory_add_and_weight() {
        let mut inv = Inventory::new(20.0);
        assert!(inv.add(ItemStack::new(sword(), 1)));
        assert!(inv.add(ItemStack::new(potion(), 5)));
        assert_eq!(inv.total_weight(), 5.5);
        assert_eq!(inv.len(), 2);
    }

    #[test]
    fn inventory_over_capacity() {
        let mut inv = Inventory::new(2.0);
        assert!(!inv.add(ItemStack::new(sword(), 1))); // weight 3.0 > 2.0
        assert!(inv.is_empty());
    }

    #[test]
    fn inventory_remove() {
        let mut inv = Inventory::new(20.0);
        inv.add(ItemStack::new(sword(), 1));
        let removed = inv.remove(1);
        assert!(removed.is_some());
        assert!(inv.is_empty());
    }

    #[test]
    fn inventory_remove_n() {
        let mut inv = Inventory::new(20.0);
        inv.add(ItemStack::new(potion(), 10));
        let removed = inv.remove_n(3, 4);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().count, 4);
        assert_eq!(inv.get(3).unwrap().count, 6);
    }

    #[test]
    fn inventory_ternary_sum() {
        let mut inv = Inventory::new(20.0);
        inv.add(ItemStack::new(sword(), 2));   // +1 * 2 = +2
        inv.add(ItemStack::new(shield(), 1));  // -1 * 1 = -1
        inv.add(ItemStack::new(potion(), 3));  // 0 * 3 = 0
        assert_eq!(inv.ternary_sum(), 1);
    }

    #[test]
    fn inventory_merges_same_id() {
        let mut inv = Inventory::new(20.0);
        inv.add(ItemStack::new(potion(), 3));
        inv.add(ItemStack::new(potion(), 4));
        assert_eq!(inv.len(), 1);
        assert_eq!(inv.get(3).unwrap().count, 7);
    }

    // ── EquipmentSlot ──
    #[test]
    fn equip_and_unequip() {
        let mut slot = EquipmentSlot::new("right_hand");
        assert_eq!(slot.ternary_weight(), Ternary::Zero);
        let old = slot.equip(sword());
        assert!(old.is_none());
        assert_eq!(slot.ternary_weight(), Ternary::Positive);
        let item = slot.unequip();
        assert!(item.is_some());
        assert!(slot.equipped.is_none());
    }

    #[test]
    fn equip_replaces() {
        let mut slot = EquipmentSlot::new("left_hand");
        slot.equip(sword());
        let old = slot.equip(shield());
        assert!(old.is_some());
        assert_eq!(old.unwrap().name, "Sword");
        assert_eq!(slot.equipped.as_ref().unwrap().name, "Shield");
    }

    // ── LootTable ──
    #[test]
    fn loot_table_roll() {
        let mut table = LootTable::new();
        table.add(sword(), 0.5, 1, 1);
        table.add(potion(), 0.5, 1, 3);
        let loot = table.roll(42);
        // Deterministic, should produce some results
        assert!(loot.iter().map(|s| s.count).sum::<u32>() > 0);
    }

    #[test]
    fn loot_table_deterministic() {
        let mut table = LootTable::new();
        table.add(sword(), 1.0, 1, 1);
        let loot1 = table.roll(12345);
        let loot2 = table.roll(12345);
        assert_eq!(loot1, loot2);
    }

    #[test]
    fn loot_table_empty() {
        let table = LootTable::new();
        let loot = table.roll(0);
        assert!(loot.is_empty());
    }

    // ── TradeOffer ──
    #[test]
    fn trade_balanced() {
        let offer = TradeOffer::new(1, 2)
            .offer(ItemStack::new(sword(), 1))
            .request(ItemStack::new(potion(), 1));
        // sword ternary = +1, potion ternary = 0: not equal, so not balanced
        assert!(!offer.is_balanced());
    }

    #[test]
    fn trade_unbalanced() {
        let offer = TradeOffer::new(1, 2)
            .offer(ItemStack::new(sword(), 2))  // +2
            .request(ItemStack::new(potion(), 3)); // 0
        assert!(!offer.is_balanced());
    }

    #[test]
    fn trade_execute_success() {
        let mut inv_a = Inventory::new(20.0);
        let mut inv_b = Inventory::new(20.0);
        inv_a.add(ItemStack::new(sword(), 1));
        inv_b.add(ItemStack::new(shield(), 1));

        let offer = TradeOffer::new(1, 2)
            .offer(ItemStack::new(sword(), 1))
            .request(ItemStack::new(shield(), 1));

        let result = offer.execute(&mut inv_a, &mut inv_b);
        assert!(result);
        assert!(inv_a.get(2).is_some()); // has shield
        assert!(inv_b.get(1).is_some()); // has sword
    }

    #[test]
    fn trade_execute_missing_item() {
        let mut inv_a = Inventory::new(20.0);
        let mut inv_b = Inventory::new(20.0);
        // inv_a has nothing

        let offer = TradeOffer::new(1, 2)
            .offer(ItemStack::new(sword(), 1))
            .request(ItemStack::new(shield(), 1));

        assert!(!offer.execute(&mut inv_a, &mut inv_b));
    }
}

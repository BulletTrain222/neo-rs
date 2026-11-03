# SPEC-0021: Items, pickups, drops and money

- Answers: R-0086 (clause 1 only); R-0104 (item-36 source and Glade possession branch, in part); proactive (starting equipment); proactive (Neopoints); R-0155 (coin model beside the price caption, Display); R-0188 (showcase effect table, the showcase camera call, the shield pose stinger); R-0195 (Panel visibility and clears, item 5, the HUD reset's callers); R-0196 (item 57 only); proactive (open close-out); R-0225 (the shop copies beside 1427)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001). Check the boot program's data words listed below and the item table's record count (208 in the config resource, 181 rows in the text database) before trusting the rest.
- Confidence: per section. Layouts and tables: High. Drop and roll formulas: High. Drop-roll events: High for breakable and scripted-add routes; High for the order of the enemy death route; Medium-High for its money-roll details. Steal coin split: High. Coin spawn argument order: High. Pickup dispatch: Medium. Heal formula: High. Shop purchase rule: High for the decision order. Buyable price: High (the 16-bit field at +0x8A of the world-item record, copied from the placement's fixed count at spawn; a script setter can overwrite it). Shop stock, take or copy, carry limit and once-per-area items: High. Equipment ranking: High for the rule, Medium for what it means. Equipment slots: the three equipment words hold Motes (word 0 the armour Mote, word 1 the shield Mote, word 2 the weapon Mote that is the attack) and the equip/unequip rules, High; the weapon, shield and body items are the hero's entries in the category 1, 2 and 5 item lists, Medium-High. Mote charge (count 100, use amounts, removal when spent): High. Weapon reach tables: High for the values and the layout, Medium-High for the meaning. Quick-item panel (pad trigger; SPEC-0033 Controls): Medium-High for the four entries, the opening, the direction, select and use rules and the marker; the second quick panel and Mote equipping are in their own section (High for the direction rule, the table layout and the equip rules; Medium-High for the slot kind arguments). Item screen (SPEC-0035 section 14): the grid's 20 kinds and the use trigger on Cross, High for the list and Medium-High for the use trigger; Circle on the Items tab sets or clears the quick-item selection marker and writes no equipment word, High; the Charms tab runs the second quick panel's Mote picker, High; Square on the Items tab is the cloak swap, High. Tattered Cloak (lists, swap, its callers, removal on an attack or a hit): High. Wallet (shared Neopoints; remove and give rules): High for the rules; High for the identity (coin pickups and the shop change the same value, Wallet identity). Pickup timing gate: High (states, thresholds, strict boundary and the age clock). World-item pool, spawn, per-tick step, motion families and expiry: High. Pickup (nearest-item search, collect distance rule, range collect, refusals): High for the rules; Medium for the character radius term (its value is open). Auto-collect: High for the steps and for the script item hook (it never fires on disc); Medium-Low for the first-find hook. Important-item give: High for when it runs and for the double-give mechanism; Medium for its visible result. Clover patches: Medium-High. Coin pickup values: High (ids 140 to 144 add nothing). The Neopoints chapter (wallet, sources, sinks, theft, display, worked examples): High for the wallet rules, the coin values, the money roll, the coin split, the steal, the buy path and the save fields; High for the 50-slot coin limit; Medium-High for the dialogue pay-out amounts; Medium for which dialogue pay-out lines are shown in play and for the totals. Stat-up items (flags and pool effects): Medium-High for the flag tests and effects, Medium for the luck rule, Low for the caps, High for the shop sources of the three stat-up Neggs. Consumable effects by item id: Medium-High for the status pairs and the item 50 revive; High for the status test direction; Medium for the placement of item 57. Heal streak window and reaction ids: Medium-High. Use cues: Medium-High. Charm spin and hop: High. Slot 0 armour Mote: High. Shop pre-check: Medium-Low (busy-slot gate; its other tests not established). Shop item highlight and the Square press: Medium-High. Enemy projectile hits and dormant reach routines (pointers; no item rule): High. Mote drain by the mote eater: High; its timing Medium-High. Party inventory lists (capacities, slot layout, queries, add and remove): High; the duplicate-slot quirk Medium-High. Panel visibility, clears and the pad mask: High. Conditional placements (flags 2 to 4): High. Ring layout (Medium), the one-shot event with cue 0xB2 and kind 0x91 (Low-Medium), item display models, including slot ids 0x32 to 0x3B and the weapon kind (High for the index rules and resource numbers; Medium-High for reach, item names and the index-in-range quirk), the item model registry (High), the item showcase (High for the numbers; Medium for the corner placement), the 0xA0 to 0xAD category and float attribute lookups (Low).

## Summary

Items are identified by a number from 0 to 322. Two tables on the disc describe them. The first is a config table (one resource in the game archive) with 208 records. Each record holds an item's type, its eight stat fields and its equipment, consumable or charm values. The second is table 0 of the item text database (archive resource 533), a 181-row table with the item name, a category, a price and a description. Both tables are indexed directly by item id.

Items reach the player by four routes (Sources): direct placements (kind-8 placements with a key of 0 or more), drop placements (key -1, which roll a drop group for money and an item), script grants by a fixed item id, and a world spawn call that places the 100 Neopoint coin (item 139) (Sources). Dialogue give-item commands are a fifth route; for 18 story ids they are the only one (Counts, Dialogue grants), and 67 of them pay Neopoints, which makes them the main fixed income after drops and chests (Neopoints, Sources). A placement either names an item directly or names a drop group. A luck gate can block a placement's pickup before it spawns. Money comes out as coin items (ids 135 to 139) split into denominations. An implementation can build the item database, drops, pickups, equipment ranking, consumable healing and the buy flow from this spec. A shop item is a kind-8 placement with flag byte 1, sold at its fixed count; each item has a carry limit, and five stat-up items can be gained only once per area (Shop purchase rule, Stat-up items). Tunable data words are read from the disc at the addresses given. Some timing and cue constants (the pickup thresholds, the heal streak window and the cue ids) are fixed constants and are stated as values. Neopoints are one shared wallet for the whole party: shop purchases, scripts and coin pickups all change the same value (Wallet). A coin picked up in the world is not kept as an item; its value goes straight to the wallet (Coin pickups). Some stat-up items raise the party's pool maxima when they are picked up (Stat-up items), and some consumables grant status buffs or cures (Consumable effects by item id). The item manager is the party inventory, which keeps per-category lists of item slots of fixed capacity in runtime memory (Party inventory lists). Those slots are not the hero's three equipment slots. World items live in one pool of 50 slots per world; coins fly and bounce, charms hop and other items hover and bob, and a dropped item fades out after its lifetime (World items). A hero collects the nearest ready item within a short distance, and important items also from up to 25 units (Pickup). A ring layout and a one-shot event are in Ring layout and item draw and One-shot event below. A second quick panel selects a Mote and equips it into one of the three equipment slots (Second quick panel and Mote equipping). Every Mote carries a charge of 100 that the hero's attacks and defences spend; a spent Mote is removed (Mote charge).

## Where it lives on the disc

- **Item config table**: archive resource 529 (a plain binary blob, not IDM chunks). Read by resource number through the archive index (SPEC-0001).
- **Item text database**: archive resource 533, which is an IDM chunk file. Its first type-42 chunk is the item database (table 0). Its other tables are not covered here.
- **Level placements**: type-36 region children inside the world packs (SPEC-0004 and SPEC-0006). Kind-8 records are item placements.
- **Placement id table**: a sub-record of the type-15 chunk of each world pack (Layout, Placement id table). Breakable drop ids and the id form of a scripted add name placements through it.
- **Wallet**: the shared Neopoints value is created in memory at start-up. It has no disc location; the rules are in Wallet and in SPEC-0050.
- **Party inventory lists**: the item manager's per-category slots exist only in runtime memory. They have no disc location (Party inventory lists).
- **Boot program data words**: the boot executable SCUS_973.67 is loaded at its load address, the lowest virtual address of its loadable segment (given in the ELF program header). A virtual address V is at file offset V − load address + the segment's file offset (0x80 in this build). Words listed below are read this way. Each is a 32-bit little-endian integer unless marked.

| Word | Virtual address | Count | Contents |
| --- | --- | --- | --- |
| Coin denominations | 0x30F6F0 | 5 | 100, 25, 10, 5, 1 (NP) |
| Coin item ids | 0x30F708 | 5 | 139, 138, 137, 136, 135 (same order as denominations) |
| Heal table A (pool A) | 0x319BE0 | 4 | 500, 400, 300, 500, indexed by character index 0..3. These words are also the per-hero banding caps of SPEC-0025 |
| Heal table B (pool B) | 0x319BE8 | 2 | 300, 500, indexed by character index 0..1 (entries 2 and 3 are not used). These words are also the per-hero banding caps of SPEC-0025 |
| Stat-up pool A gain | 0x319BD0 | 2 | 25 (hero 0), 20 (hero 1). Added to the hero's pool A value by a stat-up item (Stat-up items). The count 2 is the number of entries. The same word is the band step of the banding rule (SPEC-0025) |
| Stat-up pool B gain | 0x319BD8 | 2 | 15 (hero 0), 25 (hero 1). Added to the hero's pool B value by a stat-up item (Stat-up items). The count 2 is the number of entries. The same word is the band step of the banding rule (SPEC-0025) |
| Charm spin base (f32) | 0x2DBC50 | 1 | about 1.2566 (0.4 pi). It differs in the last bit from the lower bound of the spin draw (World items, Spawning), which is a fixed constant of the draw; the draw does not read this word, and whether any routine reads it is not established |
| Coin launch speed centre (f32) | 0x2EB3E8 | 1 | 16.0 units per second; the launch draw is uniform in [centre - 2.0, centre + 2.0) (Coins) |
| Coin gravity (f32) | 0x2EB3EC | 1 | -40.0 units per second squared (Coins) |
| Coin restitution (f32) | 0x2EB3F0 | 1 | -0.65, the factor applied to the vertical speed on a ground contact (Coins) |
| Coin stop speed (f32) | 0x2EB3F4 | 1 | 1.0 unit per second; a bounce slower than this stops (Coins) |
| Coin rest height (f32) | 0x2EB3F8 | 1 | 0.6 units above the ground (Coins) |
| Coin horizontal damping (f32) | 0x2EB3FC | 1 | 0.98 per tick (Coins) |
| Coin spin damping (f32) | 0x2EB400 | 1 | 0.98 per tick (Coins) |
| Coin horizontal speed centre (f32) | 0x2EB404 | 1 | 2.0 units per second; the draw is uniform in [0.5 x centre, 1.5 x centre) (Coins) |

The coin words are read at run time and are not changed at start-up. Every other world-item motion value in World items (the hover heights, the charm gravity and bounce draws, the lifetimes, the thresholds) is a fixed program constant with no data word, stated as a value. The word 0x2DD440 (1.0) is not a world-item value: it is the bounce cooldown of the enemy projectile pool (SPEC-0029).
| Item screen grid kinds | 0x2DCF28 | 20 | Item ids as unsigned 32-bit words, one per grid cell in cell order (cell 0 holds 150). Read at run time; not copied here. |
| Quick-item panel candidate rows | 0x315C68 | 4 rows of 8 | Signed 32-bit item ids. Row k (entry k: 0 Right, 1 Up, 2 Left, 3 Down) starts at 0x315C68 + 0x20 × k. A -1 ends a row. The lists are given in Quick-item panel. |
| Mote selection direction table | 0x2DCE58 | 16 rows of 8 bytes | Next-selection byte for each selection row and direction column. Described in Second quick panel and Mote equipping; the bytes are not copied here. |
| Weapon reach table A (Tor, f32) | 0x2DD5E0 | 11 | Reach for item ids 98 to 108 (index id - 98). The same 11 words are stored twice more, at 0x2DD60C and 0x2DD638 (Weapon reach) |
| Weapon reach table B (Roberta, f32) | 0x2DD670 | 8 | Reach for item ids 111 to 118 (index id - 111). The words after it are other data (Weapon reach) |
| Reach length factor (f32) | 0x2DD690 | 1 | 0.6, the factor of the second reach query (Weapon reach) |
| Reach length bonus (f32) | 0x2DD694 | 1 | 2.0, added to the reach for the hero's swing length (Weapon reach) |

## Layout

### Item config table (resource 529)

Little-endian. Offsets are from the start of the resource.

| Offset | Size | Field |
| --- | --- | --- |
| +0x00 | u32 | Record count N = 208 |
| +0x10 | N × 0x30 | Item records, one per id, id = record index |
| 0x2710 | u32 | Sub-table count M = 11 |
| after M | see below | Sub-tables |
| after sub-tables, aligned up to 16 | u32 | Drop group count K = 7 |
| after K, aligned up to 16 | see below | Drop groups |

**Item record** (0x30 bytes at +0x10 + 0x30 × id):

| Offset | Size | Field | Notes |
| --- | --- | --- | --- |
| +0x00 | 3 × f32 | constant (1.5, 1.0, 1.5) in every record | the third float (+0x08, 1.5) is the pickup distance base (Pickup, step 5). The first two are of unknown meaning |
| +0x0C | u16 | item id (equals record index in all 208 records) | |
| +0x0E | s16 | item type | see the type table below |
| +0x10 | 8 × u16 | constant (150, 1, 2, 3, 4, 5, 6, 7) in every record | meaning unknown |
| +0x20 | s16 | stat A | per type, see Behavior. Weapons (types 3 and 6): melee damage factor A of SPEC-0036. Shields (types 2 and 5): block amount. Body items (types 1 and 4): armour percent |
| +0x22 | s16 | stat B | Weapons: the charge stat S of the Mote charge amounts (100 on every weapon). Body items: the armour-Mote charge stat (100 on every body item except id 134). Shields of type 5: the shield-Mote charge stat |
| +0x24 | s16 | stat C | |
| +0x26 | s16 | stat D | |
| +0x28 | s16 | stat E: consumable pool A percent | 0 for most items |
| +0x2A | s16 | stat F: consumable pool B percent, or charm multiplier percent | |
| +0x2C | s16 | stat G (charms only) | |
| +0x2E | s16 | stat H (charms only) | |

Item types on disc (count of records): 0 (60), 1 (8), 2 (5), 3 (8), 4 (7), 5 (5), 6 (8), 7 (1), 9 (3), 10 (1), 11 (1), 12 (14), 13 (2), 15 (55), 16 (2), 17 (14), 18 (1), 19 (2), 20 (3), 21 (1), 22 (2), 23 (5). Records for ids 181 to 207 are all type 0 (their stat fields are not established). There are no config records for ids 208 to 322.

**Sub-table** (count M = 11): a u32 entry count n, followed by n entries of (u16 item id, s16 weight). The first sub-table starts at the next 16-byte boundary after M. Each following sub-table starts right after the previous one, with no padding. Each sub-table's weights sum to 100.

**Drop group** (count K = 7): a group header then n entries. The groups start at the next 16-byte boundary after K and follow each other with no padding.

| Offset in group | Size | Field |
| --- | --- | --- |
| +0x00 | u16 | 0xFFFF in all groups (constant) |
| +0x02 | s16 | money chance, percent |
| +0x04 | s16 | money minimum |
| +0x06 | s16 | money maximum |
| +0x08 | u32 | entry count n |
| +0x0C | n × 4 | entries: s16 sub-table index, s16 weight |

On disc:

| Group | Money chance % | Money min..max | Entries (sub-table index, weight) |
| --- | --- | --- | --- |
| 0 | 50 | 25..75 | (0, 20), (4, 30), (5, 10) |
| 1 | 40 | 300..800 | (5, 25), (2, 20), (6, 15) |
| 2 | 40 | 10..40 | (1, 20), (4, 10) |
| 3 | 30 | 400..1000 | (2, 10), (8, 15), (10, 20), (5, 15), (6, 10) |
| 4 | 35 | 100..300 | (0, 10), (9, 15), (8, 10), (4, 20), (5, 10) |
| 5 | 50 | 1..5 | none (money only) |
| 6 | 25 | 1..5 | (3, 50) |

The sub-tables, as (item id, weight):

| Sub-table | Entries |
| --- | --- |
| 0 | (146, 30), (157, 30), (153, 15), (156, 10), (147, 10), (155, 5) |
| 1 | (150, 35), (149, 35), (146, 10), (157, 10), (145, 5), (153, 5) |
| 2 | (152, 100) |
| 3 | (150, 50), (149, 50) |
| 4 | (162, 20), (164, 20), (166, 20), (160, 15), (168, 15), (170, 10) |
| 5 | (163, 20), (165, 20), (167, 20), (161, 15), (169, 15), (171, 10) |
| 6 | (172, 75), (173, 25) |
| 7 | (163, 50), (161, 50) |
| 8 | (54, 15), (57, 15), (55, 25), (56, 45) |
| 9 | (50, 20), (51, 30), (52, 30), (53, 20) |
| 10 | (58, 50), (59, 50) |

No sub-table contains an equipment item (types 1 to 6). Drops never produce equipment.

### Item text database (resource 533, first type-42 chunk = table 0)

The chunk holds an IDMX block at its payload start. Offsets below are from the start of the IDMX block (the chunk payload start, 16 bytes into the chunk).

| IDMX offset | Size | Field |
| --- | --- | --- |
| +0x08 | u32 | block size (16 + 181 × 52) |
| +0x0C | u16 | row count = 181 |
| +0x0E | u16 | row size = 52 |
| +0x10 | 181 × 52 | rows; row index = item id (ids 0 to 180) |

Row layout (52 bytes):

| Row offset | Size | Field | Notes |
| --- | --- | --- | --- |
| +0x00 | u32 | name pointer | offset from the IDMX block start; 0 means none. 163 rows have a name. |
| +0x04 | u32 | category | values 0, 1, 2, 5, 6, 7, 8, 9 are used (see Behavior, Item categories) |
| +0x08 | u32 | carry limit Q | values 0 to 5. 0 = no limit. The held count never exceeds Q (Inventory). 1 for every equipment item and most quest items. |
| +0x0C | u32 | catalogue price, NP | 0 to 15000. 110 rows are 0. Not a shop price. Read only by the item branch of an enemy steal: the enemy's carried Neopoints grow by this value (the per-item value of SPEC-0029, Behavior 11b). |
| +0x10 to +0x18 | 3 × u32 | unused, zero | |
| +0x1C | s16 | alternate-text progress flag id | −1 in 163 rows. For a non-negative id, when the selector's preferred pointer at +0x24 or +0x28 is absent, the text lookup tests this progress flag and returns +0x2C when off or +0x30 when on. The 18 non-negative values range from 116 to 546. |
| +0x1E | u8 | class | 0 to 4. Meaning unknown. |
| +0x1F | u8 | important flag | 0 or 1. 1 in 115 rows (categories 7, 1, 2, 5 and 9: quest items and equipment). An important item can be collected from up to 25 units (Pickup, step 5) and is given to the party when its dropped copy ends (Important-item give). |
| +0x20 | u32 | description pointer | same convention as the name pointer |
| +0x24, +0x28 | u32 | selector-dependent text pointers | the lookup prefers one according to its selector argument; +0x24 is the first-find line, +0x28 the repeat line |
| +0x2C, +0x30 | u32 | progress-gated alternate text pointers | used by the +0x1C progress flag: +0x2C when clear, +0x30 when set |

Reading rule: the name and description are NUL-terminated strings at the pointer's target. Table 0 lookups for an id of 181 or more return nothing. Eighteen rows have no name pointer (ids 28, 48, 88 to 97, 134, 137, 142, 148, 158 and 159). For table-0 item text, the lookup first prefers pointer +0x24 or +0x28 according to its selector argument; if that pointer is absent and +0x1C is non-negative, the progress flag selects +0x2C when clear or +0x30 when set. Test vector: item-text row 2 has linked flag 116 and both progress-gated pointers; with the preferred pointer absent, flag 116 clear selects +0x2C and flag 116 set selects +0x30.

The rows match the config table. Cross-checks on disc: the rows for ids 19 and 20 are the two clover quest items, which are config type 16 with the matching quest ids. The rows for 135, 136, 138 and 139 are coins with prices 1, 5, 25 and 100. Row 137 (the 10 NP coin) has no name and a price of 0; its config record exists (type 0, zero stats). The value 10 that a coin pickup adds comes from the code, not from the text table. The category groups match config types: category 8 is config type 17 in all 14 rows (charms), category 6 is config types 12 and 15 (consumables), category 1 is config types 3 and 6 (weapon family), category 2 is config types 2 and 5 (shield family), and category 5 is config types 1 and 4 (body family).

### Placement id table (world pack type-15 chunk, tag 0x13)

Confidence: High for the layout; Medium-High for the ordinal mapping (test vector 64).

A world pack's type-15 chunk holds sub-records. The sub-record with tag 0x13 is the placement id table. The other sub-records of that chunk (tags 0x0A, 0x0B, 0x0D, 0x0E and 0x12) have their own layouts and are not covered here.

Sub-record header (little-endian):

| Offset | Size | Field | Notes |
| --- | --- | --- | --- |
| +0x00 | u32 | tag | 0x13 |
| +0x08 | u32 | data size in bytes | a multiple of 16 |
| +0x10 | data size | entries | count = data size / 16 |

Entry (16 bytes; placement id = entry index, from 0):

| Offset in entry | Size | Field | Notes |
| --- | --- | --- | --- |
| +0x00 | s8 | region byte | 0xFF (-1) = the world pack's embedded region. Any other value j names the first-created loaded region slot whose current slot load recorded j (SPEC-0006, Region slots and Tagged tables) |
| +0x01 | s8 | placement ordinal | index, in walk order, among the kind-8 records of that region's type-36 child |
| +0x02 | 14 bytes | unused | zero in the samples on disc |

An entry gives the placement that the id names: the region and the kind-8 record at that ordinal. The placement's key, group index, fixed count and flag are used as described in When a drop roll runs, Drop and money flow and Pickup dispatch.

## Behavior

### Item ids and categories

- An item id is a number from 0 to 322. Ids 0 to 207 have config records. Ids 0 to 180 have text rows. Ids 181 to 207 have a config record but no text row. Ids 208 to 322 have neither (open question 2).
- Item names (High): the name of item id n (0 to 180) is the text that row n of table 0 of resource 533 points to (name pointer at row +0x00; layout in "Item text database" above and in SPEC-0024). 181 ids have a row and 163 of them have a name. The 18 ids with a row and no name are 28, 48, 88 to 97, 134, 137, 142, 148, 158 and 159. Ids 181 and above have no row and so no name. Read the names at run time; do not copy them. Example: item 98 is "Wooden Sword".
- Pet type ids are a separate id space from item ids and never appear in the item lists. The petpet type ids used by the companion routines are 450 to 2094 (SPEC-0047, Petpets). The item range stays 0 to 322.
- Item categories (text row +0x04), with observed members: 0 (money and quest-style items), 1 (weapon family), 2 (shield family), 5 (body family), 6 (consumables), 7 (quest and story items), 8 (charms), 9 (one quest item).
- Mote status code by item id (High; SPEC-0036, Mote status words and Status apply): ids 160 and 161 map to fire (7); 162 has no status (-1); 163 maps to stun (0); 164 and 165 map to poison (2); 166 and 167 map to slow (3); 168 and 169 map to curse (1); 170 and 171 map to confusion (6); 172 and 173 have no status (-1). Any other id also gives -1. The status code is sent with the Mote attack's duration, amount and period values to the target's status-apply method. It is separate from the item text category: these Motes are charms, category 8. Example: id 164 (Leaf) gives code 2 (poison), while id 162 (Bubble) gives -1 (no status).

### Kind pick for ids 0xA0 to 0xAD (High for the rule and the values read; Medium for its use)

The kind pick takes an item id and a flag (zero or not) and gives two numbers. Both are zero for ids outside the range 0xA0 to 0xAD.

- Odd ids 0xA1, 0xA3, ..., 0xAD: first number 1, second number 2.
- Even ids 0xA0, 0xA2, ..., 0xAC: first number 1, second number 1.

The flag selects one of two copies of the values. On disc both copies hold the same values. The two values come from eight 4-byte words at 0x2DD490 to 0x2DD4AC, read as signed 32-bit integers, in address order: 1, 20, 3, 4, 2, 40, 1, 2 (an implementation reads them from there). Which system uses the two numbers is not identified (Open questions).

### Mote parameter lookups for ids 160 to 173 (High; SPEC-0024 Table 16)

The float lookups use the 60-value parameter table in archive resource 533, type-42 table index 16. It contains two 30-value hero blocks. Each Mote id has a power factor P and, where present, status duration D, amount A and period T. The status-field readers return -1 for an absent field or unsupported hero/id; the status receiver uses -1 for its default. The exact per-id field presence, table indexes and disc values are in SPEC-0024, Table 16. P is used by hero Mote attacks as described in SPEC-0036. The fixed Mote-id-to-status-code lookup is separate and is described above.

### Slot ids 50 to 59 and 98 upward (item display models)

These id rules belong to the item display models (Equipment display indexes and item display models). Ids 50 to 59 are the ten potion models shown when a potion lies in the world (index 0 to 9 = id minus 50, resource 515 + id). The second kind is the weapon model: display ids from 98, model resources from 544, at most 16 models. High.

### Equipment families and ranking

Id ranges by family (stats are not repeated here):

| Family | Types | Ids |
| --- | --- | --- |
| Swords and wands | 6 and 3 | Swords 98 to 105 (type 6); wands and rods 111 to 118 (type 3) |
| Body armour and dresses | 4 and 1 | Body armour 119 to 125 (type 4); dresses 127 to 134 (type 1) |
| Shields | 2 and 5 | 106 to 110 and 176 to 180 |
| Charms (no family) | 17 | 160 to 173 |

- The equipment types form three families: types 1 and 4, types 2 and 5, types 3 and 6. Each family holds one equipped item at a time.
- Within most types the stat A field rises with id (types 2, 4, 5, 6 and 1 up to id 133). Type 3 is the exception at its top id (stat A 50 at id 117, then 40 at id 118). It is not monotone across the whole family. Example: in the types 1 and 4 family, ids 119 to 125 are type 4 with stat A from 0 to 70, and ids 127 to 133 are type 1 with stat A from 0 to 70, with id 134 (type 1) having stat A 0.
- A pickup of equipment type (1 to 6) is refused when its family already has an equipped item and the pickup id is less than or equal to the equipped item's id. Otherwise it is allowed. The comparison uses ids only, not stats or types. See Edge cases.

### Consumable healing

Consumables are items with a non-zero pool A or pool B percent (stats E and F). On disc these are the type-12 items, plus the two full-restore items (ids 58 and 59, which have no percent fields and are treated as 100%).

Equipment (types 1 to 6 and 17) reads zero for both stats, so equipment never heals.

Table entries for the two heroes: pool A uses T = 500 for hero 0 and T = 400 for hero 1; pool B uses T = 300 for hero 0 and T = 500 for hero 1. Let T_A(c) be the pool A table entry for hero c, and T_B(c) the pool B entry. Let p_A be stat E and p_B be stat F of the item, as percent.

- Pool A heal = trunc(T_A(c) × p_A / 100 + 1). For id 58, use p_A = 100.
- Pool B heal = trunc(T_B(c) × p_B / 100 + 1). For id 59, use p_B = 100.
- The index c is the active hero (0 or 1).
- A pool is healed when the heal is smaller than the missing amount (missing = maximum − current), or when the use is forced and the missing amount is above zero. The result is current = min(maximum, current + heal). The quick-item panel always forces the use. The item screen path also forces the use (Open question 89).
- The use is applied only to a pool whose percent is above zero, or where the item is 58 (pool A) or 59 (pool B).

Pool A is the health meter and pool B is the magic (MP) meter of the active character. Their current and maximum values live in player state, not in the item data.

### Consumable effects by item id

Status indexes: 1 curse, 2 poison, 3 slow, 4 strength, 5 speed, 8 immunity to curse, 9 immunity to poison, 10 immunity to slow, 12 health regeneration, 13 magic regeneration (the same indexes as the status order in SPEC-0025, Status timers). Statuses 4, 5, 12 and 13 last 60 seconds by default. Immunities 8, 9 and 10 last 30 seconds by default on the script route (SPEC-0036, Status apply routes; SPEC-0057 agrees).

- 50 (Potion of Revival): no percent heal. Its only effect is the partner revive. The active hero (0 or 1) revives the other hero when that hero is dead and the game's mode word is 2. The partner's state is set to idle and its current health to its maximum, then a switch request to that hero is stored (the request must be accepted). Success consumes the item and plays speech line 0x0F. When any step fails, the use is refused (cue 0xA2). Item 50's text row names it "Potion of Revival" (category 6, price 100). Its config record is type 15 with all stat fields zero (High for the record; Medium-High for the revive).
- Item 50 is also a gate of the game-over call: in level modules 1122, 1217 and 1428 the game-over call runs only when the hero holds no item 50. Module 1432 calls it only inside its scripted ending, with no item test (SPEC-0055) (High for the three gates).
- 51 (Spyder Juice Elixir): applies status 9 (immunity to poison) when status 9 is inactive. If status 9 is already active, the use does not refresh it: the use is refused with cue 0xA2 and the item is not consumed (Active immunity potion; High).
- 52 (Dispelling Potion): applies status 8 (immunity to curse) when status 8 is inactive.
- 53 (Ointment of Quickness): applies status 10 (immunity to slow) when status 10 is inactive.
- 54 (Unguent of Curing, a cure): succeeds only if at least one of statuses 1, 2 or 3 (curse, poison, slow) is active. On success it clears all three statuses and plays cue 0x9F, with no voice line. If all three are inactive, use is refused with cue 0xA2 and the item is not consumed (Cure with no active status; High).
- 55 (Potion of Power): applies status 4 (strength) when status 4 is inactive.
- 56 (Potion of Meerca Speed): applies status 5 (speed) when status 5 is inactive.
- 57 (Starlight Potion): places the single item-effect object at the point 2.0 units ABOVE the hero's position (offset (0, 2.0, 0) along world up; not ahead of him; High). The point is stored once at the use and the object does not follow the hero. The game keeps one such object. It is a world effect object, not an item: it carries effect record 4020 as its main effect and effect record 568 as a second effect, both bound the first time it is placed. The use stores the point and sets a 60-tick timer (only when the timer is idle, below 0). Neither effect is playing yet: both effects START on the tick the timer reaches 0, one second after the use, and that tick also clears the object's hit list. The free test fails while the timer is 0 or above, and afterwards while the main effect's particle is still alive (about 41 more ticks, see the hit volume below), so the potion cannot be used again for about 100 ticks. When busy, the use is refused with cue 0xA2 and the item is not consumed. On success the cue is 0xA0 with voice line 0x26 (Tor) or 0x3E (Roberta) (Starlight Potion busy case; High).
  - Starlight Potion hit volume (High for the structure and the damage call, Medium-High for the geometry). While the object is active it tests a vertical cylinder of full height 4.0 (interval stored point y - 2.0 to y + 2.0, so from the hero's feet to 4.0 above them) centred on the stored point, once per tick after the hit list's count-down. It is active while the timer is 0 or above, and then while the main effect's particle exists. Radius: 0 during the 60 timer ticks (no particle yet); after the effects start it is the size of the particle born by effect 4020, which is the same emitter as the Tor Water particle of SPEC-0036 (one burst of 3 particles, life 0.7 s): R = 0.5 + 14.5 x age / 0.7 with age in seconds, 0.5 at the first tick, about 14.3 at age 40 ticks, and the object ends on the first tick where age + 1/60 reaches 0.7 (41 active ticks; Medium for plus or minus 1 tick). The second effect's ring radius is set to the same R every tick it runs, so the drawn ring grows with the hit volume.
  - Starlight Potion targets and damage. A character overlaps when the horizontal distance between its body centre and the stored point is at most its body radius + R and its body interval (feet to feet + height) overlaps the cylinder's interval (touching counts); the shared character test of SPEC-0036 family A. Every overlapping eligible character is entered in the object's ten-slot hit list (first free slot, timer 60) and is skipped while it is listed, so a character is hit at most once per 60 ticks, and the list is cleared when the effects start (a character standing on the point during the first second and surviving can be hit again by the growing ring). A listed character whose spawn-record key (the s16 at data +0xAC of its kind-3 record) is one of the Minion-corps keys is also given a hit: its take-hit method is called with damage 100, attacker -1 (no attacker), blocked flag 0, and the last two arguments 2 and 0. The Minion-corps keys are 2801 to 2807, 2811 to 2817, 2821 to 2827, 2831 to 2837, 2841 to 2847, 2851 to 2857, 2861 to 2867 and 2871 to 2877 (56 keys; 646 placements on disc, of the Minion Artillery, Brute, Grunt, Magician, Warrior and Wizard kinds and one more unnamed Minion kind). The take-hit is the common one (SPEC-0038 Behavior 37): damage 100 is doubled to 200 when the target is not already in class state 19, then subtracted from health unless the target is undamageable; a target at 0 or below dies normally with its drops. No other character is damaged: the hero, townsfolk, petpets, bosses and ordinary enemies are only listed. The potion has no healing, status or knockback effect.
  - Starlight Potion and breakables. The same cylinder also breaks intact world breakables listed for the grid cell that holds the stored point, whatever their element (the object's element field is 7, which passes every element gate; SPEC-0040 Behavior 16), with the usual 0.25 piece push and the material sound cue; script-registered breakables run their script instead.
  - Starlight Potion worked example (Medium-High). Hero at (100, 5, 200) drinks the potion: the stored point is (100, 7, 200) and the cylinder spans y 5 to 9. A Minion Warrior (radius 1.25, height 3.0, 100 health) with its feet at y 5, horizontal distance 8.0 from the point, in class state 19. The effects start 60 ticks after the use. On the ring's age-18 tick R = 0.5 + 14.5 x 0.30 / 0.7 = 6.714, and 6.714 + 1.25 = 7.964 < 8.0: no hit. On age 19, R = 0.5 + 14.5 x (19/60) / 0.7 = 7.059 and 7.059 + 1.25 = 8.309 >= 8.0: hit, health 100 - 100 = 0, the Warrior dies. The same Warrior not in state 19 takes 200 and dies on the same tick. A Warrior 16.0 away is never hit (largest R about 14.3 plus 1.25). The hero standing on the point is listed but takes nothing.
  - Default if unresolved (rewrite choice): start both effects on the potion tick on which the decrement brings the timer to 0, set the particle age to 0 on the next tick, and treat ages 0 to 40 as active.
- Items 51, 52, 53, 55, 56 and 57 play cue 0xA0 on success, with a voice line of 0x26 for hero 0 or 0x3E for hero 1.
- 58 (Kauvara's Potion): full restore of pool A (percent 100). It applies status 12 (health regeneration) before the heal check, so the status is applied even when the heal is refused.
- 59 (Fyora's Potion): full restore of pool B (percent 100). It applies status 13 (magic regeneration) before the heal check, under the same rule as 58.
- 145 (Bagguss): its stat D is 1, so it applies status 9 when status 9 is inactive, under the rule below. Success cue 0x9F, no voice line, no heal (its pool percents are 0).
- Rule for status 9: any non-equipment item with stat D above zero applies status 9 under the same test. On disc only item 145 has stat D above zero.
- 126 (Tattered Cloak): type 15, category 9, no use effect. The use is refused (cue 0xA2) and nothing is consumed. It is worn with Square on the Items tab, not used (Tattered Cloak).
- Charms (ids 160 to 173) are not consumed by the use step. A charm use runs the equipped slot routine instead.
- Immunity checks: a curse (1) is not applied while curse immunity (8) is active, and poison (2) is not applied while poison immunity (9) is active.

Status test direction (settled): a buff or immunity applies only when its status timer is zero or below (inactive). The status test returns true when the timer is above zero (High; Open question 33). Items 51 to 56 and 145 are the buff and immunity ids; item 54 is the cure.

A use is consumed only when it succeeds. A refused use is not consumed, even when a regeneration status was applied first (items 58 and 59 at full health).

### Heal streak

Medium-High. Heals of pool A and pool B are accumulated per hero (heroes 0 and 1 only) and per pool, together with the time of the last heal of that pool.

1. A heal of amount H at time t: when t minus the time of the previous heal of the same pool is strictly less than 4.5 s, H is added to the accumulator. Otherwise the accumulator restarts at H. The time of that pool is then set to t.
2. After the heal, compare the accumulator with half of the pool's current maximum. When it is at least half, it resets to 0 and the reaction id below plays. A stat-up add that increases a pool also updates that pool's current and maximum values; a later consumable heal therefore uses the updated maximum for this comparison. The stat-up path does not run the consumable heal handler's accumulation and comparison.

Reaction ids (Medium-High): 0x23 for hero 0 pool A, 0x3B for hero 1 pool A, 0x24 for hero 0 pool B, 0x3C for hero 1 pool B. The 4.5 s window is in game time (60 ticks per second). A heal at exactly 4.5 s restarts the accumulator (Open question 34).

### Use cues

Medium-High. A successful heal plays cue 0x9F, or 0xA0 for items 58 and 59. When the pool's percent is above 10 (items 58 and 59 always qualify), an extra cue plays after it: 0x9B for pool A, or 0x9C for pool B (pool B replaces pool A's cue when both apply). A successful buff, immunity or placement (items 51 to 53 and 55 to 57) plays 0xA0, with the voice line described in Consumable effects by item id. Items 54 and 145 play 0x9F. A refused use plays cue 0xA2 and is not consumed (Item screen use).

### Quick-item selection marker (Medium-High)

Layout in the party record (byte offsets):

| Offset | Size | Meaning |
| --- | --- | --- |
| +0x155C | 4 | Selection marker: an item id, or -1 when empty. It is an item id, not an entry index. |

The words at +0x1544 + 4k, k = 0 to 2, are the three equipped slots, not quick-item slots (Equipment slots).

Rules:

1. Setting the marker to a different item id plays cue 0xA3. Setting it to the id it already holds does nothing. Clearing it to -1 plays no cue.
2. Effective marker: the marker's id when the party's count of that item is 1 or more, otherwise -1.
3. A use of the marker item that leaves its count at 0 clears the marker to -1. The entry keeps its id until it is refilled.
4. The item screen's Circle (Items tab, SPEC-0035 section 14.2) is the marker itself: when the marker equals the cell's id it clears the marker to -1, otherwise it sets the marker to the cell's id (rule 1 applies, so cue 0xA3 plays when the set succeeds); the screen also plays cue 0x9A. Clearing the marker is its only effect; no equipment word changes (High; Medium-High that the id set is the cell's id).

5. A changed marker is accepted only when the item-manager lookup for that id returns a record. On success the setter writes the id, updates three shared panel-control words to 240, and plays cue 0xA3; it does not alter inventory or equipment lists. A failed lookup leaves the marker unchanged. The visible purpose of the three control words is not established (High for the setter effects; Medium for the item-manager lookup identity).

### Quick-item panel (pad trigger; Medium-High)

The quick-item panel is a pad trigger, separate from the item screen (SPEC-0035). It has four entries, numbered 0 (Right), 1 (Up), 2 (Left) and 3 (Down). The selection is an entry number from 0 to 3, or 4 for no selection.

Candidate lists. Each entry has a fixed candidate list of item ids, read in order from the boot data (Where it lives):

- Entry 0 (Right): 145, 54, 51, 52, 53.
- Entry 1 (Up): 150, 146, 156, 58.
- Entry 2 (Left): 153, 155, 152, 57, 56, 55, 50.
- Entry 3 (Down): 149, 157, 147, 59.

Refill (High). The entry shows its current candidate. Each list is stored as a row of 8 slots ending in -1 (Where it lives). One refill makes up to 8 steps, and each step changes the entry's value as follows:

1. An entry of -1 takes the row's first id.
2. Otherwise the step finds the entry's id at position i of the row. When i is below 7 it takes the slot at i + 1, which is -1 after the last id; when i is 7 it takes slot 0. An id that is not in the row is kept unchanged.
3. The refill stops when the new value is 0 or more and the party owns it: the party's item record for that id exists, its count is above 0 and its first word is nonzero. This ownership test is the only acceptance test. Otherwise the next step runs.

So wrapping past the last id costs one step (the step that lands on -1), and a list of n ids repeats every n + 1 steps. Repeating a direction cycles through the owned candidates in list order. When no candidate is owned, all 8 steps fail and the entry ends on the value the eighth step reached, not on the value it held before: that can be an unowned id or -1. Its cross icon is then hidden, because the icon shows only while the entry's count is above 0 (SPEC-0023 section 11). An entry whose id is not in its row stays unchanged through all 8 steps.

Opening. An Up edge with the panel and the second panel both closed opens the panel, when the in-play gate passes: the hero's action state is not 0x19, a hero predicate is false, and two game-global words are zero (SPEC-0053). Opening plays cue 0x9A, refills the four entries in the order 2, 1, 3, 0, and sets the selection to entry 1. A Down edge with both panels closed opens the second panel instead (cue 0x9A; Second quick panel and Mote equipping). Once the panel is open, the in-play gate no longer applies to its direction, select, use or close actions.

Direction. While the panel is open, a direction edge sets the selection to that entry: Right 0, Up 1, Left 2, Down 3. When several directions are set, Left has priority over Up, Down and Right. A direction edge plays cue 0x9A. A direction edge equal to the current selection refills that entry (see Refill) and plays cue 0x9A. A tick with no direction keeps the selection.

Select and use (panel open, with a selection):
- Cross sets the marker to the entry's current id when the party owns that item and the id differs from the marker (cue 0xA3). The panel closes. The use step then runs on the marker. If the entry's current candidate is not owned, the marker is not updated and the use runs on the old marker.
- Circle sets the marker in the same way (cue 0xA3) and closes the panel. Circle does not use an item.
- Cross or Circle with no selection does nothing; the panel stays open.
- Triangle closes the panel. It does not change the marker, does not run a selection step and does not use an item.

Triangle with the panel closed runs the use step on the current marker. The marker is not changed. With an empty marker nothing happens.

Use step (Cross and Triangle). The use is refused (cue 0xA2) when the hero is dead or when the charm-only gate applies and the id is not a charm. A use that a level script's use hook takes is not applied (Use-request gate and charm-only flag). The two gates are:
- Charm-only gate: while a game-global word is nonzero, heroes 0 and 1 can use only the charms (ids 160 to 173). The word is set by the script call SetItemBlockMode(bool) (Use-request gate and charm-only flag).
- Use hook: while a level script has registered a use-item hook, the use is handed to the hook instead of being applied (Use-request gate and charm-only flag).

A use is forced, so the heal rule of Consumable healing applies with the forced branch. A use that succeeds runs the consumable effects of Consumable effects by item id and removes one item from the party's count. When the count reaches 0 the marker is cleared (Quick-item selection marker).

See SPEC-0033 Controls for the button rows. The item-screen trigger is the Cross press on a cell (Item screen use, above).

### Second quick panel and Mote equipping

Confidence: High for the opening, the direction rule, the table layout, the panel text, the timer re-arm, the equip tests and the equip cue; Medium-High for the button rules and the slot kind arguments; Medium for the icon tween motion.

The second quick panel is a Mote picker. It selects one of the Motes (see Motes and elements) or "No Mote", and equips the selection into one of the three equipment slots. It is not a family picker.

Opening:

1. A Down edge opens the second panel only when both quick panels are closed and the in-play open gate passes (SPEC-0053). Opening plays cue 0x9A.
2. Once it is open, the second panel's handler runs each tick while the first panel is closed. The in-play open gate is not applied to this handler.
3. While the first panel is open, a Down edge selects entry 3 of the first panel instead (Quick-item panel).

Contents:

4. The panel lists the 14 Motes, ids 160 to 173, and "No Mote". Selection values 0 to 13 are the Motes (the Mote id is 160 plus the value). The value 255 is "No Mote". The selection is 255 at boot.
5. Panel text (High). On every handler call the panel's label buffer and the two item-screen name texts (name and description) are first reset to a single space. Then, when the selection is not 255, the hero's owned count of the selected Mote is written, and when that count is above 0 the Mote's name and description are written as well. So a selected Mote that the hero does not own shows its count with a blank name, and "No Mote" (255) shows a blank label. The text "No Mote" is only the boot content of the label buffer: it shows until the first handler call after boot and never again.
5a. Each handler call re-arms the equipment-panel timers G3, G4 and G2 of SPEC-0023 section 4 (set to 240), so the three equipment panels stay fully visible while the Mote panel is open (High).
5b. In the HUD the panel is group node 392, drawn while the Mote panel is open. Its label text is at canvas (30, 404). Mote k's icon is shown while the hero owns it, and the highlight while the selection is k (SPEC-0023 section 24; High).

Movement:

6. A direction edge moves the selection through a direction table in boot data at address 0x2DCE58. The table has 15 rows of 8 bytes (0x2DCE58 to 0x2DCECF), one per selection value: the row used is (selection + 1) modulo 256, so selection 255 uses row 0 and selection k (0 to 13) uses row k + 1. The column is the direction code of item 7. The byte at that row and column is the new selection. Every byte in the table is 0 to 13 or 255, so no move produces a selection of 14 or more and no read goes past the table (High; Open question 103, Resolved).
7. The direction code uses the first matching rule:
   - Up and Left both held: column 4 (Up+Left).
   - Up and Right both held: column 5 (Up+Right).
   - Down and Left both held: column 6 (Down+Left).
   - Right and Down both held: column 7 (Down+Right).
   - Left not held and Right held: column 1 (Right).
   - Left and Right not held: column 2 (Up) when Up is held, including Up with Down; otherwise column 3 (Down).
   - Left held otherwise (Left alone, or Left with Right): column 0 (Left).
   The columns are, in order: 0 Left, 1 Right, 2 Up, 3 Down, 4 Up+Left, 5 Up+Right, 6 Down+Left, 7 Down+Right.
8. The selection is not limited to owned Motes. A direction can rest the selection on a Mote that the hero does not own, or on "No Mote". The move does not check ownership.
9. Every direction edge plays cue 0x9A.

Buttons:

10. Button presses take priority over direction. When Cross, Square or Circle is pressed in a tick, the direction is ignored in that tick, even when the press equips nothing.
11. Triangle is checked first. It closes the panel and clears the panel's open flag. It plays no cue and has no other effect.
12. Cross equips the selection into slot 0, the armour Mote word (Slot 0: armour Mote). No equipment object test applies. When Cross is pressed, the Square and Circle rules are not checked.
13. Square equips the selection into slot 1, the shield Mote word, only when the hero's category-2 inventory slot (the shield family) is not empty (count 1 or more). If Square is pressed and that test passes, Circle is not checked. If the test fails and Circle is also pressed, the Circle rule applies. Otherwise nothing happens.
14. Circle equips the selection into slot 2, the weapon Mote word, only when the hero's category-1 inventory slot (the weapon family) is not empty (count 1 or more). If the test fails, nothing happens. The kind arguments are the category numbers of the weapon and shield families (Open question 95, Resolved, Medium-High).

Equip or clear for the target slot k (s is the selection):

15. If s is 255, the slot is cleared.
16. If the Mote with id 160 plus s is held (its inventory slot is found and not empty, count 1 or more) and its id is not already in slot k, the Mote is equipped into slot k (Equipment slots, equip rule; High).
17. Otherwise the slot is cleared. Clearing sets the slot to -1. For hero 0 (Tor), a clear of word 1 or 2 stops the matching Mote glow. If the slot held an item, cue 0xA4 plays.
18. Pressing the Mote that is already in slot k therefore clears slot k.
19. After any equip or clear from the panel, the HUD equipment icons are refreshed and two HUD icon tweens may run (High for the triggers; Medium for the motion): on an equip, the new Mote's icon flies from its panel cell to the slot's icon position at 800 canvas units per second (word 0x2DCE2C); when the slot held a Mote with a count above 0 before the press (replaced or cleared), that old icon flies out from the slot at 100 units per second (word 0x2DCED0). The tween pool and its motion are in SPEC-0023 section 11.1. This is not the portrait swing of SPEC-0023 section 12.

Item screen (SPEC-0035 section 14.1): the same picker runs on the pause item screen's Charms tab. Its input there is the menu block, which has the pad-record layout with each word the OR of all plugged pads: directions come from the newly-pressed word that includes the left-stick direction bits, and buttons from the newly-pressed button word. In play the picker reads directions and buttons from the hero's own pad record's newly-pressed button word, so only the D-pad steps it (High). Its selection is the same value as the in-play panel's, so a choice made on either is seen on the other (High).

Test vectors (from the boot direction table, selection values in brackets): with no Mote selected (255), Left gives Mote 172 (12), Right gives Mote 173 (13), and Up+Left gives Mote 162 (2). From Fire (Mote 160, selection 0), Right gives Mote 161 (1).

### Panel visibility and clears

Confidence: High unless marked.

1. Both quick panels read as closed while the camera mode word is nonzero. The camera mode is the mode field of the camera record (SPEC-0027). It is not a menu or pause flag. Any nonzero mode hides both panels. The camera sets modes 8, 9, 10 and 12 (SPEC-0027). In play only mode 8 occurs: the script call SetHeroSceneCamera sets it (one level module on disc, pack 1250), and so does the hero death step when the dead hero's partner cannot take over (the death camera at distance 15.0). Mode 10 is set only by SetAI_DistCamera and SetHeroDistCamera, which no level module calls, and modes 9 and 12 have no entry that runs. ReturnToNormalCamera and SetToNormalCamera return modes 8 to 10 to 0. So the panels are hidden during the pack-1250 scene camera and after a hero death with no partner to take over and no respawn point, until a return to normal.
2. Mutual exclusion. Opening the first panel sets its flag to (second flag = 0), and opening the second sets its flag to (first flag = 0), so both flags are never set together.
3. Hidden open. The camera mode does not clear the panel flags. While the mode is nonzero, both readers report closed, so an Up edge that passes the in-play gate runs the open again: the first flag stays set (or is set), cue 0x9A plays, the four entries step to their next owned candidates and the selection becomes 1. A Triangle edge in that state uses the marker. The panel shows when the mode returns to 0 (High).
4. A third reader of the second flag is front-end predicate slot 37, the Mote panel's draw gate: the second flag is set and camera word +0x50 is 0 (slot 32 is the first panel's gate).
5. A panel flag is cleared by:
   - The in-play state, when Select or Start opens a menu and the busy test below is false. Select opens the pause page and Start opens the item screen. The menu opening plays cue 0x94 and clears both flags.
   - The quit-hold state. It clears both flags.
   - The overlay box: each in-play pass, when the overlay box is busy, both flags are cleared (set or not) and the state goes to the box host (code 0x17C, state 23; SPEC-0035). Normal play reaches it whenever a script opens the box.
   - The HUD reset, the body of the FrontendHUDReset script call (one call on disc, in module 1176). It clears both flags, the prompt-slot counts, every HUD timer (SPEC-0023 section 4.1) and the boss-bar timer. On disc it runs at: every scripted camera play start of the PlayCamera family (PlayCamera, PlayTrackHeroCamera, PlayLookAtPlayerCamera, PlayTrackAICamera, PlayLookAtAnimObjCamera and PlayTrackAnimObjCamera all share one start; the play without a target and the play with a target both reset), the open of an element-sealed chest (type-40 kind 2), a dig (kind 6) and the script call itself. A PlayCamera-family call whose track index is below 0 or at or above the world's track count returns before starting anything and does not reset. SetAiHotspotDistCamera (a one-key camera play start that also resets) is never reached: no level module imports it. The static head-pose play start and the look-at play start also reset, but nothing calls them (no caller, no table entry), so they never run. The module 1176 call is on the arena win path: in the death watcher, after the opponent has died with the hero alive, once the win line has been shown and the two blackboard messages added, just before the load back to the arena master's world (SPEC-0041, 1176). The hero-death branch of the same watcher does not reset. The same reset also runs from the in-play Select and Start openers, and from the quit-hold state's first pass. A rewrite that resets at the camera play start, the sealed chest and the dig, and keeps the 1176 call, the menu openers and the quit-hold reset, matches the disc. Confidence: High.
   - Triangle on the second panel. It clears the second panel's flag only.
   - The first panel's own clear, which clears the first panel's flag only. The selector calls it on Cross or Circle with a selection (after setting the marker) and on Triangle. The hero tick calls it whenever game-globals +0x20 or the game-state code (game-globals +0x5C) is nonzero; the selector does not run then.
   - The second panel's second clear routine has no reference at all and never runs.
6. Select branch. The pause page opens when Select is pressed, and also with no Select when pad slot 0 is unplugged or not ready (the pad-slot-0 usable test; SPEC-0035; SPEC-0033 auto-pause). The busy test is true when any of these hold: game-globals +0x20; +0x14 > 0; +0x24 and +0x28 both set; +0x2C set; +0x44; the active hero's state is 0x19, 0x21, 0x32, 0x33, 0x34 or 0x37; either loaded world's script object has its +0xD0 or +0xEC word set; +0x34 bit 0 set with +0x38 ≥ 0.10000001 and the three objects at 0x35C83C, 0x35C840 and 0x35C844 all absent or idle (+0x10 = 0); +0x3C set.
7. Pad mask. While either panel flag is set, and on the one tick after both clear, the hero's own control step sees its pad words with the D-pad and face buttons (mask 0xF0F0) removed; a masked button held across the close stays blocked until it is released, and the analog stick is not masked (SPEC-0033 Behavior 1A). Prompt captions and the Square-press interactable prompts are also suppressed while a panel is open.

### Use-request gate and charm-only flag

Confidence: High for the hook path, the consumption rule and the charm rules; Medium-High for the visible result of the default replay.

1. The item-use dispatch takes a hero, an item id, a force flag, a notify flag and a gate flag. When the gate flag is 1, the use hook is checked first. The quick panel's use step (force 1, notify 1), the pause-menu item screen's Cross (force 1, notify 0) and the item screen's scrolling list (force 1, notify 0) all pass gate 1.
1a. Notify flag (High). After a successful consumable use (one that returns 1), a notify flag of 1 starts the item's use effect at the hero's position: one pooled effect object of the use-effect group, from effect bank 3 with a key fixed by the item id (2000 to 2025 for ids 19, 20, 50 to 59, 145 to 147, 149 to 157, 174 and 175; any other id has no key and nothing plays). With notify 0 no use effect plays. So a heal from the quick panel shows the potion effect on the hero, and the same heal from the item screen shows none. For item 50 the notify value is also passed to the swap request that follows the revive, as both its fade word and its pad-cue word (SPEC-0053): from the panel the swap fades with cue 0xB0 and sends the pad cues; from the item screen it switches with no fade and no pad cue.

2. The use hook is a callback that a level script registers with RegisterHeroUseItemCallback(callback, handled) (host row 269, SPEC-0030). It lives in the current level's script manager and is cleared when that manager is reset for a new level. While no hook is registered the use goes on. While one is registered, the use is diverted: the item id is stored in the hook's request block, the block's handled word is set to the registration's handled value, a hook thread is started if none is running, and the dispatch returns 0 at once. Nothing is healed, removed or consumed by the game. A second use while the thread still runs only overwrites the stored id.
3. On the item screen a 0 return plays cue 0xA2, so a diverted use plays the refusal cue there. The quick panel removes nothing when the dispatch returns 0.
4. The hook thread calls the callback with the request block: word 0 the item id, word 2 the handled word. When the callback returns and the handled word is 0, the thread makes one default dispatch for the active hero with id 0, force 0, notify 1 and gate 0. Id 0 (Signed Letter of Consent) has no use effect, so that dispatch plays cue 0xA2 and changes nothing; the stored id is not replayed. The thread then ends, and the manager's per-tick pass frees its handle so the next use can start a new one.
4a. Hooks on disc (High): three. Pack 1176 registers one, handled 1, in its fight-start routine; its callback plays main sound 142. Pack 1350 registers one, handled 0; its callback returns at once. Pack 1371 (the Coliseum) registers one, handled 1, while blackboard value (2, 157) is 20; its callback returns at once. None of the three callbacks reads which item was used. Packs 1176 and 1371 pass handled 1, so the default dispatch never runs in them. Pack 1350 passes handled 0, so each use of an item there is followed by one default dispatch of id 0 after its callback, which plays cue 0xA2. In all three places every quick-panel and item-screen use is blocked.
5. The pause-menu item screen forces heals, the same as the quick panel: a pool is healed up to the missing amount when the missing amount is above zero (Consumable healing, forced rule).
6. When the dispatch returns 1, the use removes one of the item from the party's count (the inventory remove with count 1). This is the same on both paths. The quick panel then clears its marker when the count reaches 0 (Quick-item selection marker). The item screen has no marker.
7. Charms. For charm ids 160 to 173 the dispatch returns 0. The charm's slot routine runs only when that charm is equipped. No charm is consumed. On the item screen a charm use plays cue 0xA2.
8. Charm-only flag. The script call SetItemBlockMode(bool) (SPEC-0030) stores its argument as a 32-bit word at offset +0x48 of the game-globals object. The word is zero at boot. While it is nonzero, heroes 0 and 1 can use only the charm ids 160 to 173 through the consumable path. Any other id is refused: cue 0xA2 and no removal. The check runs at the start of the consumable use, before any effect. No level module on disc calls SetItemBlockMode, so the word stays 0 and this rule never applies in the shipped game (High; Open question 108).

### When a drop roll runs

Confidence: High for breakables and scripted adds and for the order of the enemy death route; Medium-High for its money-roll details.

The drop action (the money roll, the coin split and the item roll of Drop and money flow) runs only on the events below. Region load never rolls.

1. **Breakable break.** At the break tick, the breakable's drop id is read as a placement id (Placement id table). A negative drop id does nothing. If the placement's key is below 0, the drop action runs on that placement at the break position. Otherwise its key spawns directly as the item.
2. **Enemy first death tick.** The roll runs once, on the first tick of death. Later hits do not repeat it. The parts run in this order:
   A. Direct item. If the enemy's direct-item word (set only by a character activation; see Sources of the enemy drop words in Open questions) is 0 or more, that item spawns at the enemy's position and the word becomes -1. Part B is then skipped.
   B. Placement. Otherwise the enemy's spawn record holds a drop placement ordinal (a signed 16-bit value at record data offset 0xB0; SPEC-0006). It is read as a placement number in the enemy's region. An ordinal of -1, or a value outside the region's placement list, does nothing. Otherwise:
      i. Key 0 or more: that item spawns at the enemy's position directly. The placement's fixed count is ignored.
      ii. Key -1: the drop action (Drop and money flow) runs on that placement at the enemy's position. The amount M is the group's money amount when the group's money roll succeeds. Otherwise M is the placement's fixed count, but only when it is above 0; otherwise M is 0. If M is above 0, it is split into coins (100, 25, 10, 5, 1), an exact greedy split, and no item is rolled. If M is 0, the group's item roll runs and an item may spawn. A group of -1 has no money roll and no item roll, so only the fixed count can pay coins.
   C. Death countdown. The enemy's death disappear countdown is set from the monster row's death timer. This runs on every first tick, after A or B.
   D. Carried Neopoints. The enemy's carried Neopoints word is split into coins (100, 25, 10, 5, 1, exact greedy split) at the enemy's position, and the word is set to 0. This runs on every first tick, after A or B, whatever A or B paid. The word grows only in the item steal branch of an attack, by the stolen item's catalogue price (text row +0x0C; SPEC-0029, Behavior 11b).
   Examples: a direct item 172 with carried Neopoints 30 spawns item 172, then coins 25 and 5 (the ordinal is not read). An ordinal whose key is -1 and whose group is -1, with fixed count 150, pays coins 100, 25 and 25. A key -1 placement whose money roll gives 12 with fixed count 150 pays coins 10, 1 and 1 (the fixed count is not used). If the carried value is then 7, part D adds coins 5, 1 and 1.
3. **Scripted add.** A scripted add of a placement rolls only when its key is negative: the item roll only, with no money roll and no coins. A key of 0 or more spawns that item directly, as described in Pickup dispatch. The scripted add applies the luck gate and the vetoes of Pickup dispatch before the roll, by the same rule as region start-up (luck draw first, then for keys 0 or more the equipment and quest-style vetoes). Routes 1 and 2 (the drop path) skip them.
4. **Region load.** Never rolls. Region load spawns each flag 0 or 1 placement's key as the item id (Pickup dispatch).

The percent roll on an enemy projectile hit is the projectile's status roll (Enemy projectile hits; SPEC-0029), not a drop: it spawns no item. The bounce with a 1 s bounce cooldown also belongs to the enemy projectile pool (SPEC-0029), not to world items.

### Drop and money flow (placement with a negative key)

1. Money roll: draw r = a uniform integer in 0..99 (100 values). If r is less than the group's money chance, the money amount is a uniform integer in [money min, money max], both ends inclusive.
2. If the money roll fails, the placement's fixed count (its stat at placement +0x2A) is used as the amount when it is above zero.
3. If there is an amount above zero, it is split into coins: count of 100 first, then 25, 10, 5 and 1, each count being the amount divided by the denomination, with the remainder carried on. Each coin is one world item: 100 gives item 139, 25 gives 138, 10 gives 137, 5 gives 136 and 1 gives 135. Picking one up adds its value to the wallet (Coin pickups). All coins of one split spawn at the same point. When the caller gives a launch direction, each coin waits 0.4 s longer than the one before it before it launches (delays 0, 0.4, 0.8 s and so on); with no direction every coin launches at once in its own random direction (World items, Spawn routes). No item roll happens after coins were spawned.
4. If there is no amount, the item roll happens. Let R = 100 − chance. Draw t = a uniform integer in 0..R−1 (a random integer modulo R). Walk the group's entries in order, accumulating weights: the first entry whose cumulative weight is greater than t is chosen. If t is beyond the cumulative total, no item drops.
5. The chosen sub-table is walked the same way with u = a uniform integer in 0..99 and the sub-table's weights. The item whose band contains u is spawned, one item.

Probability of coins and items per group (derived from the tables above):

| Group | P(coins) | P(one item) | Notes |
| --- | --- | --- | --- |
| 0 | 0.50 | 0.50 | entry (5, 10) can never be chosen |
| 1 | 0.40 | 0.60 | |
| 2 | 0.40 | 0.30 | 30 of 60 item-roll values drop nothing |
| 3 | 0.30 | 0.70 | |
| 4 | 0.35 | 0.65 | |
| 5 | 0.50 | 0.00 | money only |
| 6 | 0.25 | 0.50 | 25 of 75 item-roll values drop nothing |

A placement whose group index is −1 (394 of the 2,360 negative-key placements on disc; see Sources) gets no group money and no item. Only its fixed count can produce coins.

The group chances, the money ranges and all entry and sub-table weights are fixed data values. No hero stat, no luck value and no difficulty setting changes a weight or a roll on this drop path. The luck gate of Pickup dispatch is not applied on this path.

### Stolen Neopoints split into coins (High)

Confidence: High. This is the coin split used when an enemy's steal attack takes Neopoints (the Neopoint steal is Behavior 11a of SPEC-0029). The coins are spawned at once at the hero's position, all at one point. This split is separate from the drop split above. Let R be the amount still to be paid, starting at the stolen amount. While R is above 0:

1. R of 100 or more: spawn a 100 coin, then R falls by 100.
2. R of 25 to 99: spawn a 25 coin, then R falls by 25.
3. R of 10 to 24: spawn a 10 coin, then R falls by 10.
4. R of 5 to 9: spawn a 5 coin, then R falls by 5.
5. R of 1 to 4: spawn a 1 coin, then R rises by 1 (the step is -1, subtracted).

The loop ends at exactly 0 for every amount from 1 to 200. Step 5 pushes R up to 5, which then pays one 5 coin. Coins therefore total the stolen amount when it is a multiple of 5. Other amounts are paid as more than they were taken:

- 10 taken: one 10 coin. Total 10.
- 7 taken: a 5 coin, three 1 coins, then a 5 coin. Total 13.
- 11 taken: a 10 coin, four 1 coins, then a 5 coin. Total 19.
- 1 taken: four 1 coins, then a 5 coin. Total 9.

This overpay is part of the rule. Each coin is one world item as in Drop and money flow.

### Placement with a positive key (direct item)

A kind-8 placement with key 0 or more names the item id directly. No money roll and no item roll happen. The item is spawned as that id (on disc, keys range from 2 to 178).

### Stat-up Negg and shield placements (direct keys)

Confidence: Medium. The lists are archive resources whose type-36 region children hold a direct-key placement (key 0 or more) for the item id. Except where marked, they are world packs (SPEC-0006). Only the presence of the placements is listed here. The flag-1 placements among them are shop stock priced at their fixed count (SPEC-0050 sections 8 and 10): Golden Negg in packs 984 and 1423 (2000 NP) and 1429, 1430, 1431 and 1433 (5000 NP); Starry Negg in packs 857 and 1251 (800 NP) and 1011 (1000 NP); Red Negg in packs 749 (1000 NP) and 1011 (1250 NP). Pack 749 is reached under the 174-pack reach of SPEC-0006 (Medium). Packs 1429, 1430 and 1431 are copies of 1433 that the hero cannot stand in (SPEC-0006), so 1433 is the only one of the four that sells in play. The other placements of these ids have flags 0 or 2 to 4.

- Golden Negg (151): packs 796, 984, 1423, 1429, 1430, 1431 and 1433.
- Starry Negg (154): packs 857, 1011, 1230, 1250, 1251, 1253 and 1358. Resource 733 is also a source, but it is a character region, not a world pack: a top-level type-36 region whose only kind-8 record has key 154, flag 3 and count 0. It is the only region resource with a Negg or shield key. A Giant Slorg in region 733 drops it on death: its spawn record names that region's only placement (drop placement ordinal 0), and the key 154 spawns directly (Medium-High; it depends on region 733 being loaded for the fight).
- Red Negg (174): packs 749, 790, 793, 934, 1011, 1074, 1230, 1249, 1250 and 1369.
- Shield id 176: pack 1327. Shield id 177: pack 749. Shield id 178: pack 816.
- Ids 179 and 180 have no placement in any pack, no script give and no price on disc.

No script gives ids 151, 154 or 174 to the hero. The new game's Golden Negg is never held: it is consumed by the add, and its effects are undone by the reset steps that follow (Sources, New game).

### World items

Confidence: High unless marked. Each tick is dt = 1/60 s (SPEC-0007). "U[a, b)" is a uniform float draw with a included and b excluded (SPEC-0008).

#### Pool and record (runtime)

- Each loaded world has one world-item pool of 50 slots of 0x170 bytes. The world allocates the pool (a 64-byte header and the 50 slots) from its own world memory at every load, so each world buffer has its own pool (SPEC-0026, World memory budget and level setup). Every route takes its slot from this pool: region-load placements, drop actions, coin splits, script spawns and clover finds. There is no per-kind cap. A slot is free when its state is 0.
- A spawn fails only when the id is outside 0 to 322, when the item's carry limit is reached, or when all 50 slots are taken. When no slot is free nothing spawns; the region-load route logs the failure and goes on with the next placement.
- Lifetime of the pool (High): a world's items last as long as the world stays in its buffer. Items lying in the exterior survive an interior visit and swaps between the exterior and the transition. When the buffer is released (any load into it, or the interior jobs releasing the transition and interior buffers), every slot is reset, so an important dropped item is given to the party (Important-item give), and the pool is discarded with the rest of the world's memory. On the next load the placed items respawn from the disc unless vetoed (carry limit, once-per-area set, held map, equipment ranking); dropped and scripted items do not come back.

Slot fields used below (byte offsets in the 0x170-byte slot; runtime memory only):

| Offset | Size | Meaning |
| --- | --- | --- |
| +0x30 | vec4 | position |
| +0x40 | u32 | state (below) |
| +0x44 | f32 | spin rate, radians per second |
| +0x48 | f32 | vertical speed, units per second |
| +0x4C | f32 | horizontal speed (coins), units per second |
| +0x50 | f32 | life: the draw alpha, 1.0 at spawn |
| +0x54 | pointer | the item's config record (Item config table) |
| +0x60 to +0x8F | 48 bytes | copy of the placement's data (Shop purchase rule); flag byte at +0x80 |
| +0x94 | f32 | lifetime L, seconds |
| +0x98 | f32 | age, seconds |
| +0x9C | f32 | ground reference height |
| +0xA0 | f32 | spin angle, radians |
| +0xA8 | u32 | highlight flag (buyable this tick) |
| +0xAC | u32 | no-pickup flag |
| +0xB0 | u32 | serial token: the pool's spawn counter at spawn (the counter starts at 1) |
| +0x150 | vec4 | horizontal direction (coins) |
| +0x160 | f32 | current scale |
| +0x164 | f32 | target scale S1 |
| +0x168 | f32 | start scale S0 |
| +0x16C | f32 | delay, seconds |

States: 0 free; 1 placed (static); 2 settling (non-coin) or rising (coin); 3 and 4 moving; 5 a coin waiting for its delay.

#### Spawning

A spawn takes an item id, a point (and for one form a facing), two flags *snap* and *placed*, a start scale S0 and a target scale S1, a delay D in seconds, a lifetime argument Larg in seconds, an optional direction vector and an optional placement record. In order:

1. Refuse when the id is outside 0 to 322. The id is found in one of four ranges: 0 to 97, 98 to 190, 191 to 282 and 283 to 322.
2. Refuse when the item's carry limit Q (text row +0x08) is above 0 and the party already holds at least Q.
3. Take the first free slot. The pool's spawn counter goes up by one and becomes the slot's serial token. With no free slot nothing spawns.
4. If *snap* is set, the spawn point's height becomes the ground height under it (the ground probe of SPEC-0018, cast from 2.0 above the point).
5. Lifetime: L = Larg when Larg is 0 or more. Otherwise L is infinite when *placed* is set, else 15.2 s for a charm (config type 17) and 11.2 s for any other item.
6. Ground reference: the spawn height when *placed* is set; otherwise the ground height under the point (probe cast from 8.0 above) plus 0.2.
7. Motion start. A coin (ids 135 to 144) gets spin 0 and both speeds 0. A charm gets spin rate s x U[0.4 pi, 0.9 pi) (1.2566371 to 2.8274334 rad/s), where the sign s is -1 or +1 with equal chance, and vertical speed U[2.0, 6.0). The 3.5 cap of shop charms (Charms) does not apply to this first draw, because the placement copy (step 10) is written after it. Any other item gets spin 1.2566 rad/s (0.4 pi) and both speeds 0. Both spin bounds are fixed constants of the draw; the word at 0x2DBC50 is one unit lower in the last bit and is not read by the draw.
8. State: 1 when *placed* is set and the item is not a charm. Otherwise 5 for a coin and 2 for any other item, charms included.
9. Direction: a random horizontal unit vector (x and z each U[-1, 1), then normalised). When the caller gives a direction vector, that vector is used as given, without normalising.
10. Life 1.0, age 0, spin angle 0, current scale S0, delay D. Then the placement's 48 data bytes, if a placement is given, are copied into +0x60 to +0x8F.

#### Spawn routes

| Route | Point | snap | placed | S0 to S1 | Delay | Larg | Direction |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Region load (flag 0 or 1 placements) | the placement point; when placement byte +0x21 is non-zero (3,029 of 3,421 on disc) its height is replaced by the ground height under it plus 0.5, the probe cast from 4.0 above the stored height (0.5 when no ground is found; High). The scripted add by placement number applies the same rule to the placement's point, unless the caller supplies its own point or facing, which then replace the placement's | no | yes | 1 to 1 | 0 | -1 (infinite) | the placement's facing |
| Drop action, one item (breakable, enemy placement with key -1, interactables) | the object's or enemy's position | no | no | 0.25 to 1 | 0 | -1 (11.2 s or 15.2 s) | the caller's, else random |
| Drop action, coins (coin split) | the same point for every coin | no | no | 0.25 to 1 | 0, then 0.4 s more per coin when a direction is given | -1 | the caller's, else random |
| Enemy carried Neopoints | the enemy's position | no | no | 0.25 to 1 | 0 for every coin | -1 | random |
| Enemy direct item (the enemy's direct-item word, item 172) | the enemy's position | no | yes | 1 to 1 | 0 | -1 (infinite) | none |
| Clover patch find (Clover patches) | the hero's position | no | no | 1 to 1 | 0 | -1 (11.2 s) | random |
| Script AddItemToWorld (locator, id, snap, placed, lifetime) | the locator's point, with the locator's vector as the facing | argument | argument | 1 to 1 | 0 | argument | none |

A region-load placement with a negative key (flag 0; 60 on disc) passes the key itself as the item id. Spawning step 1 refuses it, so nothing spawns, no slot is taken and the spawn counter does not change; the start-up logs an error and goes on to the next placement (High). The scripted add instead replaces a negative key with the group item roll.

The region-load route also runs for a character region loaded into a script slot: its item placements spawn when SlotLoad completes its read with the activate argument set (SPEC-0030 row 205). With activate clear, that region's placements do not spawn.

All 118 script AddItemToWorld calls on the disc pass snap 1, placed 1 and lifetime -1. Their items (for example the coins of level module 1397) sit on the ground, never move and never expire.

#### Per-tick step

Runs once per tick for every live item whose display model is loaded, as part of the item draw pass. It does not run at all while the game-state code of the game globals (SPEC-0026) is 9. Order:

1. **Placed (state 1)**: nothing below runs. The item is static (no spin, no bob), never ages and never expires.
2. **Maturity**: the item is mature when the game-state code is above 0, its state is not 2 or 5, its age is at least 0.8 s and its age is at least L / 2.
3. **Fade**: when the item is not mature and its age is more than L, life falls by 2.12 x dt (0.035333 per tick). When life falls below 0, the state becomes 0 (the slot is freed), life is set to 0 and the important-item give runs (Important-item give). The slot is therefore freed on the 29th fading tick: after 28 fading ticks life is 0.0107 and the item is still drawn.
4. **Clock**: when the delay is above 0, the delay falls by dt and the age does not advance. Otherwise the age grows by dt, except when the item is mature and its scale has reached the target (then the age stays). In normal play the game-state code is 0, so every item that is not placed ages.
5. **Motion** by family (below), then the spin: angle := angle + spin rate x dt. The item's rotation is a turn about the world up axis by that angle.
6. **Scale**: while the current scale differs from the target: if the age is more than 0.5 s, current := S1; else current := S0 + 2 (S1 - S0) x age (a linear grow over 0.5 s). The drawn scale is 0 while the delay is above 0, else the current scale.
7. **Draw**: the colour's alpha is the life value. When the highlight flag is set (the item is the buyable candidate this tick; Shop items), each colour component is multiplied by 2.4 and capped at 0.99, and the flag is cleared.

#### Other items: hover and bob

For an item that is neither a coin nor a charm, let h = y - ground reference.

- State 2 (settling): y := y + (2.08 - h), a snap to 2.08 above the reference. When that step was in the window 0 <= (2.08 - h) < 4.8 dt, or -7.8 dt < (2.08 - h) < 0, the state becomes 3. A fresh drop therefore snaps on its first tick and enters state 3 on its second.
- State 3 (rising): if h > 2.08, the state becomes 4 and the item does not move this tick. Else v = 0.75 x (1.3 - min(1, |h - 1.9| x 5.5556)) and y := y + v dt.
- State 4 (falling): if h < 1.72, the state becomes 3 and the item does not move this tick. Else v = 0.75 x (|h - 1.9| x 5.5556 - 1.3), at least -8.75, and y := y + v dt.
- The reference is set at spawn and is not probed again. The item bobs between 1.72 and 2.08 above the reference with a period of about 90 ticks (1.5 s). There is no ballistic flight for these items.

#### Charms: hop

For a charm (config type 17):

- State 2: as for other items, with a hover height of 1.78 instead of 2.08 (the same 4.8 dt and -7.8 dt window), then state 3.
- Any later state: vertical speed := vertical speed - 12.0 dt. When h < 0 and the new speed is below 0, the charm bounces: a new spin rate s x U[0.4 pi, 0.9 pi) with a new random sign, a new vertical speed U[2.0, 6.0) (capped at 3.5 when the item's flag byte is 1, a shop item), and state 4. Then y := y + speed x dt.
- Charms hop forever, until they are collected or expire. The reference is the spawn reference; 12.0, 1.78, 4.8 and -7.8 are fixed program constants.

#### Coins: ballistic flight

For a coin (ids 135 to 144), each tick, the vertical speed first changes: vy := vy - 40.0 dt. Then g is the ground height under the coin (the probe cast from 2.0 above, every tick) and h = y - g.

- State 5: when the delay is 0 or below, the coin launches: vy := U[14.0, 18.0), horizontal speed hs := U[1.0, 3.0), spin := U[0.3, 0.5) x pi, state 2. Otherwise vy, hs and the spin are set to 0.
- Otherwise, when h <= 0.6 and vy <= 0 (a ground contact): vy := vy x -0.65. If |vy| < 1.0 then vy := 0; else bounce cue 0xA7 plays at the coin. Then spin := spin x 0.98 and hs := hs x 0.98.
- When |vy| < 1.0 and h <= 0.66 (0.6 x 1.1): y := g + 0.6 (the coin rests on the ground).
- When vy < 0 and the state is 0 or 2, the state becomes 3.
- Move: y := y + vy dt; position := position + direction x hs x dt (the caller's direction is not normalised).
- At rest the coin keeps sliding: the contact rule runs every tick, so hs and the spin fall by 2 percent per tick. The rest slide distance is about 0.833 x hs.
- The values 16.0 (launch centre, range plus or minus 2.0), -40.0, -0.65, 1.0, 0.6, 0.98, 0.98 and 2.0 (horizontal centre, range 0.5 to 1.5 times) are the data words at 0x2EB3E8 to 0x2EB404 (Where it lives). The 0.3 pi to 0.5 pi spin range and the 1.1 factor are fixed constants.

### Enemy projectile hits (pointer; High)

An enemy projectile hitting the hero (SPEC-0029, Behavior 9) is not an item rule, and no item is spawned by it. Its parts are:
- the bounce: the hero's shield block (the hero's block query is true and dot(projectile direction, hero forward) < 0);
- the percent roll: the status roll; a status is applied when its id is not -1 and an integer draw from 0 to 100 is at most the chance;
- the skip for id 0: status 0 (stun) is not applied while the hero's stun timer is running.

### Dormant reach routines (High)

Three routines walk the world object pools at world +0x2D40 (0x160-byte records), +0x29F0 (0x160-byte records) and +0x3850 (0x140-byte records). These pools are never filled (SPEC-0028, Built-in object pools that are never filled), and no call, jump or stored pointer in the boot program reaches the three routines. Nothing in them concerns items. A clean build omits them.

For the record:
- Their tests use the horizontal squared distance (the vertical difference is set to 0) against 50 and 500, strictly below.
- The 500 routine pushes the record's position (w = 1) out of a character capsule along the hit normal by the penetration depth. Its second store only rewrites w = 1.0; it does not zero a velocity.

### Ring layout and item draw

Confidence: Medium for the ring layout; High for the draw.

Ring layout (Medium-Low; the owner is not the world-item pool but the twinkling point set that box records of key 0 and 12 build, SPEC-0006 and SPEC-0034):
1. The points of the set are grouped 12 to a ring. With N points there are R = (N div 12) + 1 rings, and ring r holds the points 12r to 12r + 11. Each successive point in a ring sits 30 degrees (one twelfth of a turn, the single-precision value 0.5235988 held in the routine itself) further round, and each ring is turned a further 360 / R degrees from the one before.
2. The angle of point k (0 to 11) in ring r is theta = (T mod 12) x 30 degrees + 30 k degrees + (360 / R) r degrees, where T is the set's running time in seconds (0 at creation, plus 1/60 each tick). The value added to the point is A x sin(theta), with A the set's amplitude word (1.0 at creation). It is added to the first lane of the point's sprite vector, which the per-tick step resets each tick to the set's base word (2.0 at creation), so the effect is a size twinkle with period 12 s, not a change of height.
3. The sine is the folded series of SPEC-0034 section 12. Its fold always lies between -0.25 and 0.25, so it equals the ideal sine to about 5e-8 at every angle, including angles past 360 degrees; the clean side may use an ordinary sine.
Test vector: N = 24 gives R = 3. At T = 0 the point k = 3 of ring 0 has theta = 90 degrees and offset +1.0 (size 3.0); the point k = 0 of ring 1 has theta = 120 degrees and offset 0.866 (size 2.866); the point k = 10 of ring 0 has theta = 300 degrees and offset -0.763 (size 1.237). At T = 3 s every angle gains 90 degrees.

Item draw (High): each tick a world item runs its per-tick step (World items, Per-tick step) and is drawn with alpha equal to its life value. The factor 2.4 is the colour boost of the buyable candidate (each colour component times 2.4, capped at 0.99; Shop items), not a draw scale.

Not world items: a plate-and-sinking-object manager kept at world +0x2D40 (a fall of 0.1 per tick, a reset below -50 and a radius-plus-1.5 trigger) is a world-object manager of SPEC-0048. The facing test (within 5.0 units vertically and 4.0 horizontally, facing cosine above 0.8) is the Buy chooser of SPEC-0050 section 3 (Shop items), not a pickup test.

### One-shot event (Low-Medium)

1. The event plays cue 0xB2 (178).
2. It resets its fields.
3. When its drop record exists, it spawns one world item with id 145 (Bagguss) at the event's object position. The common item spawner treats this argument as an item id and looks it up in the item-id ranges (High).

### Cue bag (pointer; High)

The eight-of-twelve draw belongs to the Faerie Sisters' mote-eater projectile (SPEC-0029, Behavior 9, kinds 14 and 15; SPEC-0036, Timed attack-family groups). Its item-side effect is the Mote drain below.

### Mote drain (mote eater) (High; timing Medium-High)

A mote-eater hit (projectile kind 14) that the hero does not block fills the one global timed group with up to eight Mote ids drawn from 160 to 171 and starts it; the draw rule, the index-12 case (a group of 7 entries) and the 1801-step timing are in SPEC-0029 (Behavior 9, kind 14), and the per-sister step rate is in SPEC-0011. Item side:
- Each group entry, when it spawns, drains 20 charge from its Mote by the use rule of Mote charge: if the party holds the Mote and 20 is below its count, the count falls by 20; otherwise the slot is cleared.
- When the Mote is then not held (or was never held), every equipment word of the active hero that holds it is cleared to -1, with cue 0xA4 when the word held it (Equipment slots).
- Test vectors: a Fire Mote with charge 15 is removed by one drain and unequipped (cue 0xA4 if it was equipped); a charge of 100 becomes 80.

### Pickup dispatch and the luck gate

Kind-8 placements reach the world by three entry points. Which one applies depends on the caller:

- **Region start-up**: for each kind-8 placement, in order: the luck gate (below); then, for keys 0 or more, the equipment veto and the quest-style vetoes; then the flag test. A placement with flag byte (+0x20) of 0 or 1 that is not vetoed is spawned once when the region loads, using its key directly as the item id (negative keys included; see Edge cases). Placements with flag above 1 (flags 2, 3 and 4) are skipped here. They spawn only through a scripted add, or when a breakable break or an enemy's death names them (When a drop roll runs). The luck draw is taken before the flag test, so a placement that the flag test skips still uses one draw.
- **Scripted add** (the script asks for an item to be added at a placement): the luck gate, then for keys 0 or more the equipment veto and the quest-style vetoes. A negative key skips those vetoes and uses the group item roll (item roll only, no money roll). A key of 0 or more spawns that id directly.
- **Drop action** on a placement: the money and item flow described above, run only on the events of When a drop roll runs. The luck gate and the vetoes are not applied on this path.

The same luck gate and vetoes apply on region start-up and on scripted add, and on no other path (Open question 83).

**Luck gate** (region start-up and scripted add; applied only when the placement's gate field at +0x28 is not zero):

- Draw r = a uniform integer in 0..100 (101 values, both ends inclusive).
- Let L = trunc(luck × 4.166667), where luck is the player's luck, an integer stored in the hero record at +0x15EC. Compute the product as a float, then truncate.
- The pickup is vetoed when L ≤ r. The pass chance is L / 101.

**Quest-style vetoes** for ids 19, 20, 126, 151, 154, 174 and 175 (keys 0 or more, on both paths). Ids 19, 20, 151, 154 and 174 are once-per-area items: each keeps a list of world ids, and the pickup is vetoed when the current world id is in its list (Stat-up items). Id 126 is vetoed when it is held. Id 175 is vetoed when its map (placement byte +0x2C plus 1) is already held (Treasure maps).

**Carry-limit veto.** The world-item spawner does not spawn an item whose held count is already at least its carry limit Q (text row +0x08) when Q is above 0 (Inventory).

**Equipment veto** (for equipment types 1 to 6, keys 0 or more, after the luck gate, on both paths): vetoed when the family slot is occupied and the pickup id is less than or equal to the equipped id. The slot is chosen by the item's config type alone, from the party's category lists (Party inventory lists): types 4, 5 and 6 test entry 0 (Tor's body, shield or weapon) and types 1, 2 and 3 test entry 1 (Roberta's). The active hero plays no part (High). An empty entry (no object, or count 0) never vetoes.

The 2-in-101 per-tick roll that finds item 19 or 20 belongs to the clover patches (Clover patches), not to placements. It is separate from the luck gate.

### Pickup timing gate

High. A world item can be collected (and found by the nearest-item search of Pickup) only when it is ready.

- State 1 (placed) is always ready. States 0 and 2 are never ready.
- Any other state is ready when the item's age is strictly more than 0.8 s. A coin (ids 135 to 144) uses 0.2 s instead.
- The age is the clock of the per-tick step (World items, Per-tick step): it starts at 0 at spawn and grows by dt each tick once the delay has run out, and it is kept in single-precision float, with each add truncated toward zero (SPEC-0008). After 48 ticks it is 0.79999948, not above 0.8, so a dropped non-coin item becomes ready on its 49th tick of age.

### Pickup

High for the rules; High for the character radius (1.175, SPEC-0040).

1. Each tick the hero's motion record (SPEC-0040) stores the nearest *ready* world item: over all live items with a serial token above 0, the one with the smallest 3-D distance d from the record's position to the item's position (ties keep the first found). It stores the item's token and d.
2. Script item hook (High): when the hero is within 8.0 units, the hook searches the world's item ask-about records for that item's id. A found record with its prompt flag set and not yet asked shows the Square caption label 9 and blocks the collect for that tick (SPEC-0030 section 3, Item ask-about). On disc the hook never finds a record: each world keeps 10 ask-about records, the per-tick lookup accepts only item ids 0 to 9 (SPEC-0030 section 3), all 27 registrations on disc name ids 57 to 174, and no module registers an item-proximity callback. So it never shows the caption or blocks a collect. If a record with an id below 10 existed with its prompt flag set, the Square caption label 9 would show, the collect would be blocked that tick, and a Square press would start its callback once.
3. Candidate: the item is live, its flag byte is not 1 (shop stock is never collected this way; Shop items) and d < 25.0.
4. A non-coin item is refused while the hero's action state is 0x19, 0x21, 0x33 or 0x37.
5. Collect when d < 1.5 + r, where 1.5 is the float at +0x08 of the item's config record (the same in all 208 records) and r is the character's own radius in its motion record: 1.175 for both heroes (SPEC-0040, body size), so the normal collect distance is 2.675. An important item (text row +0x1F = 1) is also collected when its age is more than 0.4 x L, at any d below 25.0 (range collect). There is no pull toward the hero: the range collect is an instant collect at a distance. A placed item has an infinite L, so it is never range-collected; for an 11.2 s drop the range collect starts after 4.48 s.
6. At most one item is collected per tick.

### Auto-collect

High for the steps; Medium-Low for the first-find hook. The collect step adds the chosen world item to the party:

1. The item must be ready (Pickup timing gate).
2. The collect is refused when the item's no-pickup flag (+0xAC) is set, when the id is 191 or more and the item is not a charm, or when the item is the one reserved by a petpet: a single reserved token, set by a petpet's fetch, blocks every hero's collect of that item; the petpet takes the item 5 s later (SPEC-0047; Medium for the timer).
3. A non-coin item that is not a Mote (Motes are ids 160 to 173) is refused while the hero is busy (the busy state list of SPEC-0040, states 7 to 0x3C).
4. When a level script has registered a pickup hook (host row 265, SPEC-0030), the pickup is handed to it: the item id is stored, the hook runs on its own script thread (one at a time), and when it returns the pickup is run again in shop mode with the hook skipped, unless the hook marked the request as handled (Medium-High). Only packs 816, 1096 and 1238 register a pickup hook, and none of them holds shop stock. Otherwise the world item is taken (its slot is freed; Important-item give) and the item is added to the inventory: a charm (type 17) with count 100, a coin's value to the wallet with cue 0xA5 (Coin pickups), id 175 with map number (the record's byte +0x8C, the placement's byte +0x2C) plus 1 (Treasure maps), other items with count 1. In shop mode the add either takes the world item (equipment, stat-up items and maps) or copies it (all other items); see Shop purchase rule.
5. Quick-item auto-select: when the added item's stat E (+0x28, the pool A percent; 0 for types 1 to 6 and 17) is 1 or more and the effective quick-item marker is empty (-1), the marker is set to that id and cue 0xA3 plays (Quick-item selection marker).
6. For heroes 0 and 1 the pickup popup shows. The first find of a kind plays voice 0x2A (Tor) or 0x42 (Roberta) and runs the world's item-obtained hook (Medium-Low; SPEC-0023).
7. After the add, when Q is above 0 and the party holds Q or more, every world item of that id is removed (Inventory).

### Important-item give (quirk, reproducible)

High for when it runs and for the double-give mechanism; Medium for its visible result until a black-box check.

The slot reset runs when an item is taken, when a freed slot is reused by a new spawn, and when the world's pool is reset (at the release of the world's buffer; World items, Pool and record). It first checks that the lifetime is finite (a dropped item, not a placed one) and that a config record is still attached. When the id's text row flag +0x1F is 1 (115 rows: quest items and equipment), the party inventory gets the id with count 1 (capped at the carry limit). The expiry (World items, Per-tick step 3) runs the same give but leaves the record attached, so the later reuse of that slot gives the item again.

Effects:
- An important drop is never lost: if it expires, or its world's buffer is released, it is added to the inventory. Leaving the exterior for an interior does not release the exterior, so no give happens then.
- Quirk: for an important id whose carry limit Q is not 1, a dropped copy can be added twice: once by the take and once by the normal add, or once at expiry and once at the slot's reuse. The ids are 8 (Drackonack Fang, Q 0), 70 (Venom Vial, Q 5), 78 (Black Feather, Q 5), 79 (Green Feather, Q 4), 80 (Brown Feather, Q 3) and 82 (Poison Seed, Q 4). For Q = 1 the carry-limit cap hides the second add.
- These six ids have no source except flag-3 enemy drops (Sources, Conditional placements), so every world copy is a dropped item with a finite lifetime. Each pickup runs the take, whose slot reset gives 1, before the normal add of 1, so every pickup adds 2 (capped at Q). An expired copy gives 1 at expiry and 1 more when its slot is reused or the world is torn down.
- Test vectors: Black Feather (78, Q 5): pickups give counts 2, 4, then 5; at 5 every world copy of 78 is removed. Drackonack Fang (8, Q 0, no cap): 2 per pickup. Brown Feather (80, Q 3): 2, then 3.

### Clover patches

Medium-High. A clover patch is a world object that holds entries of 0x70 bytes. Each tick, for each entry:

1. The entry is tested when the hero's stick speed value is above 0 (the stick length after the dead zone, divided by 87.5 and capped at 1.0; SPEC-0033), that is, while the player pushes the stick, or when the hero's action state is 7 to 11 or 0x2B to 0x2E. With the stick released and any other state, no entry is tested and no roll is made (High).
2. The entry must contain the hero (within 5.0 units horizontally, and a 0.7 sphere against the hero's body) and its found flag must be clear.
3. Patch kind 239 (Four-Leaf Clover) rolls only while the current world's bit in the item-19 world set is clear; patch kind 268 (Twelve-Leaf Clover) only while its bit in the item-20 set is clear. These two sets are the once-per-area lists of Stat-up items (193 world ids).
4. Roll r = a uniform integer in 0 to 100. When r < 2 (2 in 101 per tick), item 19 (kind 239) or item 20 (kind 268) spawns at the hero (Spawn routes) and the entry's found flag is set.
5. While the hero is inside an entry, a rustle effect plays every tick.

### Coin pickups

High. Coins are not kept as items. Collecting a coin adds its value to the wallet (Wallet identity).

- Coin class: ids 135 to 144 (the motion and the 0.2 s ready gate apply to all ten).
- Id 135 adds its count (1 per world coin). Id 136 adds 5. Id 137 adds 10. Id 138 adds 25. Id 139 adds 100. The values are fixed in the add routine itself, not read from the denomination words (the words are used only by the coin split).
- Ids 140 to 144 (the Tal rows) add no value. The add then falls through to the ordinary item add, so a Tal id would be stored as a plain item if one were ever granted; nothing in the game spawns or grants them (High; Counts).
- A coin pickup plays cue 0xA5, and the gain goes to price caption line 1 as a running total (Neopoints, Display).
- The coin add is a plain addition with no cap and no floor (Neopoints, Wallet).

These values match the denomination words in the table above (coin ids 139 to 135 with denominations 100 to 1).

### Stat-up items

Confidence: Medium-High for the flag tests and the pool effects; High for the luck rule; Low for the caps (see the add rule below); High for the once-per-area rule. All three stat-up Neggs are sold in shops (Stat-up Negg and shield placements; SPEC-0050 section 10), and Starry Negg also has a death source.

1. The add path (an item put into the party, not a drop) reads three flags from the item. A flag is set when the item's type is 0, or 7 or higher except 17 (so weapons, body armour, shields and charms never qualify), and the item's stat for that flag is at least 1. Flag A reads stat A, flag C reads stat C and flag B reads stat B.
2. Flag A: each hero's maximum health grows by that hero's health step (Tor 25, Roberta 20), and current health is set to the new maximum (a full refill). The start values and the load-time caps are in SPEC-0036.
3. Flag C: each hero's maximum magic grows by that hero's magic step (Tor 15, Roberta 25), and current magic is set to the new maximum.
4. Flag B: luck grows by stat B times the item count, then is clamped to the range 0 to 24 (High: every write of the luck word clamps to 0 to 24 or stores 0).
5. All three effects apply to both heroes. Each flag adds once per add; the count multiplies only the luck add. The add checks no maximum, so pool values can exceed the load-time caps until a save is loaded (Medium).
6. Once per area (High). The stat-up items on disc are 19, 20, 151, 154 and 174. Each keeps a list of world ids. An add is refused when the current world id is already in that item's list; otherwise the world id is added. Region load does not spawn them for a listed world id. A new game empties the five lists. So each is gained at most once per area, whether found or bought.
7. A successful stat-up plays cue 0x9F.

Items on disc (High for the stats; sources in Stat-up Negg and shield placements and SPEC-0050 section 10):

- Golden Negg (151): stat A 25 and stat C 15. Sets flags A and C.
- Red Negg (174): stat A 25. Sets flag A.
- Starry Negg (154): stat C 15. Sets flag C.
- Four-Leaf Clover (19): stat B 1, so luck +1 per item. Twelve-Leaf Clover (20): stat B 3, so luck +3 per item.
- Silver Negg (152) is not a flag item. It fully restores both pools (its heal fields, see Consumable healing) and is given by script modules 1139 and 777.
- Catalogue prices of the Neggs are not shop prices. A shop price is the placement's fixed count (Shop purchase rule); the wallet and the shop lists are in SPEC-0050.

Test vectors (start values from SPEC-0036):

- Tor health 150 of 150 and magic 90 of 90; Roberta health 120 of 120 and magic 150 of 150. Add a Golden Negg: Tor health 175 of 175, Tor magic 105 of 105, Roberta health 140 of 140, Roberta magic 175 of 175.
- Then add a Red Negg: Tor health 200 of 200, Roberta health 160 of 160, magic unchanged.
- Luck 23, add a Twelve-Leaf Clover: luck 24 (clamped). Luck 24, add another: luck 24.
- Luck 0, add a Four-Leaf Clover: luck 1.

### Treasure maps (item 175)

Confidence: High for the grant sites, the map numbers, the world-pickup veto and the world-pickup number rule. What a held map does on the map page and at its dig site, and the reward a dig drops, are in SPEC-0051 section 10.

- Item 175 is "Treasure Map" (text category 0, catalogue price 200; config type 0 with zero stat fields). The catalogue price is not used: a shop map costs its placement's fixed count, and a granting prompt charges its own amount (SPEC-0050).
- A script give of item 175 takes a treasure number n (1 to 46) as its quantity argument. It is not a count. The give marks map n as held. The item count is not changed: item 175 is never kept as an inventory item. The add plays sound cue 159 (0x9F), not a message. It then requests stinger 37 (Tor active) or 61 (Roberta active); the stinger player accepts only indexes 0 to 18 (SPEC-0031), so the request is dropped and no stinger plays (reproducible quirk; High). A stat-up add ends with the same cue and request.
- A give is refused when map n is already held (Medium). A world pickup, a purchase and a script give all reach the same add of item 175, so all three set the held flag of map n in the same way, play the same cue and dropped stinger, and leave the item count unchanged (High). For a world pickup or a purchase, the map number is the placement's byte +0x2C plus 1; that byte is copied into the world-item record at +0x8C at spawn. A map placement is not spawned when its map is already held (High).
- Twenty-one treasure maps are granted by level scripts: maps 1, 3, 6, 8, 10, 11, 13, 14, 16, 17, 18, 21, 24, 27, 28, 30, 35, 36, 38, 40 and 46. There are 26 grant sites in five level modules: 8 in module 880, 6 in module 1230, 6 in module 1023, 3 in module 991 (maps 21, 17 and 35, for 300, 350 and 400 NP; SPEC-0045) and 3 in module 1353 (maps 36, 40 and 38, for 200, 200 and 300 NP; SPEC-0041). The map number of each of the 991 and 1353 grants is a fixed argument of its give call (High). Each site sits in a yes/no prompt that charges Neopoints (SPEC-0050). A site runs only when its map is not held, and in some chains it also needs the earlier maps of the chain to be held (High for the sites; Medium for the chain gates).
- The other 23 real treasures (rows 2, 4, 5, 9, 12, 19, 20, 22, 23, 25, 26, 29, 31 to 34, 37, 39 and 41 to 45) have no grant in any level script. Map placements (item 175) supply all of them: 24 placements carry maps 1, 2, 4, 5, 9, 12, 19, 20, 22, 23, 25, 26, 29, 31 to 34, 37, 39 and 41 to 45. 21 of them are flag-1 shop stock; three are flag-4 placements (maps 20, 23 and 34, a conditional route). So every one of the 44 real treasures has a map source on disc (High).
- Maps 2, 4, 9, 12, 25, 32 and 33 are sold only as shop stock: maps 25 and 33 in pack 749 and maps 2, 4, 9, 12 and 32 in pack 787. Both packs are reached under the 174-pack reach of SPEC-0006, so all 44 real treasures have a source in reached packs (Medium; it depends on the reach classes and on the stock gates of SPEC-0050).
- Some level scripts use the number 175 as a message id (modules 758, 909 and 934) or as a game-state id (module 1056). Those calls are not grants and are not counted above (High).

### Outfit items (body family)

Confidence: High for the id mapping and for the list of grant sites; Medium for their gates; Low for the sources of the unsourced items.

- The outfit shown is read from the hero's body entry (the hero's slot of the category-5 list). Hero 0's outfit k (k from 0 to 7) is item 119 + k. Hero 1's outfit k is item 127 + k. An empty body entry gives outfit 0. Outfit k loads model resource 504 + k for hero 0 and 512 + k for hero 1. Item 126 in the body entry gives outfit 7 for either hero (for hero 1, 126 minus 127 is -1, which maps to outfit 7), so the cloak model is resource 511 for Tor and 519 for Roberta. Item 126 reaches the body entry only through the cloak swap (Tattered Cloak, below; High). Roberta's body entry holds item 127 (Diplomat Dress) from the start of Act 2, given by the act-start kit (Starting equipment), so she shows outfit 0, model resource 512, until a dress with a higher id is placed there. An empty entry would give the same outfit (High for the kit).
- Item 134 never reaches a body entry (High). Its text row has category 0, so an add puts it in the general list, and no swap or equip path moves a general-list item into the category-5 list. Its outfit offset (7 for hero 1; 15 for hero 0, outside outfits 0 to 7) is therefore never used.
- Known sources:
  - Item 120: one script grant, in level module 1148.
  - Item 122: two script grants, in level modules 842 and 855.
  - Item 126: a grant chain in level module 842. Most of its grant sites sit under a debug-mode test. Two sites are outside it: one unconditional site, and one under a callback that checks state values.
  - Item 130: shop stock (a flag-1 direct placement) in pack 859, priced 6000 NP, with a purchase hook in level module 859 that reacts to a purchase of 130 (SPEC-0050 section 4).
  - Items 121 (resource 1132, shop pack 1131), 123 (pack 1005), 125 (resource 1406, shop pack 1405), 131 (pack 1005), 132 (pack 1433) and 133 (resource 1406, shop pack 1405): shop stock (flag-1 direct placements) in reached packs, sold at their placement's fixed count (SPEC-0050 section 10).
  - Item 129: a flag-1 direct placement only in pack 776, which is reached under the 174-pack reach (see Sources).
- Item 119, Farmer's Clothes (text category 5, config type 4, armour percent 0, carry limit 1, price 0, no description text): a new game puts it into Tor's body entry as a real item with a held count of 1 (Sources, New game, Starting equipment). It is the only item Tor wears at the start. It is the plain starting outfit, outfit 0 (model resource 504).
- Item 127 (price 0) comes from the Act 2 start kit (Starting equipment); item 128 (price 0) is given by a dialogue line (Dialogue grants). Item 124 (price 0) has only a conditional source, a flag-4 direct placement in pack 1096. Item 134 is a blank row (no text name, category 0, price 0) and has no source.
- Drop groups never contain items 119 to 134 (High). Equipment outfits do not come from drops.

### Tattered Cloak (item 126)

Confidence: High for the lists, the swap, its callers and the queries; Medium-High for the visible order of the removal sequence.

- Owning. Item 126 has text category 9. An add fills both slots of the category-9 list (each hero's cloak entry) with it (Party inventory lists). Owning the cloak does not change the outfit.
- Wearing. "Hero h wears the cloak" means hero h's body entry (category-5 slot h) holds item 126. The wearing query used by the level scripts (SPEC-0030 row 200) tests this for the active hero. Either hero can wear it.
- Cloak swap for hero h: the contents of hero h's body entry and hero h's cloak entry are exchanged (item, count and stored values). The first swap puts the cloak on, and the outfit it replaces waits in the cloak entry; the next swap takes it off again. After the swap:
  - when hero h now wears the cloak: the party weapon-drawn and shield-raised flags are cleared (the sheathe sound plays when the weapon-drawn flag was set) and, for heroes 0 and 1, both held-model slots are set to none, so no weapon or shield is shown;
  - otherwise: the hero's held shield and weapon models are rebuilt from its equipment.
- Callers of the swap:
  1. Square on the item screen's Items tab (both heroes' pages), when the active hero's cloak entry is not empty: swap for the active hero and play cue 0xA1. With an empty cloak entry Square does nothing.
  2. A category-5 add while the active hero wears the cloak: the swap for the active hero runs first (the cloak comes off), then the normal add (Party inventory lists).
  3. Loading a save that marks the cloak as worn (SPEC-0025, Tattered Cloak).
  4. The removal sequence below.
- Removal sequence (in play). When the active hero wears the cloak and either its state is set to 13 (the attack step, SPEC-0033) or its hit reaction starts (it took damage), a task runs: one effect (id 0xFB8 of effect bank 3) at the hero, cue 0xA1, a fade to white (255, 255, 255) to alpha 128 over 37 ticks, the hero's one-shot cue 0xB1 with state 0x33; it waits while the hero is in state 0x33, then two ticks, then until the fade reaches its target; then the swap for that hero (the cloak comes off); two more ticks; a fade back to clear over 8 ticks; and the hero's state is set to 1 (idle). So an attack or a hit while cloaked takes the cloak off and ends in idle.
- Other readers of the wearing test (High): the shield draw and the L1 guard need a held shield and no cloak; the item draw hides every held item (SPEC-0046); the hero swap's cloak step (SPEC-0053 step 14); an enemy perception routine, in which one test gives no detection and another halves its value while the active hero wears the cloak (Medium for what the values mean); and several hero action checks refuse while it is worn.
- Pickup veto: a placement of item 126 is vetoed while a category-9 slot still holds it (Pickup dispatch and the luck gate).

### Placement flow and capacity

- A region takes at most 120 kind-8 placements. Placements past 120 are logged and skipped.
- Each placement occupies 0x40 bytes in its type-36 region child (its data starts 0x10 bytes in).
- The live world items of all regions of a world share the world's 50-slot pool (World items, Pool and record). A placement that finds no free slot does not spawn.

### Inventory

- Per-id owned counts are one byte each for ids 0 to 190. An id of 191 or more reads a count of 0.
- There is no second ownership record. The hero object holds a 41-byte (323-bit) set, cleared at hero creation, whose setter marks the bit of the id of the hero's current buyable item. That setter sits on an ask-about-item branch of the world prompt that the game never takes (the test that would open it always answers no), so the set stays all zero in play and nothing reads it (High). The Buy prompt does not touch it. Clean code needs no such set.
- Adding a charm (type 17, a Mote) uses a count of 100 through the player's pool. Other items add one. The 100 is the Mote's charge (Mote charge).
- A byte count is at most 255. The add also never raises the held count above the item's carry limit Q (text row +0x08) when Q is above 0. After a successful add, if the party holds at least Q, every world item of that id in the current level is removed (High).
- The item screen shows each kind's owned count. A count of 0 hides the cell, a count of 1 shows the icon without a number, and a count above 1 shows the number (SPEC-0035 section 14.1). The Items tab shows the 20 grid kinds from the boot-data list above; the Charms and Quest items tabs show the owned records of categories 8 and 7.

### Mote charge

Confidence: High for the count, the use rule, the removal and the amounts.

- Every Mote (type 17, ids 160 to 173) in the inventory has a charge count, set to 100 when the Mote is added.
- Using charge n: if n is below the count, the count becomes count minus n. Otherwise the Mote is removed from the inventory (its slot is cleared: item and count 0) and every equipment word that holds its id is cleared to -1 (Equipment slots). Each use also sets the equipment-panel timer of the spent word to 240 (SPEC-0023 section 4).
- The same scheme serves all three equipment words: word 2 (the weapon Mote), word 1 (the shield Mote) and word 0 (the armour Mote). Each use finds the Mote by its id and spends from that Mote's count.
- Amounts (SPEC-0036, Mote charge use by attacks and Enemy attack damage to a hero; Q from the magic table of SPEC-0036; S is the weapon's stat B, 100 on every weapon, 0 with no weapon):
  - Tor, landed swing (word 2): trunc(Q x (S x 0.01)), at least 1 when Q is at least 1: lesser Mote 1, greater Mote 3.
  - Tor, power attack release (word 2): the same rule at level 1: lesser 24, greater 39.
  - Roberta, full release (word 2): Q x (S x 0.01) rounded half away from zero, at least 1 when Q is at least 1: lesser 25, greater 40.
  - Roberta, quick-shot hit (word 2): exactly Q (2 or 4).
  - Shield block (word 1): trunc(1 x (s x 0.01)), at least 1, where s is the shield's stat B (type 5), stat D (type 3), else 0: always 1 on disc.
  - Armour (word 0): trunc(Q0 x (body stat B x 0.01)) with Q0 = 4 (even) or 6 (odd), no minimum, used only while Q0 is at most the remaining count: 3 or 5.
- The products use the single-precision arithmetic of SPEC-0036 (Damage, Platform arithmetic): 100 x 0.01 gives 0.99999994, so 40 gives 39.999996 and 25 gives 24.999998.
- The charge is never a damage value.

### Weapon reach (High for the values and the layout; Medium-High for the meaning)

The per-hero reach table gives the length of the hero's weapon. It is not a damage factor: the melee damage factor A of SPEC-0036 is the weapon's stat A (Weapon damage stat below).

1. If the hero's weapon-family entry (Equipment slots) is missing, or it holds no item (zero count or no item record), the reach W is 1.5.
2. Otherwise, Tor (hero index 0) takes entry (item id minus 98) of table A. Roberta (hero index 1) takes entry (item id minus 111) of table B.
3. Table A has 11 single-precision entries, for ids 98 to 108, at 0x2DD5E0; the same 11 words are stored again at 0x2DD60C and at 0x2DD638. Table B has 8 single-precision entries, for ids 111 to 118, at 0x2DD670. An implementation reads both tables from the disc at these addresses. Their values are not listed here.
4. Two queries use W: the swing length W + 2.0 (word 0x2DD694) of the hero swing, and W x 0.6 (word 0x2DD690) when the query's argument is nonzero. The Mote power-attack objects are placed at the hero's position plus the facing times W (SPEC-0036, Hit detection and Charm attack hits).
5. The lookup itself has no bounds check. In ordinary game paths, inventory type/owner routing keeps weapon entries in range: Tor receives type-6 ids 98 to 105 (table A indexes 0 to 7), and Roberta receives type-3 ids 111 to 118 (table B indexes 0 to 7). A mismatched or malformed weapon entry can still read outside a table; those raw edge reads are covered under Edge cases.

Test vectors (from the stored tables): Tor with item 98 gives 1.7 (swing length 3.7). Tor with item 101 gives 2.5. Tor with item 105 gives 2.87 (length 4.87). Roberta with item 111 gives 1.2 (length 3.2). Roberta with item 114 gives 1.0. Roberta with item 118 gives 1.45 (length 3.45). An empty weapon entry gives 1.5 (length 3.5; the second query gives 0.9).

**Weapon damage stat.** The melee damage factor A (SPEC-0036, Damage) is stat A (+0x20) of the hero's weapon-family entry when its type is 3 or 6, and 0 otherwise, including with no weapon. On disc: swords 98 to 105 have stat A 4, 7, 10, 15, 20, 25, 30 and 40; wands and rods 111 to 118 have 5, 8, 10, 15, 20, 25, 50 and 40. Stat B (100 on every weapon) is the charge stat S (Mote charge).

### Motes and elements

The fourteen charm records (type 17, ids 160 to 173) are called Motes here. Each element has a lesser Mote (even id) and a greater Mote (odd id). Both members of a pair have the same element kind. Kind 0 means no element. High for the pairs and stats; Medium for the element labels.

| Kind | Element label | Lesser Mote (even id) | Greater Mote (odd id) |
| ---: | --- | --- | --- |
| 1 | Fire | 160 Fire Mote | 161 Lava Mote |
| 2 | Leaf or Rock | 164 Leaf Mote | 165 Rock Mote |
| 3 | Light | 170 Light Mote | 171 Sun Mote |
| 4 | Bubble or Water | 162 Bubble Mote | 163 Water Mote |
| 5 | Fog or Wind | 166 Fog Mote | 167 Wind Mote |
| 6 | Smoke or Shadow | 168 Smoke Mote | 169 Shadow Mote |
| 7 | Nova (all) | 172 Nova Mote | 173 Supernova |

Rules:
- The element belongs to the Mote, not to the wand. An item's element is the kind of its own id: ids 160 to 173 give the kind in the table, and every other id gives kind 0. Wands (111 to 118), swords (98 to 105), shields (106 to 110 and 176 to 180), and body and dress items give kind 0.
- The kind pick (above) is a different lookup and is not used here as the element. Resolved: hero combat uses its two outputs for a Mote use. The first scales a Mote-specific power value; the second is the charge cost subtracted from the equipped Mote. It selects combat values, not animation, sound or drop behavior (Open question 65; High).
- Mote stats, read from the item config records (High):
  - Lesser Motes (even ids 160 to 170): stat F 15, stat G 50, stat H 50.
  - Greater Motes (odd ids 161 to 169): stat F 25, stat G 100, stat H 75.
  - Nova Motes: 172 (Nova Mote) has stat F 0, stat G 50, stat H 50; 173 (Supernova) has stat F 0, stat G 100, stat H 75. Stat F is 0 for both, unlike the other pairs.
- Stat F is the charm multiplier of the equipped charm (Charm multiplier below). Stat G is the type bonus of SPEC-0036 (factor C). Stat H is the armour-Mote reduction: a Mote in word 0 scales an enemy hit by (100 - H) percent (Slot 0: armour Mote).

### Charm multiplier

A Mote in equipment word 2 (the weapon Mote) gives the hero's damage a multiplier of 1 + stat F x 0.01 (factor B of SPEC-0036, Damage). Stat G (factor C) and stat H (armour) are covered in SPEC-0036, not here.

### Item bit set (High)

A set of up to 255 list entries is kept in 32 bytes (256 bits). The count is the list's number of entries, capped at 255. For each entry index i below the count, bit i (byte i divided by 8, bit i modulo 8) is set when the entry passes the test and cleared when it does not. Bits for indexes at or above the count keep their previous value. Bit 255 is never written.

Two variants read the count differently (a 16-bit header field, or the list's count accessor). Both cap at 255 and use the same bit rule.

### Party inventory lists (High)

The item manager is the party inventory object, which both heroes reach through their inventory link. A second object of the same class, the stash, has the same lists. Each list is a count and a base of 0x70-byte slots, held as fields of the object. The count is a fixed capacity set once at boot and never changed:

| List | Count field | Base field | Capacity |
| --- | --- | --- | ---: |
| General list (every other category) | +0x04 | +0x08 | 20 |
| Category 7 | +0x2C | +0x30 | 60 |
| Category 8 | +0x34 | +0x38 | 14 |
| Category 1 | +0x3C | +0x40 | 2 |
| Category 5 | +0x44 | +0x48 | 2 |
| Category 9 | +0x4C | +0x50 | 2 |
| Category 2 | +0x54 | +0x58 | 2 |

Slot record (0x70 bytes each, in the list):

| Offset | Size | Field | Notes |
| --- | --- | --- | --- |
| +0x00 | pointer | item object | a zero pointer means the slot is empty |
| +0x60 | u32 | held count of the item (a Mote's charge); the count query returns it, and a script remove (SPEC-0030 row 191) subtracts from it | High |

The item object's item id is the u16 at +0x0C of that object. A slot is empty when its pointer is 0 or its count is 0 or less. A slot matches an id when it is not empty and its item object's id equals the id; empty slots never match.

Queries, for an id (the category is the text row's category, see Item categories):
- The slot query returns the pointer of the first matching slot, or 0. For categories 7 and 8 it scans in index order up to the capacity. For categories 1, 2, 5 and 9 it tests slot 0 and then slot 1, which is the whole list. Any other category scans the general list.
- The count query (the category-list query) returns the matching slot's +0x60, or 0 when there is no match.

Remove n: find the slot with the slot query. If n < count, count := count - n; otherwise the slot is cleared (pointer and count 0). Remove all clears it. Removal never moves slots and never changes a list's count field, so a slot beyond the count cannot occur.

Add, by category, after the carry-limit rule (Inventory):
- Categories 1, 2 and 5. Slot 0 takes items of type 6, 5 or 4 and slot 1 takes the other type of the family, so the slot index is the hero index. An empty slot is filled. A filled slot is replaced only when the new id is greater. For category 5 only: when the active hero wears the Tattered Cloak, the cloak swap for the active hero runs first (the cloak comes off), whichever slot the new item goes to (Tattered Cloak).
- Category 9. Both slots are filled with the item.
- Categories 7 and 8 (one pass). In index order, the first slot that is empty is filled (count 1, then n - 1 more), unless a non-empty slot with the same id comes before it, which gets n added.
- General list (two passes). First a non-empty slot with the same id gets n added. Otherwise the first empty slot is filled. With neither, the add fails.

Duplicate-slot quirk (categories 7 and 8; Medium-High). Because the one-pass scan stops at the first empty slot, re-adding an id that is held in a later slot, after an earlier slot was emptied, creates a second slot with the same id. The queries then see only the earlier slot. Motes have no carry limit (Q 0), so this can happen with Motes, and also with quest ids whose Q is 0 or more than 1. Quest ids with Q = 1 cannot duplicate: their add meets the cap rule first, which sets the found slot to 1.
- Example: a Fire Mote in slot 0 and a Water Mote in slot 1. Fire's charge is spent to 0. A new Water Mote is picked up: slot 0 gets Water with 100, and slot 1 keeps the old Water charge, which stays hidden until slot 0 is emptied.
- **Re-adding a Mote already in an earlier slot adds charge** (High): if a category-8 scan finds the same Mote id in a non-empty slot before it reaches an empty slot, the add amount is added to that record's count. Since each added Mote contributes 100 and Motes have no carry limit, an existing Mote at 100 charge becomes 200 after a second copy is added. This differs from the later-slot duplicate case above, where an earlier empty slot hides the older charge.
- Numeric vector: category-8 slot 0 contains Mote 161 at count 100; the next slot is empty. Adding Mote 161 adds 100 to the first record, so its charge becomes 200 and the count query returns 200. Two Tor power-attack releases with this greater Mote spend 39 each, leaving 122 charge; a third leaves 83.

These slots are not the hero's three equipment slots (see Equipment slots below). The equipment slots hold an id only. The inventory lists hold pointers to item objects.

### Flag set for ids below 193 (High)

The 193-bit (25-byte) flag set is the storage of the once-per-area lists. There are five of them, one each for items 19, 20, 151, 154 and 174. Bit N stands for world slot N (the world identity number, 0 to 192), not for an item id. The setter sets bit N and the test reads it only for N below 193; any other N is ignored (set) or reads 0 (test).
- Writer: only the inventory add. Adding one of the five items tests the bit of the current world slot; when it is set the add is refused, otherwise the add sets it (Pickup dispatch and the luck gate).
- Readers: the shared placement veto (region start-up and scripted add) and the foliage build, which does not build that item's clover or Negg patch in an area whose bit is set.
- The five sets are cleared on a new game and are written to and read from the save (SPEC-0025).

### World item material list (Medium-High)

Confidence: High for the order of the rules and the id sets; Medium for which boot resource each of the two boot lists comes from.

When a world item is set up (World items, Spawning), its material list is chosen from its item id d and the area number a of the world that spawns it (the world's area word, SPEC-0028). The ids are item ids. Rules, first match wins:
1. The item model registry's per-id material list for d, when it has one (Equipment display indexes and item display models).
2. d is 36, or d is the id held by the world-item display slot, or d is 181: the item-model material list (the list loaded with the boot item models).
3. d is in 98 to 190, unless d is an equipment-model id (98 to 134, 151 to 155 or 174) and a is the area of one of the two loaded worlds: the item-model material list.
4. d is in 283 to 322: the second boot material list (the one that serves the 283 to 322 range).
5. Otherwise the material list of the loaded world whose area is a (the first loaded world when it matches, else the second); none when neither loaded world has area a.

So equipment dropped in a loaded world is drawn with that world's materials, and other ids 98 to 190 with the item-model materials.

Test vectors (synthetic, no registry entries): d = 145 with a the current world's area gives the item-model list (rule 3, not equipment). d = 100 with a the current world's area gives the current world's list (rule 5). d = 100 with an area that no loaded world has gives the item-model list (rule 3). d = 181 always gives the item-model list. d = 300 gives the second boot list. d = 20 with an unloaded area gives none.

### Equipment display indexes and item display models

Confidence: High for the index rules, the resource numbers, the slots and the registry; Medium-High for the reach (the refresh runs in the hero's per-tick update), for the item names and for the index-in-range quirk.

The game keeps four item display slots. Each one holds at most one item model, a standalone mesh container from resources 544 to 575 drawn with the material list of resource 7 (SPEC-0002, SPEC-0004). Each slot records a display id and a state (loading or loaded). A model load does not start while a sound stream is loading; it waits for the stream.

**Item model registry (High).** Item models are found by item id through a registry with four id ranges: 0 to 97, 98 to 190, 191 to 282 and 283 to 322. A lookup finds the range and the index (id minus the range start) and gives a model or none; a second per-id table (323 entries) gives each id its material list. Registering stores both; unregistering clears both, but only when the stored material list matches.
- When a slot finishes loading, the model is registered under id = the model chunk's header id + the slot's base. The bases are 98 for the weapon and shield slots and 0 for the fixed and world-item slots. The header id field and its value for each resource are in SPEC-0004. On disc every resource from 544 to 575 registers under its own item id (98 to 105, 111 to 118, 106 to 110, 50 to 59 and 36).
- At boot, resource 7 registers its own models in the 98 to 190 range by the same header rule (SPEC-0004 gives the ids; they include every Mote and the treasure map).
- **Resource 9 registration (R-0086 clause 1; High).** Its type-8 container registers children in the 283 to 322 range, using base 283 plus each child's role id. It does not fill 191 to 282; world-pack role registrations supply models to that separate range. Resource 9 has 38 children with role ids 0 to 37: roles 0 to 15 map to ids 283 to 298, roles 16 to 28 map to ids 299 to 311, and roles 29 to 37 map to ids 312 to 320; ids 321 and 322 have no child in this resource. The 13 children in roles 16 to 28 are type 12; both type 12 and type 13 children register by this rule.
- A slot load first unregisters the slot's previous display id, then reads the resource. On completion it checks the chunk type (12 or 13; any other leaves the slot empty), fixes the model up against resource 7's material list, registers it and marks the slot loaded. While loading, the slot reports no model.
- The four slots and their buffer sizes: weapon (51,200 bytes, base 98), shield (47,104 bytes, base 98), fixed (20,480 bytes, base 0) and world item (61,440 bytes, base 0). There is only one world-item slot.
- When an item is added to a category list, its entry takes its model and material list from the registry. Each frame the hero draw (SPEC-0046) overwrites the weapon and shield entries' model with the weapon-slot and shield-slot models.

- **Weapon slot.** Each hero update reads the item id of the hero's weapon-family entry (the category-1 list entry; Equipment slots; Medium-High for this reading). Weapon index w = id − 98 for ids up to 110, and id − 103 for ids 111 and above. No item gives −1. When w changes and 0 ≤ w ≤ 15, the slot loads resource 544 + w. Its display id is the item id itself: 98 + w for w = 0 to 7 (the swords 98 to 105, resources 544 to 551), and 103 + w for w = 8 to 15 (the wands and rods 111 to 118, resources 552 to 559). Any other w loads nothing.
- **Shield slot.** In the same update, the item id of the hero's shield-family entry (the category-2 list entry; Medium-High) gives shield index s = id − 106. Only s = 0 to 4 counts (ids 106 to 110, the five shields by name); any other id gives −1 and loads nothing. When s changes, the slot loads resource 560 + s with display id 106 + s. This reading resolves Open question 64.
- **Load keys (values High; meaning Medium-Low on the loader alone).** The loader passes the key 0x220 + w for the weapon slot, 0x230 + s for the shield slot and the fixed key 0x23F with display id 0x24 for the fixed slot. These are 544 + w, 560 + s, 575 and 36 in decimal, the resource numbers and display id given here. That the keys are archive resource numbers rather than separate cache keys follows from the registrations (each of resources 544 to 575 registers under the item id that this index rule gives); the key values alone would not settle it.
- **Fixed slot (item 36).** On the first hero update the slot loads resource 575 with display id 36. It loads only while the slot is neither loading nor loaded, so it loads once per session, whatever the inventory holds. Item 36 is named "Rope Gun" in the item text table (Medium for the name). It is loaded at once because hero state 27 (the staged special of SPEC-0036) sets item 36 as the hero's held entry, and that state can run at any time. The held entry is drawn with the fixed-slot model (resource 575) at the hand while the weapon is sheathed (SPEC-0046) (High for the state-27 write; Medium for its meaning as the Rope Gun use).
- **World-item slot.** When an item is spawned as a world item, the spawn finds the item-id range that holds the id: 0 to 97, 98 to 190, 191 to 282 or 283 to 322. Let n = id − range start. When a world is loaded and 50 ≤ n ≤ 59, the slot loads resource 515 + n with display id n. For the ten potions (ids 50 to 59), n = id, so they load resources 565 to 574. The load is skipped when id n is already registered. Because there is only one world-item slot, only one potion model is resident: spawning a different potion kind unregisters the first, so world potions of the first kind have no model until it is loaded again (High).

Item names for the model resources (item text table 0; Medium-High):

| Resources | Items |
| --- | --- |
| 544 to 551 | 98 Wooden Sword, 99 Squire's Sword, 100 Knight's Sword, 101 Kreludan Blade, 102 Battle Blade, 103 Sword of the Ancients, 104 Razor Sword, 105 Altadorian Sword |
| 552 to 559 | 111 Magic Twig, 112 Apprentice Wand, 113 Enchanter's Wand, 114 Witch's Wand, 115 Magician's Wand, 116 Sorcerer's Wand, 117 Fyora's Rod, 118 Wand of the Ancients |
| 560 to 564 | 106 Wooden Shield, 107 Iron Shield, 108 Knight's Shield, 109 Golden Shield, 110 Legendary Shield |
| 565 to 574 | 50 to 59, the ten potions |
| 575 | 36 Rope Gun |

Three of these models belong to items whose only sources are in packs that are not reached (Counts): 547 (item 101), 551 (105) and 552 (111), all in pack 1259. Items 108 (model 562) and 112 (model 553) have placements in reached packs: 108 is shop stock in pack 776, and 112 has only a conditional flag-4 placement in pack 793. All ten potions have reached sources.

These indexes are display positions for models. They do not replace the slot rules below (Open questions).

### Item showcase

Confidence: High for the numbers and the poses; Medium for the corner placement (modes 1 to 3). The showcase is drawn as the last part of draw step 4 (SPEC-0046), which points here.

The showcase shows a newly collected item. Its object has a mode (0 off, 1 grow, 2 hold, 3 shrink, 4 full), a timer, an angle, a position (x, y, z), a scale s, the item id and an effect.

**Start (item id, quiet flag).** Nothing happens while the mode is 4. Otherwise:
- When the id has a registry model, the timer and the angle are reset.
- When the item has showcase text, the quiet flag is 0 and the hero is not busy (the busy state list of SPEC-0040), the mode becomes 4 with s = 1.0. The world script's item task starts, and stinger 42 (Tor) or 66 (Roberta) is requested. That request goes to the same stinger player as the map add, which accepts only indexes 0 to 18, so it is dropped and no stinger plays (reproducible quirk; High). The pose stingers 6, 7 and 8 of the full showcase task are in range and play.
- Otherwise the mode becomes 1, at (0.5, 0.6, 5.0) with s = 0.
- The item's showcase effect starts at the same time in every mode, whenever the shared effect manager has its group loaded. The effect id comes from the item-effect mapping of SPEC-0034, 3.15 (the same mapping that world items use): 16 item ids map to 759 to 786, the Motes 160 to 173 map to 951 to 964, item 139 maps to none, and every other id, in range or not, gets the default effect 792 (a slow grey puff). So only item 139 and a missing effect group give no showcase effect; an id without its own entry does not get none. The mapping is held in the boot program as a branch table of 161 four-byte entries starting at 0x314B30, one for each item id from 13 to 173 (entry = id - 13); each entry is the address of the code that picks the effect id, not an effect id, and ids below 13 or above 173 skip the table and take the default. The table is not a data table of ids, so a clean build uses the mapping of SPEC-0034, 3.15 (High). Test vectors: item 135 starts effect 761; item 50 starts 792; item 139 starts none; item 163 starts 954.
- The world-item pickup starts it with quiet 0; two give paths always pass quiet 1. The dialogue give (kind 0x13) is a quiet-popup caller, notice mode 1 (SPEC-0022 table row 0x13).

**Each frame (dt in seconds).** The angle advances by 1.2566372 dt (2π/5 rad/s) and wraps by ±2π.
- Mode 1 (grow): s rises by 1.5 dt. For ids 135 to 144 (coins and Tal), once s > 0.4, s is set to 0.4 and the mode becomes 3. For other ids, x rises by 0.5 dt and y falls by 0.5 dt, and once s > 0.8, s is set to 0.8 and the mode becomes 2.
- Mode 2 (hold): after 1.3 s the mode becomes 3.
- Mode 3 (shrink): s falls by 2.5 dt and x rises by 0.5 dt. When s < 0 the showcase turns off.
- Mode 4 (full): only the timer and the angle change.

**Draw.** Nothing is drawn when the id has no registry model.
- Mode 4: the item model is drawn at the hero position + 1.1 × the hero's forward axis + 2.8 × (0, 1, 0), that is 2.8 units up and 1.1 units ahead, turned about the vertical axis by the angle, with the effect at the item.
- Modes 1 to 3 (Medium): the model is anchored to the screen. With the camera's view-to-world rows R (right), U (up), F (forward) and E (position), S the render camera's focal length (512 in normal play), and (x, y, z, s) the showcase position and scale:
  - the model's origin is E + z·F − 640·(x − 0.5)·(z / S)·R − 480·(y − 0.5)·(z / S)·U, so (x, y) are fractions of a 640 × 480 screen, (0.5, 0.5) is the centre, and z = 5.0 units in front of the camera;
  - the model is first turned about its own vertical axis by the showcase angle, then given the camera's orientation, and every axis is scaled by s.
  - In practice the item spins facing the viewer as it slides from (0.5, 0.6) toward the lower right. The sign of each screen axis follows the camera rows of SPEC-0027.
  - Quirk: the factors 640/W and 480/H (W, H the camera record's screen size words, 640 and 480) are computed on the first anchored draw and then cached.
  - The showcase effect is drawn at scale 1.5.
  - Test vector: start (0.5, 0.6, 5.0) with S = 512 gives a horizontal offset of 0 and a vertical offset of 0.46875 units along −U.

**Full-showcase task** (run when mode 4 starts):
1. Scene camera on the hero: the same routine as the script call SetHeroSceneCamera(yaw offset 0.9425, radius 10.0). It starts the blended-transition camera mode of SPEC-0027 (Blended transition, mode 8) on the selected hero with a yaw offset of 0.9425 radians (54 degrees) from the hero's facing, a horizontal reach of 10.0, a height of 1.5 (distance 10.112, elevation 0.1506 radians), all three blend fractions 0.01, no pivot step and no instant placement. The camera therefore eases from where it was toward an eye 10.112 units from its pivot whose yaw is the yaw of the hero's forward row plus 0.9425 (SPEC-0027, Polar convention), closing 1 percent of the remaining distance, yaw and elevation each tick (about 70 percent after 2 s). Test vector: for a hero whose forward row is (1, 0, 0) the target eye is at the pivot plus (5.877, 1.517, 8.088). Step 6 undoes it with the return-to-normal routine of SPEC-0027 (Return to normal; the same effect as ReturnToNormalCamera(-1, true)): mode 0 and a camera reset from the pivot, then the hero is released (High).
2. Clear the weapon-drawn and shield-raised flags (SPEC-0040).
3. Pick the pose and stinger:
   - Sword 98 to 105 with Tor, or wand 111 to 118 with Roberta: set both flags, pose 0xA4, hide the floating item, stinger 8.
   - Shield 106 to 110 with Tor: set both flags, pose 0xA6, hide the floating item, stinger 6. The shield branch shares its tail with the armour branch, so stinger 6 is requested here exactly as for armour; it is in range (indexes 0 to 18) and plays (High). A build that plays no stinger for a shield is wrong.
   - Armour 119 to 125 with Tor, or dress 127 to 134 with Roberta: pose 0xA5, hide the floating item, stinger 6.
   - Anything else: pose 0xA3, and the item stays floating. With text: stinger 7.
4. With text, show it with the speaker name "Tor" or "Roberta".
5. Wait at least 2.0 s and until the text closes, requesting the pose again while waiting.
6. Restore the camera, turn the showcase off and release control.

The flags are not cleared at the end, so a new sword, wand or shield stays drawn in hand.

Test vectors:
- Tor collects 102 Battle Blade, which has text. Mode 4 starts, pose 0xA4 plays, the floating model is hidden and the sword is drawn in Tor's hand (resource 548). After at least 2 s the showcase ends with the weapon-drawn flag still set.
- A quiet start of item 146 (mode 1): s = 1.5 × 0.5 = 0.75 after 0.5 s, then s passes 0.8 during the next frames, and the mode becomes 2. It holds for 1.3 s, then shrinks at 2.5 per second, about 0.32 s, and turns off.
- A quiet start of coin 139: s passes 0.4 after about 0.27 s, then shrinks (mode 3) with no hold.
- Angle after 1.0 s: 2π/5 = 1.2566 rad.

### Equipment slots (per hero)

Each hero (SPEC-0036) has three equipment slots. Each holds an item id, or -1 when empty. These are not the item manager's category slots (Party inventory lists).

Storage (Medium-High). Each hero's three equipped-id words sit in the hero's record at offsets +0x1544, +0x1548 and +0x154C, for slots 0, 1 and 2 in that order, each holding an id or -1. These are not quick-item slots. The quick-item panel has its own four entries, and its marker is at +0x155C (Quick-item selection marker). Each hero also has its own entry in each equipment list, and the list index is the hero index (0 for Tor, 1 for Roberta).

- **Slot 2, the weapon Mote (the attack)**: holds a Mote, ids 160 to 173 (High). Its id is the attack id of SPEC-0036; its stat F gives factor B, its stat G factor C, and its element kind is the attack element. A landed attack spends its charge (Mote charge).
- **Slot 1, the shield Mote**: holds a Mote that strengthens the shield block of SPEC-0036 (Enemy attack damage to a hero): with an item in word 1 the block is 1.25 times the shield's stat A, and a blocked hit whose element passes spends 1 charge and 2 or 1 MP (High).
- **Slot 0, the armour Mote**: holds a Mote that reduces enemy hits by (100 - H) percent while its cost and charge allow (Slot 0: armour Mote below; High).
- **The weapon, shield and body items** are not in these words. They are the hero's entries in the item manager's category lists (Party inventory lists): category 1 (weapon family, types 3 and 6), category 2 (shield family, types 2 and 5) and category 5 (body family, types 1 and 4), with the list index equal to the hero index (High: the add puts types 6, 5 and 4 in slot 0 and the other type of each family in slot 1). The weapon entry supplies factor A (stat A), the reach (Weapon reach) and the charge stat S (stat B); the shield entry the block amount (stat A); the body entry the armour percent (stat A) and the armour charge stat (stat B).
- The second quick panel's equip tests match this pairing: Square fills word 1 only when the hero's category-2 slot (a shield) is not empty and Circle fills word 2 only when the hero's category-1 slot (a weapon) is not empty; Cross fills word 0 with no test (High for the test; Medium-High for the pairing; Open question 95).

Equip and unequip rules (High):

- The item screen writes the equipment words only from its Charms tab, which runs the Mote picker of the second quick panel (Second quick panel and Mote equipping): Cross into word 0, Square into word 1, Circle into word 2. Circle on the Items tab sets or clears the active hero's quick-item selection marker (+0x155C) and writes no equipment word (Quick-item selection marker, rule 4). High.
- Setting a slot to the id it already holds does nothing and reports no change.
- Setting word k to a different id m, in order: (1) store m; (2) re-arm word k's equipment-panel timer to 240 (word 0: G3, word 1: G4, word 2: G2; SPEC-0023 section 4); (3) for hero 0 (Tor) only, start the shield Mote glow (word 1) or the weapon Mote glow (word 2) (SPEC-0046); (4) when the caller's show flag is 1, play cue 0xA3. The Mote panel always passes 1, so every panel equip plays cue 0xA3; no slot has its own cue.
- Clearing a slot sets it to -1. For hero 0, a clear of word 1 or 2 stops the matching glow. Cue 0xA4 plays only when the caller asks for it and the word held an id.
- Owned records: charms and quest items are the owned records of categories 8 and 7, taken in list order. Where the numbering overlaps, see Open question 59.
- The family-ranking veto (Equipment ranking above) applies to pickups. It is not part of the equip or unequip calls.

Using an equipped item (Medium-High): the quick panel and the item screen both send a use through one shared item-use dispatch, with the arguments (id, force 1, notify, gate 1); notify is 1 from the panel and 0 from the item screen. The dispatch first applies the use-request gate (Use-request gate and charm-only flag). Then:
- A charm id (160 to 173) is never consumed and the dispatch reports failure. For each of the hero's three Mote words that holds that id, the matching equipment panel's visibility timer is set to 240 ticks (word 0: G3, word 1: G4, word 2: G2; SPEC-0023 section 4), so that panel shows again. Nothing else changes. On the item screen the failure plays cue 0xA2.
- Any other id goes to the consumable use (Consumable healing; Consumable effects by item id). A success removes one item.
- No item class is exempt on either path. The dispatch and the consumable use test the charm range and the charm-only gate, not the text-row class.

Item screen use (resolves the use trigger for owned items): on the item screen, Cross on a cell runs, in order: (a) the cell must be occupied (count above 0 and a non-empty item record), else cue 0xA2 and nothing else; (b) a party-side check for the id must pass, else cue 0xA2; (c) the item manager's use call runs for that id. Square on the Items tab puts the Tattered Cloak on or takes it off: it runs the cloak swap for the active hero when that hero's cloak entry is not empty (Tattered Cloak, below; SPEC-0035 section 14.2). Medium-High for Cross, High for Square. Step (c) is the shared item-use dispatch of Using an equipped item, with notify 0; no item class is exempt.

### Slot 0: armour Mote

High. The Mote in equipment word 0 is the armour Mote. It is not a use action: it acts when an enemy hit lands on the hero (SPEC-0036, Enemy attack damage to a hero).

1. It acts only when the hero has a body-family item, the hero's pool B (magic) current value is at least its cost, its power Q0 is at most its remaining charge (Mote charge), and the attack's element is 0, equals the Mote's element kind, or is 7.
2. The hit is scaled by (100 - H) percent, where H is the Mote's stat H (75 for greater Motes, 50 for lesser ones).
3. Then word-0 charge is used (trunc(Q0 x (body stat B x 0.01)), no minimum: 5 or 3 on disc) and the cost is taken from pool B.
4. In every case, the body item's stat A then reduces the hit by that percent (d x (1 - stat A x 0.01)).

Cost and power of the armour Mote, by its family (High for the values; the same for Tor and Roberta):

| Family of the Mote in word 0 | Pool B cost | Power Q0 |
| --- | ---: | ---: |
| Odd (greater Motes 161 to 173) | 4 | 6 |
| Even (lesser Motes 160 to 172) | 2 | 4 |
| None (word 0 empty or not a Mote) | 0 | 0 |

The power is not a damage value; it gates and sizes the charge use.

### Shop purchase rule

The buy action runs against the player's current buyable object, which carries a price as a signed 16-bit value at +0x8A. The money rules for this section (the wallet, the buy decision order, the refusal messages, the buy hooks, the shop lists) are in SPEC-0050. This section keeps the buyable object's layout and the item-side checks.

Shop stock and price (High):
- When a region spawns a kind-8 placement (at region load or by a scripted add), the placement's 48 data bytes are copied unchanged into the world-item record at +0x60 to +0x8F. So record +0x80 is the flag byte (placement +0x20), +0x84 the item key (placement +0x24), +0x8A the fixed count (placement +0x2A) and +0x8C the placement byte +0x2C.
- A world item is a shop item exactly when its flag byte is 1. Only a flag-1 item can become the buyable object, and only flag-1 items are scaled by the price multiplier.
- The buy price is the record's +0x8A, which is the placement's fixed count. On disc every flag-1 placement has a price from 20 to 10,000 NP and a key of 0 or more.
- Take or copy: a purchase takes the world item (removes it from the level) when the item is equipment (types 1 to 6), a stat-up item (Stat-up items) or a treasure map (175). Otherwise it adds a copy and the world item stays, so it can be bought again.

Shop items in the world (Medium-High for the highlight and the press; High for the rest):
- Flag-1 placements spawn as placed items (World items): static (state 1), always ready, never expiring and at full scale at once. A flag-1 charm hops like any charm, with its bounce speed capped at 3.5.
- They are never auto-collected (Pickup, step 3). Each tick the Buy chooser (SPEC-0050 section 3: within 5.0 units vertically and 4.0 horizontally, facing cosine above 0.8) picks among the ready items; when its winner has flag 1, the hero's buyable pointer is set to it and its highlight flag is set, so it is drawn with its colour times 2.4, capped at 0.99 (World items, Per-tick step 7). The chooser does not run while a hero test of SPEC-0040 is true or while the hero's action state is 0x19.
- The purchase runs on a Square press edge (the newly-pressed bit 0x0080 of the pad word; SPEC-0033). Without that press the prompt captions are shown.

Price setter (High for the write):
- A world script calls a price setter with the new price. The setter stores the price as 16 bits at +0x8A of the buyable object.
- The setter finds the buyable object through the hero object: the hero object holds a pointer to the buyable object at +0x970 (High for the offset; Medium for the rest of the path). When the object is missing, the setter does nothing; it may print a debug message only.
- A second script call registers a buy hook (host row 260) in the level's script manager, not on the buyable object (High). SPEC-0050 section 3, step 2, gives its effect on a purchase.
- The price is in neopoints (the same wallet the rule below compares it with). The setter overwrites the price copied from the placement; no shop stock depends on it.

Price multiplier (High): a script call scales the buy price of every list record whose flag byte (+0x80) is 1 by 4.0. The result is truncated to a whole number and capped at 32767. The field changed is the buy price at +0x8A, the record's copy of the placement's fixed count; the placement data is not changed. The call is one pass over the list at the time of the call; records added later are not changed by it. It is made in four packs while story flag 298 is clear (SPEC-0050 section 5).

1. If there is no buyable object, the action stops.
2. Shop pre-check (Medium-Low): a separate check runs before the purchase. It is not a plain stop. The check tests whether a request slot is busy. When the slot is free, the check returns 1 and the purchase continues. When the slot is busy, the purchase is queued on a separate thread, this action ends, and no price is taken. The queued purchase runs again later without this check, unless the queued request's own flag is set. No message from the check is known (Open question 12).
3. If the price is 0x7FFF, the message "I could never afford that!" is shown. The purchase does not happen.
4. The price is tested against the balance first. The item is added, in shop mode, only when the price is at most the player's NP, by the same add routine as Auto-collect. NP decreases by the price only after the add succeeds. The purchase message is then not shown. The add routine has its own busy gate. When that gate is busy, the add fails and is queued; the queued add later gives the item with no price check and no charge, so a busy add can give the item without a charge (Medium). A stored price of 0x8000 or more reads as negative. It passes this test, and a successful add raises NP by the size of the price.
5. If the add fails, or the price is more than NP: when NP is less than the price, the message says how much more NP is needed (price − NP). Otherwise the message is "I can't buy that now!".

NP is a signed 32-bit wallet value. No cap exists on the buy path. Table 0's price field is the catalogue price. No shop reads it; it is read only by the enemy item steal (Item text database).

No shop screen exists in the front-end states or the item screen. The shop is the world "Buy" prompt for the nearest ready flag-1 world item (SPEC-0050 section 3). Items are never sold for Neopoints.

### Wallet (shared Neopoints)

- **One wallet.** Neopoints are one signed 32-bit value held in a single wallet object that the party shares. Both heroes read and change the same value; there is no per-hero purse (High). The wallet object is created at start-up and is not stored as a disc file.
- **Query.** The Neopoints query returns the value. When no wallet object exists it returns 0.
- **Remove (amount a).** When a is above 0, the value becomes the value minus a, floored at 0. The wallet does not need to hold the full amount, and the call never makes the value negative. When a is 0 or below, nothing changes (High).
- **Give (amount a).** When a is above 0, the value becomes the value plus a. A sum past the signed 32-bit limit wraps negative and is then clamped to 0. When a is 0 or below, nothing changes (High).
- **No wallet object.** Remove and give test for the wallet object before they change anything: with no wallet, both do nothing and the query returns 0 (High).
- **Shop payments** subtract the price from this same value (Shop purchase rule above).
- The wallet is the source of the "NP" that the shop rule compares.
- **Wallet identity (Medium-High; see Open question 20).** Coin pickups and the shop purchase both read and change the same wallet value. That value is held on the party's inventory object, which the hero reaches through its inventory link. The coin route and the shop route therefore share one value. The script calls described above also act on one wallet; the shared coin and shop routes make that identity Medium-High rather than Medium. The hero argument of the script calls is covered by Open question 23.

Wallet test vectors and edge cases are listed below. A script that takes more money than the party holds gets 0, not a debt.

## Sources

Confidence: High for the routes, the placement and drop counts, the drop-group rules and the script-grant counts. Medium for the coin spawn route's conditions and for the pack reachability classes (Open question 90). Medium-High for the enemy death trigger; High for breakable and scripted-add triggers (When a drop roll runs).

An item reaches the player by one of four routes. The counts below cover all direct placements, drop placements, script grant calls and world spawn calls on the disc.

### Direct placements

- A kind-8 placement with a key of 0 or more names its item id. The disc has 1,061 such records, with 111 distinct ids from 2 to 178.
- The flag byte decides region load. The counts here cover the 1,061 keyed records in all packs, reached or not (in reached packs: 391, 154, 145, 178 and 65 for flags 0 to 4). Over all 3,421 item points, keyed and drop records together, the flag counts are 512, 201, 1,241, 1,241 and 226 (SPEC-0006, Kind 8); the drop records hold the other 60, 0, 1,096, 1,051 and 153. Flags 0 and 1 spawn when the region loads (452 and 201 keyed records). Flag 1 marks shop stock: the item is bought at its fixed count (Shop purchase rule; SPEC-0050 section 10). Flag 0 items are free pickups. Flags 2, 3 and 4 (145, 190 and 73 records) do not spawn at region load. They spawn only through a scripted add, or when a breakable break, an enemy's death or a chest or dig reward names them (When a drop roll runs; Conditional placements below).
- Placements with a negative key and flag 0 also pass through region start-up, where the spawn refuses the negative id and nothing appears (Edge cases).
- Pack 735 has one item point, item 3 (Black Bogberries), on the pit island at (3780.64, 111.59, -614.61).

### Drop placements

- A kind-8 placement with key -1 is a drop placement. The disc has 2,360 such records. Their group field selects drop group 0 to 6, or is -1 for none. Counts: group -1: 394; group 0: 427; group 1: 79; group 2: 408; group 3: 79; group 4: 74; group 5: 610; group 6: 289. All 394 group -1 placements are referenced. 325 of them lie in world packs, whose placement-id table lists every placement of the pack. 69 lie in character regions, which have no id table. 131 of the 394 are the target of an enemy drop ordinal; these include the 69 character-region placements, which no other route reaches. Of the 131, 67 have a fixed count above 0 and so give coins only (Open question 91).
- The item roll is made only when the placement's fixed count (+0x2A) is 0 and the money roll fails. 1,969 of the 2,360 records have a fixed count of 0. A record with a positive fixed count gives coins only.
- Group 5 gives money only. It has no item entries.
- Sub-table 7 is named by no drop group, so it is never used.
- Group 0's third entry (sub-table 5, weight 10) is never chosen. Every other entry of groups 0 to 4 and 6 can be chosen, and every sub-table entry has a positive weight.
- The 34 ids that appear in a sub-table each also have a direct placement in a reachable named region or a script grant. Drops therefore add no item that lacks another source; their effect is on money and on which of the other items a placement yields.
- The drop action runs only on the events of When a drop roll runs. A drop placement therefore rolls only when a breakable names its placement id, an enemy dies with its drop placement ordinal, or a script adds it with a negative key. Each of the 394 group -1 placements is either the target of an enemy drop ordinal or listed in its world pack's placement-id table (Open question 91). Whether a breakable or a script uses a placement-id table entry is not established.

### Conditional placements (flags 2, 3 and 4) (High; every world pack)

The three conditional flags are a clean partition by trigger:
- Flag 2: breakable drops, named by a breakable's drop id (+0x56).
- Flag 3: enemy drops, named by an enemy spawn record's drop ordinal (+0xB0 of the record data).
- Flag 4: type-40 group rewards: Square-press chests (kind 1), element-sealed chests (kind 2), dig sites (kind 6) and breakable chains (kind 4).

The per-flag counts, the few placements that are named twice, and the table of large Neopoint stashes (key -1, count of 1,000 or more) with the trigger of each are in SPEC-0050 sections 12 and 12.1. Every large flag-4 stash has a named trigger: the 25,000 NP and 6,000 NP stashes of pack 1096 are unlocked kind-1 chests that open on a Square press, the 1,487 NP of pack 1363 is an element-sealed chest, and all the others are dig treasures.

Bush drops (coin bushes and Negg bushes) are specified in SPEC-0060, Foliage patches.

### Script grants

- A script gives an item to the hero by a fixed item id. The disc has 514 such calls. 513 name a valid id; one names -1 and gives nothing. No call computes its item id.
- 57 calls sit under a debug-mode test and never run. No item has only debug-gated grants: item 126 keeps two normal grants (module 842), and item 170 keeps its other sources.
- Item 7 (Bucket of Water) has one source: a grant gated by a button combination in module 909.
- Item 36 (Rope Gun) has one in-play grant: quantity 1 in a staged Farm scene on the Boss Defeated reactivation event for world 68. Pack 1217 registers the Farm-return teleport callback as index 0 during module initialization, without a game-flag gate. When that callback fires, it posts event value 50 to world 68 and replacement-loads that world. A separate pack-1217 progression path sets flags 278 and 6 under flag 83 and two additional predicates that remain unidentified. The Farm consumes event 50 as its Act 3 return variant: when stage 6 is active and item 36 is absent, the script selects locator 28 at (−1822.084, 50, 1190.448), stages that scene, and grants one Rope Gun if game-state flag 6 ("Free Tor's Farm") is set; otherwise it selects locator 4 at (−1790.221, 68.718, 884.497). Locator 28 is a script-selected placement, not a decoded geographic label. Lower-numbered progress flags do not suppress flag 6. The module's additional scene-enable field is initialized true and the level script does not change it, so it adds no dynamic story-state requirement. When item 36 is already held, the Farm selects the alternate return variant and skips the grant scene. The story-path sequence places this grant on the Act 3 return to the freed Farm before the later Glade revival visit; it also has an earlier Act 1 visit to the Glade before this grant. In the later Glade interaction in module 956, possession changes the missing-item dialogue, but all three rope targets are activated either way and this branch sets no completion flag. The Farm-return contact plane is mapped, but it is not the only static re-entry: the overworld cell-neighbor graph has a backtrack chain 1023↔696↔1066↔691↔909, and Farm Road interiors also exit to the Farm. Physical traversal of the full chain remains unconfirmed. These alternate arrivals do not establish another item-36 grant route: only event value 50 is tied to the stage-6 missing-item scene, and among event producers only pack 1217 posts value 50. The meanings of the separate progression predicates remain open. This supports preparation for the later Glade scene but does not establish that rope use is required for story progression. (High for the static event chain, scene predicates, selector and grant; Medium for route interpretation.)
- The corresponding pack-1217 teleport plane is the first plane in the filtered teleport list (source plane record 1). Its reference point is (18.375, −3.416, 46.995), its normal is (−0.279, 0, −0.960), and its half-extents are 26.754 along the in-plane width axis and 14.995 vertically. Contact with this plane transitions to world 68 (Farm pack 909) without a prompt or confirm press. The mapped plane closes the event's physical entry geometry; it does not establish whether other Farm-return routes exist.
- GiveItemToHero (SPEC-0030 row 190) runs the party add with quantity n. For coins, 135 adds n Neopoints, and 136, 137, 138 and 139 add 5, 10, 25 and 100 whatever n is, with cue 0xA5. With its bool set, the plain pickup popup shows first, with no first-find voice and no world item hook (High for the call path; Medium-High for the popup).

### Coin spawn (world spawn call)

- A world spawn call (AddItemToWorld) places an item from a level record table. Its first two arguments are integers, in this order: the first is a locator index (the spawn point, counted in the world's locator list) and the second is the item id. It then takes a snap flag, a placed flag and a lifetime; every call on the disc passes snap 1, placed 1 and lifetime -1, so its items rest on the ground, never move and never expire (World items, Spawn routes). The index must be zero or more and below the length of that list, which is set at run time and is not on the disc; an index outside that range spawns nothing. In level module 1397 there are 33 distinct calls, all passing item 139 (the 100 Neopoint coin), at locator indexes 24 to 71, one coin per call. A button-press trigger that opens a mechanism spawns 21 coins (2,100 NP). A second trigger spawns 12 coins (1,200 NP) and sets flag 349. Each trigger runs once per visit. Total 3,300 NP.
- In level module 1382, one call spawns a Mote at locator indexes 5 to 10. It takes its item id from a runtime record field at +0x50. The six initialized pairs set this field to 168, 160, 166, 170, 162 or 164; the paired odd Mote id is stored separately at +0x54. No other direct store to the item field occurs in this module. This spawn therefore cannot supply any of the 78 no-source ids below (Open question 93).
- Coins 135 to 139 each have a flag-0 or flag-1 direct placement in a reachable named region (135 in pack 757). Items 135 to 139 are also granted by script in module 1023: they are the payout of its wheel game (20, 40, 60, 80 or 160 NP, split greedily into coins; SPEC-0050 section 11). Coin splits from money rolls (drop groups 0 to 4 and 6) are a conditional route.

### Boot grant handler (not a source)

- The boot program has a grant handler that gives items by event code 1 to 12. No direct caller or registered boot table entry reaches it, so its grants are not counted as item sources. It is the only grant of item 180 (Aegis Omnibus); no route to that grant is established. An indirect runtime entry is outside this static result (Open question 93).

### Outfit placements

- Outfits sold as shop stock (flag-1 direct placements) in a reachable pack: 121 (resource 1132, shop pack 1131), 123 (pack 1005), 125 (resource 1406, shop pack 1405), 130 (pack 859), 131 (pack 1005), 132 (pack 1433) and 133 (resource 1406, shop pack 1405).
- Item 129 has a flag-1 placement only in pack 776, which is reached under the 174-pack reach. Item 119 comes from the new game (below). Item 127 comes from the Act 2 start kit (Starting equipment) and item 128 from a dialogue line (Dialogue grants).

### New game

- The game reset clears the party inventory, sets the wallet to 0, and then adds Farmer's Clothes (119) x1, Red Juppie (150) x3, Purple Juppie (149) x3 and Golden Negg (151) x1 through the inventory's normal add, the same add a pickup uses (High for the items and the add).
- The normal add consumes stat-up items (Stat-up items), so the Golden Negg is not stored. Its stat-up is applied to both heroes, and the current world's bit is set in its once-per-area list. If that bit is still set from an earlier game in the session, the add is refused instead. Either way, no Golden Negg is held after a new game (Medium-High).
- Later steps of the same reset undo its effects: the five once-per-area lists are emptied, and each hero's current and maximum pools are set to the start values (SPEC-0036). The new game therefore starts with items 119, 150 and 149 only, and with the start pools (Medium-High).

#### Starting equipment (High)

- After the add of the kit, the party holds exactly: Farmer's Clothes (119) x1, Red Juppie (150) x3 and Purple Juppie (149) x3. Nothing else: no weapon, no shield, no charm, no Tattered Cloak, no quest item, and 0 Neopoints.
- Item 119 is type 4, so the add (Party inventory lists, category 5) puts it in slot 0 of the category-5 list, which is Tor's body entry (hero index 0), with held count 1. It is an ordinary list entry, not a default. Tor's body entry is therefore not empty at the start, and the outfit shown (outfit 0, model resource 504) is read from that entry. The equipped body item is item 119.
- Red Juppie and Purple Juppie have category 6 and carry limit 0, so they go to the general list with counts 3 and 3.
- Everything else starts empty. Tor's weapon entry (category-1 slot 0) and shield entry (category-2 slot 0) are empty, so Tor starts bare-handed with no shield. Roberta's entries (slot 1 of the category-1, 2 and 5 lists) are all empty. Both cloak entries (category 9) are empty. The three Mote words of each hero (equipment slots 0 to 2) hold -1, and the charm list is empty.
- Roberta is not playable in Act 1 (swap mode 0 keeps Tor selected), so the new game gives her nothing. Her entries are empty until the Act 2 start (next bullet).
- Tor's first weapon is not part of the kit. Wooden Sword (98) is a later find: the chest in the first house (pack 951) and the squire gift in pack 1148 (Sources).
- Act 2 start (the act-start setup, stage 2; SPEC-0053 section 3.7; Medium-High for the kit order, High for the ids): the whole party inventory and wallet are copied to the stash, the party inventory is cleared (every list, wallet 0), and a second kit is added through the normal add while Roberta is the selected hero: Red Juppie (150) x3, Purple Juppie (149) x3, Diplomat Dress (127) x1 and the coin item 135 x1150 (the coin rule adds 1150 Neopoints to the wallet). Diplomat Dress is type 1, so it goes to slot 1 of the category-5 list: Roberta's body entry, a real item with held count 1 (text category 5, armour percent 0, price 0, carry limit 1). It is her starting outfit (outfit 0, model resource 512). Nothing else is granted to her: no weapon, shield or Mote. Her first weapon and shield come later.
- Tor's items during Act 2: his whole Act 1 holdings, including the body entry (item 119 unless he replaced it), are in the stash, and his own entries in the party inventory are empty (he shows outfit 0 if he appears). The Act 3 start (stage 3) merges the stash back: for each hero, a category-1, 2, 5 or 9 entry that is empty in the party inventory takes the stash entry, so Tor's weapon, shield and body item return, while Roberta's non-empty entries stay; the general, quest and charm records of the stash are added through the normal add, the wallets are added together and the stash is cleared (SPEC-0025, Item lists).
- No other step adds item 119. No story event, save, load or equipment change gives it. It stays in the body entry until a body item with a higher id replaces it (the replaced item is not kept anywhere), until the Act 2 start moves it to the stash, or until the Tattered Cloak is put on, when it moves to the cloak entry and comes back when the cloak is taken off (Tattered Cloak). A save stores the body entry in the container tail, and a load adds it back (SPEC-0025), so the starting outfit is part of every save.
- There is no unequip action for body items. An outfit can only change by an add of a higher-id body item (Party inventory lists) or by the cloak swap.

### Counts (all 208 config records)

| Class | Count | Ids |
| --- | --- | --- |
| Obtainable in play (an unconditional source: a flag-0 or flag-1 direct placement in a reachable pack, a non-debug script grant, the coin spawn of item 139, the new-game items, or the Act 2 start kit) | 112 | all ids not in the rows below |
| Obtainable in play, conditional only (drop-group roll, a flag 2 to 4 placement, a placement in an unnamed region, or a coin split) | 16 | 8, 12, 13, 18, 29, 69, 70, 76, 78, 79, 80, 82, 112, 124, 148, 178 |
| Obtainable only in unreachable packs | 4 | 68 (pack 1170), 101, 105 and 111 (pack 1259) |
| Debug-only | 0 | none |
| Dialogue give only (a give-item command, kind 0x13 of SPEC-0022, in a line of the pack named in "Dialogue grants" below) | 18 | 1, 16, 17, 21, 23, 33, 38, 64, 65, 66, 67, 71, 74, 75, 84, 85, 87, 128 |
| No source | 58 | 43 blank or placeholder rows and 15 named items |
| Total | 208 | |

Blank or placeholder rows (43): 28, 48, 88 to 97, 134, 142, 158, 159 and 181 to 207.

Named items with no source (15):

- Story and quest items (9): 31 Jerdana's Orb; 35 Crimson Ergyfruit; 37 Sir Not Appearing; 44 Fauna's Squeaky Toy; 45 Torakar's Trident; 46 Kelland's Lockpick; 47 Gordos's Coin; 63 Hourglass Amulet; 81 Signet Ring.
- Equipment (1): 179 Lexicon of Defense. (127 Diplomat Dress is the Act 2 start kit; Starting equipment.)
- Tal currency (4): 140 Tal; 141 5 Tal; 143 25 Tal; 144 100 Tal. No placement, drop sub-table row, script grant, dialogue give or fixed code id names them (High).
- Item 180 Aegis Omnibus: only the boot grant handler (above).

Dialogue grants (High for the commands; whether each pack is reached follows the pack classes of Open question 90; every pack named in the table is reached under the 174-pack reach). These ids have no placement and no script grant; their only grant is a give-item command (count 1) inside a dialogue line, run when the line is shown (SPEC-0022):

| Id | Item | Pack (speaker) |
| --- | --- | --- |
| 1 | Letter of Consent | 1148 (Officer Kendrick) |
| 16 | Rathbone Crypt Key | 772 and 991 (Prince Tourin) |
| 17 | Old Mine Key | 748 |
| 21 | Letters of Devotion | 777 and 793 (Princess Dona) |
| 23 | Golden Trove of Riches | 1017 (Guildmaster Marl) |
| 33 | Strange Package | 1358 (Postmaster Tilbar; eight lines) |
| 38 | Signed Sponsorship Paper | 1150 (Master Torak) |
| 64 | Regal Stationary | 787 |
| 65 | Lord Cedrick's Invite | 777 |
| 66 | Lady Deborah's Invite | 777 (Lady Felina) |
| 67 | Lady Margo's Invite | 777 (Lady Felina) |
| 71 | Stolen Loot | 893 |
| 74 | Broken Watch | 842 (Mayor Jurgin) |
| 75 | Pocket Watch | 909 and 951 (Papa Hubert; two lines each) |
| 84 | Creepy Package | 1017 (Guildsman Guinoch) |
| 85 | Box of Spare Parts | 1017 (Guildsman Fearghas) |
| 87 | The Bandit's Plans | 784 |
| 128 | Neophyte Garb | 1327 |

Twelve of these (1, 16, 21, 23, 33, 38, 66, 67, 74, 75, 84 and 85) are also removed or tested by scripts; the dialogue give is the grant those scripts expect. The element-pair altar in level module 1382 sets only Mote ids 160, 162, 164, 166, 168 and 170, so it plays no part here.

## Neopoints

Neopoints (NP) are the game's money. This chapter gathers every rule about them in one place: the wallet, every way NP comes in and goes out, theft, the display, and the fixed amounts on the disc. Details that other sections own are pointed to, not repeated. Shop stock and prices are in SPEC-0050; the wheel and the betting race are in SPEC-0059.

Confidence: High for the wallet rules, the coin values, the money roll and coin split, the 50-slot limit, the steal rules, the buy path and the save fields. Medium-High for the dialogue pay-out amounts (the amounts are disc data; High). Medium for which dialogue pay-out lines the game shows (some lines are chosen by a computed index) and for the totals (expected values, not a measured playthrough).

### The wallet

- **Where.** One signed 32-bit word at +0x14 of the party item container. Both heroes' inventory links point at that one container, so the wallet is shared by the whole party; there is no per-hero purse (High). The Act 2 stash is a second container with its own word at +0x14 (the stash wallet). Nothing reads the stash wallet while it is held except the save and the Act 3 merge.
- **No cap.** No maximum value exists anywhere in the program. A coin pickup, a reward and the Act 3 merge are plain additions. The only guard is on the shared add used by the script give call and the boot grant codes: after the addition, a result below zero is stored as 0, so a total past 2,147,483,647 would be stored as 0 there. A coin pickup has no such guard (it would wrap negative); both cases are unreachable in play (the whole disc holds under about 200,000 NP). A clean implementation may use a wider integer, but must not clamp lower than 999,999,999 (the widest value the nine-cell display can show).
- **How each route changes it** (High):

| Route | Rule |
| --- | --- |
| Coin pickup, coin grant (item ids 135 to 139) | Wallet = wallet + value (Coin pickups). No floor, no cap. |
| Script GiveNeopoints, boot grant codes | Amount above 0: wallet = wallet + amount; a negative result is stored as 0. |
| Script RemoveNeopoints | Amount above 0: wallet = wallet - amount, never below 0. A removal larger than the wallet leaves 0. |
| Neopoint steal (Behavior 11a of SPEC-0029) | Amount taken = the draw, or the whole wallet when that is less; wallet = wallet - taken. |
| Shop purchase | After the item add succeeds: wallet = wallet - price. Not floored; a stored price of 0x8000 or more is negative and raises the wallet. |
| Act 2 start | Stash wallet = party wallet; party wallet = 0; then the kit adds 1150. |
| Act 3 start | Party wallet = party wallet + stash wallet (a plain addition). |
| New game reset | Both wallets 0. |
| Save and load | Both words are stored and restored raw, with no clamp (SPEC-0025: party wallet at slot offset 0x200, stash wallet at 0x204). |

- **Values at the start of each act.** New game (Act 1): 0. Act 2 start: exactly 1150, whatever Act 1 left (the Act 1 wallet waits in the stash). Act 3 start: the Act 2 wallet plus the stash wallet. Act 4 start: unchanged. A game started from the start-point chooser runs only that act's start step: a chooser game at Act 2 has 1150, and chooser games at Act 3 or Act 4 start with 0 (Medium-High; the chooser applies no money of its own).
- **Query.** The query returns the wallet, or 0 when no container exists. The hero argument of the script calls is ignored.

### Display

- **No in-play counter.** The wallet is never drawn during play on its own. Three places show NP:
  1. **Coin gains.** Each coin add writes its value to price caption line 1 (font 0, right-aligned at x 410, y 115). If line 1 is still showing (its counter above 0) the new value is added to the shown number, otherwise it replaces it; either way the counter is set to 180 draw passes (3 s). So quick pickups show one running total that disappears 3 s after the last coin. The one-Neopoint coin model (item 135) spins beside the caption while it shows: a lit, depth-tested 3D model centred at 0.65 of the screen width and 0.24 of the screen height (about (333, 108) on the 512 by 448 canvas, 77 pixels left of the caption's right edge), 6.0 units in front of the camera, scale 0.5886, turning about the vertical axis at 1 radian per second (SPEC-0035 section 15.2). There is no count-up animation: the shown number jumps by the gain.
  2. **Buy prompt.** Line 1 shows the price, line 2 (y 140) the balance, both as plain numbers with no label, for as long as the prompt is up (SPEC-0050 section 9). Line 1 turns to the second colour when the price is above the balance.
  3. **Item screens.** The Items, Charms and Quest tabs draw a "NeoPoints" label (centred near x 102, y 324) and the balance (about x 68, y 342) with the shared nine-cell number formatter (SPEC-0023 section 34). The value is drawn as it is; no animation.
- **Sounds.** A coin pickup plays cue 0xA5. A coin bounce plays cue 0xA7 (only a bounce with a landing speed of 1.0 or more). A purchase plays no cue of its own. Nothing plays when NP is removed by a script.
- **"Not enough Neopoints."** There is no general message. The buy path has three refusal boxes, spoken by the active hero through the shared box: "I could never afford that!" (price 32767, tested first), "I need another " + shortfall + the NeoPoints logo glyph + "!" (the glyph is character code 36 in font 0, shown as the NeoPoints logo), and "I can't buy that now!" (the add failed). Script shops have their own lines in the pack's text: the "$" in a line is the same logo glyph (for example "Only 1000$"), and each shop has its own refusal line. The wheel and the betting race refuse with their own lines (SPEC-0059).

### Sources

Every NP source is a coin or a coin-valued reward. There is no selling; no routine turns an item into NP (High).

| Source | Amounts | Rule |
| --- | --- | --- |
| Enemy death | The enemy's drop placement: group money, else the fixed count; plus carried NP | Money roll and coin split below. Direction none: every coin launches at once in a random direction. |
| Breakables | The drop placement named by the breakable's drop id | The same drop action, at the break position. |
| Chests and dig sites | The reward placement | The same drop action with a launch direction, so coins launch 0.4 s apart (SPEC-0048). |
| Placed coins | Coin items 135 to 139 as direct placements | Pickup only. 79 flag-0 coins in reached packs, worth 79 NP in all. |
| Script coin spawns | Pack 1397: 21 + 12 coins of 100 | 2,100 NP at the first trigger and 1,200 NP at the second (3,300 NP). Placed coins: they never move and never expire. |
| Dialogue pay-outs | 50 to 2,500 each | A give-item command for item 135 whose count is the amount (table below). |
| Act 2 kit | 1150 | Item 135 with count 1150 in the Act 2 start. |
| Prize wheel and betting race | 20 to 700 per game; odds x 100 | Coins through the hero's add (SPEC-0059). |
| Boot grant codes | 1,000, 1,000, 10,000 | Never called in the game (SPEC-0050 section 7). Do not implement. |
| Carried NP of an enemy that stole an item | Catalogue price of the item | Paid out as coins at its death (Theft). |

- **Money roll** (High; the formula is in Drop and money flow). For a placement with key -1 and a drop group 0 to 6: draw r in 0 to 99; r below the group's money chance gives an amount drawn uniformly in the group's money range (both ends inclusive); otherwise the amount is the placement's fixed count when it is above 0, else there is no amount and the item roll runs. A group of -1 has no money roll. The groups on disc:

| Group | Money chance | Money range | Mean NP per roll (chance x mean, plus the fixed count on a miss) |
| --- | --- | --- | --- |
| 0 | 50% | 25 to 75 | 25 (plus the fixed count half the time) |
| 1 | 40% | 300 to 800 | 220 |
| 2 | 40% | 10 to 40 | 10 |
| 3 | 30% | 400 to 1000 | 210 |
| 4 | 35% | 100 to 300 | 70 |
| 5 | 50% | 1 to 5 | 1.5 |
| 6 | 25% | 1 to 5 | 0.75 |

  No hero stat, luck value, difficulty or enemy level changes a chance, a range or an amount. The money is never scaled by the enemy's row: the only per-enemy inputs are its drop placement ordinal and its carried NP.
- **Coin split** (High). The amount is divided into 100, 25, 10, 5 and 1 coins greedily, each count being amount / denomination with the remainder carried down; each coin is one world item (ids 139, 138, 137, 136, 135). A drop with a direction launches coin k at 0.4 s x k after the first; with no direction (enemy death and carried NP) every coin launches at once. The coin items start at scale 0.25 and grow to 1.0 over 0.5 s.
- **50-slot limit** (High for the pool and the loop; Medium for how many slots are in use at a given chest). All coins of a split are requested in one loop, each taking a slot of the world's 50-slot item pool at once (a coin waiting for its delay holds its slot). A request with no free slot is dropped silently and the NP is lost. So a large reward pays only as many coins as there are free slots: the 25,000 NP chest of pack 1096 asks for 250 coins of 100 and can pay at most 5,000 NP (50 coins), fewer when placed items or other coins occupy slots; the 6,000 NP chest asks for 60 coins and likewise pays at most 5,000 NP. Dig rewards of 4,118 NP need 46 coins and fit. A clean implementation that wants the original amounts must keep the 50-slot pool and this behaviour. Default if unresolved (rewrite choice): keep the pool limit and let the loop drop what does not fit.
- **Dialogue pay-outs** (High for the amounts; Medium for liveness). 67 dialogue lines carry a give-item command for item 135; the count is the NP amount. The command runs when the line is shown (SPEC-0022 kind 0x13). Lines by pack and activity:

| Pack | Speaker | Amounts (NP) | What earns it |
| --- | --- | --- | --- |
| 1023 | Shanra (Meridell arena) | 300, 500, 750, 1000, 2000 | Arena win prizes. After a win the prize line depends on the win flags: first win gives the Wooden Shield (item 106) when the hero holds neither shield 106 nor 107, 300 NP when the hero holds 107 but not 106, then 500, 750, 1000 and 2000 for the next wins, and 2000 again for every later win (the ladder flag stays set). A hero holding 106 gets no first-win prize line. (Medium-High) |
| 758 | Syd | 500, 1000, 2000 | Time-trial tiers (SPEC-0059). |
| 758 | Kescher | 100 each, 8 lines | Each vermin caught. |
| 991 | Luger, Vernan, Zippy | 1000, 500, 2000 | Race and ride prizes. |
| 722 | Haplo | 1500 | Returning his Mortog. |
| 772 | Isaac | 400 | The lighthouse. |
| 842 | Giovanni | 1600 | The lodestone. |
| 859 | Motata | 2500 | The Crimson Ergyfruit. |
| 893 | Douglas | 2000 | The Drackonack hunter's repair. |
| 934 | Addison | 100 | The scuttling noises. |
| 956 (twin 970) | Celestia | 1200, 800, 800 | Brown, green and black feathers. |
| 1066 | Prince Tourin | 1019, 350 | The love letters and the rose. |
| 1136, 1140, 1143, 1148, 1184, 1251 | Gamon, Morag, Angus, Kendrick, Giovanni, Sophie | 500, 750, 500, 550, 800, 400 | Quest hand-ins. |
| 1023, 1172, 1176 | Sowter | 50, 100, 200, 400 | Weeding bets. Liveness Medium-Low. |
| 745, 777 | Fautt, Felina | 750, 500 | Unreached packs. |
| 1176, 1350, 1371 | Shanra, Sevaric | 200 to 2000 | Arena announcer lines in packs the game does not run live; not shown in play (Medium-Low). |

  The one-off quest rewards of the reached packs add up to 15,769 NP. With the competition prizes (vermin 800, time trial 3,500, races 3,500, arena ladder 4,250 before the repeat), dialogue pay-outs are about 27,900 NP. The arena's repeated 2,000 is unbounded.
- **Total obtainable.** Expected NP from every drop placement the game can name, in the reached packs, with every one triggered once: enemies about 38,000; breakables about 11,000; chests and dig sites about 84,000 (about 63,000 after the 50-slot limit on the two largest chests); plus 79 placed coins, 3,300 from pack 1397, 1150 from the Act 2 kit and about 27,900 of dialogue pay-outs. A thorough player can take about 145,000 NP over the whole game, most of it from dug treasures and the two big chests. Wheel and race play is a net loss for a random player (SPEC-0059). (Medium: expected values.)

### Sinks

- **Shops.** A flag-1 item placement sells at its fixed count (20 to 10,000 NP; four times that in five packs before the Black Knight is beaten). Rules, stock lists and the buy hooks are in SPEC-0050; the refusal order is in Shop purchase rule.
- **Script payments.** A yes answer removes the amount; there is no refusal charge. The sites on disc: the prize wheel (15 in pack 1023, 25 in 758, 100 in 1358); the betting race (50 per bet in 722); Rory the Shady Guy in pack 1023 (1000 for the dungeon way-in, which needs the Tattered Cloak worn and story flag 304 clear and sets flag 13; 200 for each of maps 13, 14, 24, 28 and 46 and 300 for map 16 after flag 304; a hero short of 1000 gets his refusal line); Gord the Merchant in packs 880 and 1230 (250, 300, 400 and, in 1230, 500, by the maps held); Tark Guterrunner in pack 1148 (100 for the sword hilt); Matkin in pack 787 (500); the map sellers of packs 991 and 1353 (SPEC-0050 section 6); and Gill the Fence in pack 1139. **Gill's quirk:** he offers one of five items (50, 152, 54, 55 or 56; picked at random once per module life) with a different quoted price each (200, 650, 350, 450 or 550), but the code tests and removes 500 NP for every one. Reproduce the 500.
- **None.** There is no arena entry fee, no training fee, no bank, no donation and no tax. Nothing charges NP on a death or a load.
- **Boundary.** Every script payment tests balance below price as a refusal; a balance equal to the price passes and leaves 0.

### Theft

- **Who.** Neopoint steals: Thief (monster row 152) and Rogue (153), 10 to 100 NP, and Thug (154), 20 to 200 NP. Item steal: the plain Crokabek (136) and the Swamp Crokabek (138). Other rows carry unused steal data (SPEC-0029 Behavior 11a).
- **Rule** (High). When the steal attack lands, draw r in 0 to 100 (101 values); the steal succeeds when r is below the slot's chance (50 for all three Neopoint thieves, so 50 of 101). The amount is drawn uniformly from the slot's minimum to maximum (both ends inclusive; if minimum and maximum are equal there is no draw). The amount taken is that draw or the whole wallet if smaller; the wallet falls by the amount taken and never goes below 0.
- **What drops.** The amount taken is paid out at once as coins at the hero's position, with the steal coin split (Stolen Neopoints split into coins). Amounts that are not multiples of 5 are paid over: a stolen 47 comes back as 53 coin NP. The coins use the ordinary coin flight below (random launch directions, all at once). The thief does not carry the money: its carried NP is unchanged.
- **Can the hero get it back?** Yes. The coins are world items; the hero collects them like any coin (no attack or key is needed). They stay for 11.2 s of age, then fade for about 0.5 s. A Thief, Rogue or Thug that is more than 10 units from the hero goes for the nearest coin of each coin kind and deletes it when it arrives within 1 unit (the Thief and Rogue roll 49 of 100 and the Thug 99 of 100 per coin kind each tick). It takes any coin on the ground, not only stolen ones, and it does not keep the NP: the NP is gone. So a hero who stands near the coins, or fights on beside them, collects them back (plus the rounding overpay); a hero who runs more than 10 units away risks losing them.
- **Armour.** The Rogue's and Thief's steal slot needs a strike result of at least 1; their blow does 1 damage, and any body armour with an armour percent above 0 reduces it to 0, so an armoured hero is never robbed by them. A blocked blow does not steal. The Thug's steal is in its grab-and-throw and has no armour test: armour does not stop it.
- **Item steal.** The Crokabek's swoop takes one unit of item 150 or 149 the hero holds (100 of 101), carries it for 3 s (drawn at 0.45 scale) and its carried NP grows by the item's catalogue price (Item text database). At its death the carried NP is paid as coins; the item is not returned.
- **Enemy death payout.** Every death pays its drop placement (money roll above) and then splits its carried NP, on the same first tick; the carried NP word is then 0.

### Death and game over

Death takes no NP and no items. The respawn after a deadly-ground death costs up to 5 HP and nothing else (SPEC-0055). The game-over page's Continue loads the last save, so NP earned since that save is lost with everything else since it (High).

### Coin flight and collection (summary)

Full motion rules are in World items, Coins: ballistic flight. In short: a coin starts at scale 0.25 and grows over 0.5 s; it is launched with vertical speed 14 to 18 units/s and horizontal speed 1 to 3 units/s in its direction, under gravity -40; each ground contact multiplies the vertical speed by -0.65 (a landing of 1.0 or more plays cue 0xA7); the coin stops bouncing when the speed after the bounce is under 1.0 and then rests 0.6 above the ground, still sliding with a 2% loss per tick. It is ready to collect when its age is above 0.2 s. There is no magnet and no pull: a hero collects the nearest ready coin whose centre is closer than 2.675 units (1.5 + the hero radius 1.175), one coin per tick, whichever hero it is. The pickup test runs in every hero's tick, so the inactive partner collects too; the value goes to the one party wallet. A coin that is not collected lives 11.2 s of age (the age starts when its launch delay ends), then fades over about 0.5 s and frees its slot.

### Worked examples

1. **Enemy death, money roll and coin split.** An enemy whose drop placement has key -1, group 2 (chance 40, range 10 to 40) and fixed count 0 dies.
   - Draw r = 17: below 40, so money. Draw the amount in 10 to 40: 23. Split: 100s 0, 25s 0, 10s 2 (remainder 3), 5s 0, 1s 3. Five coins (10, 10, 1, 1, 1) spawn at the enemy's position and all launch at once in random directions. No item is rolled. The carried NP is 0, so nothing more spawns.
   - Draw r = 55 instead: no money and no fixed count, so the item roll runs. R = 100 - 40 = 60. Draw t in 0 to 59: 25. Cumulative weights: sub-table 1 (weight 20) then sub-table 4 (weight 10, cumulative 30): 25 falls in sub-table 4. Draw u in 0 to 99: 45. Sub-table 4 bands (162, 20), (164, 20), (166, 20), (160, 15), (168, 15), (170, 10): cumulative 20, 40, 60: item 166 spawns.
   - Disc example of a fixed count: a Thug in pack 888 has a drop placement of key -1, group -1, fixed count 299: 299 = 2 x 100 + 3 x 25 + 2 x 10 + 4 x 1, eleven coins.
2. **Coin pickup from spawn to collect.** A 10 NP coin (id 137) from the example above, with launch speeds at the centre of the draws (vertical 16.0, horizontal 2.0). It is spawned at the enemy's position with its delay at 0: on its first tick it launches. It first touches the ground about 0.78 s (tick 47) later and bounces at 9.97 units/s (cue 0xA7), then at about ticks 78, 100, 115 and 126 with speeds 6.95, 5.01, 3.24 and 2.66, and rests by about 2.2 s, having moved about 7 units (3.5 x the horizontal speed). Its age passes 0.2 s on about its 13th tick of age (the float sum falls just short of 0.2 at the 12th), so from then on a hero standing within 2.675 units collects it: wallet + 10, cue 0xA5, and price caption line 1 reads 10 (or, with the other four coins collected within 3 s of each other, a running total ending at 23). If nobody collects it, it fades out after 11.2 s and about 0.5 s.
3. **Shop purchase with too little NP, then enough.** Pack 1136, Dispelling Potion (item 52) at 125 NP; the hero stands before it with 100 NP. The prompt shows the price 125 in the second colour and the balance 100. Square: the box "I need another 25$!" (the shortfall 25 and the NeoPoints glyph) is spoken by the active hero; the wallet stays 100 and no add is attempted. With 150 NP: Square adds the potion (a copy: the potion stays on display), the wallet becomes 25 and no message shows. Buying again with 25 NP is refused with "I need another 100$!". With a price of 32767 the first refusal "I could never afford that!" comes before any balance test.
4. **Theft by a Rogue and the coins back.** The hero (no body armour) has 500 NP and is struck by the Rogue's steal slot. Chance draw r = 30: below 50, so the steal succeeds. Amount draw 47 in 10 to 100: the wallet becomes 453. The 47 is paid as coins 25, 10, 10, 1, 1, 1, 5 (the 1-coins round the last 2 up through a 5): 53 NP of coins at the hero's position. If the hero stays, he picks them up from about 0.2 s after the steal as they land, one per tick, and the wallet reaches 506 (a net gain of 6). If the hero runs more than 10 units away, the Rogue's coin search fires (49 of 100 per coin kind per tick), the Rogue runs to a coin, deletes it on arrival and then walks home; each deleted coin's NP is gone for good. A chance draw of 50 or more takes nothing and spawns nothing.

### Mistakes to avoid when porting

1. Giving money only to the active hero or keeping a wallet per hero. There is one party wallet, plus a stash word for Act 2.
2. Reading coin values from the item text table. The table has a 10 NP coin with a blank price and a name; the values are 1, 5, 10, 25 and 100.
3. Capping the wallet, clamping a coin pickup at 0, or flooring a purchase. Only remove and give floor at 0.
4. Paying a stolen amount exactly. Amounts that are not multiples of 5 come back larger.
5. Giving carried NP to a Neopoint thief, or letting it keep the stolen money. The thief never holds it; coins left lying are deleted, not kept.
6. Treating the steal as "50 percent". It is 50 of 101 draws.
7. Skipping the 50-slot item pool: without it the 25,000 NP chest pays all 25,000.
8. Charging Gill the Fence the quoted price instead of 500, or forgetting that the first Meridell arena win gives a shield (or 300 NP), not 500.
9. Using a pull or magnet for coins, or an expiry other than 11.2 s after launch.
10. Adding NP on death, charging a continue, or showing a persistent counter in play.
11. Starting Act 3 without the stash wallet, or Act 2 with the Act 1 wallet instead of exactly 1150.
12. Using the money roll draw as 1 to 100 or 0 to 100: it is 0 to 99, compared strictly below the chance. The steal chance uses 0 to 100.

Default if unresolved (rewrite choice): show every dialogue pay-out line that has a live call, plus the quest turn-ins whose line index is computed; omit the arena-announcer lines of packs 1176, 1350 and 1371. For the Sowter bets use the amounts on disc when his talk is reachable.

## Edge cases and error handling

- **Luck 0**: L = 0, so every gated pickup is vetoed. Luck above 24 gives L above 100, so the gate never vetoes once L exceeds 100.
- **Negative key on flag 0 placements**: 60 placements on disc have flag 0 and a negative key. At region start-up the spawn is asked for a negative item id, which it refuses: nothing spawns and an error is logged (High).
- **Equipment ranking by id**: a pickup of id 127 (type 1, stat A 0) is allowed over equipped id 125 (type 4, stat A 70) because 127 is greater. The original does this. Clean code must use the id rule.
- **Id 134** (type 1, all stats 0) is ranked by id like any other item.
- **Unreachable entry**: group 0's third entry (sub-table 5, weight 10) can never be chosen, because the first two entries already cover all 50 item-roll values.
- **Group −1**: no money roll and no item roll. Only the fixed count can spawn coins.
- **Fixed count with money**: if the money roll succeeds, the fixed count is ignored.
- **Ids above 322** from a sub-table or script are not handled by the drop path. No sub-table entry has one.
- **Quick-item panel selection kept**: a tick with the panel open and no D-pad press keeps the current selection; a Down press selects entry 3 (Medium-High; Quick-item panel).
- **Quick-item panel with no selection**: Circle or Cross does nothing and the panel stays open. Circle with a selection closes the panel without using an item.
- **Second panel, press with no target**: a Square press while the hero's category-2 slot is empty and no Circle press equips nothing, and the direction is still ignored in that tick (Second quick panel and Mote equipping).
- **Second panel, same Mote pressed twice**: pressing the Mote already in the target slot clears the slot. Cue 0xA4 plays if the slot held something.
- **Second panel, unowned Mote**: the selection can rest on an unowned Mote. A press on a slot with that selection clears the slot.
- **Diverted item-screen use** (a use hook is registered): plays cue 0xA2 and changes nothing. The hook receives the id; the game never applies that use later (Use-request gate and charm-only flag).
- **Text rows missing**: ids 181 and above have no name or price in table 0. Show the id instead.
- **World-item model, index-in-range quirk** (reproducible; Medium-High): the world-item display slot uses the index inside the id range, not the id. Spawning item ids 148 to 157 as world items also gives n = 50 to 59 and loads a potion model (148 loads the model of potion 50, and so on). The model registers under id 50 to 59, not under the item's own id, so it is never drawn for that item (Open question 120). Because there is only one world-item slot, the load can still evict a resident potion model. Item 148 has a conditional source (Counts), so this may show in play. By the same arithmetic ids 241 to 250 give n = 50 to 59; those ids have no config record.
- **One world-item slot** (reproducible; High): only one potion model is resident at a time. After a second potion kind is spawned, world potions of the first kind have no model until theirs is loaded again.
- **Weapon model for a non-weapon id** (reproducible; Medium): the weapon index rule covers every id up to 110, so a shield id (106 to 110) in the weapon-family entry would give w = 8 to 12 and load a wand model (552 to 556). Ids below 98 or above 118 load nothing.
- **Negative price** does not occur in table 0 (all values are 0 to 15000) or in any flag-1 placement (20 to 10,000).
- **Buy with price 0x7FFF**: never affordable, regardless of NP.
- **Price setter with no buyable object**: nothing is stored; the setter may print a debug message only (Shop purchase rule). A price of 0x8000 or above reads as negative (Shop purchase rule, step 4).
- **Item screen, empty cell**: Cross on a cell with count 0 does nothing except cue 0xA2. No use call runs.
- **Item screen, count display**: a count of exactly 1 shows no number (only counts above 1 show text).
- **Remove more than held** (reproducible): a removal larger than the wallet leaves 0, never a negative value.
- **Remove with a zero or negative amount** (reproducible): nothing changes. A negative remove therefore does not add money, and a negative give does not take money.
- **Give past the 32-bit limit**: the Neopoints value has no limit in the give rule, but a sum past the signed 32-bit range wraps negative and is clamped to 0 (Wallet).
- **No wallet object**: the query returns 0. What remove and give do in that case is not established (Open questions).
- **Enemy projectile hits**: no item is spawned on any projectile hit, blocked or not; the roll there is a status roll (Enemy projectile hits; SPEC-0029).
- **Cue bag wrap**: index 12 reads the first drawn index (0 to 11), not a Mote id; the group add rejects it, so that group holds 7 entries and drains 7 Motes (SPEC-0029, Behavior 9, kind 14; Mote drain).
- **Pickup gate at the boundary**: the comparison is strictly "more than": an item whose age is exactly 0.8 s (or 0.2 s for a coin) is not ready.
- **Pickup gate, state 2**: never ready, whatever the age.
- **Coin ids 140 to 144**: they use the 0.2 s gate, but add no value to the wallet, and no source exists (High). Clean code should not invent a value for them.
- **Coin pickup with count 0**: id 135 adds its count, so a count of 0 adds nothing.
- **Petpet-reserved item**: while a petpet holds the reserved token of an item, no hero collects it (Auto-collect, step 2).
- **Quick-item auto-select**: an added item with stat E of 1 or more becomes the quick-item marker only when the effective marker is empty; an existing marker is never replaced (Auto-collect, step 5).
- **Pool full**: with all 50 world-item slots in use, a further spawn of any kind (drop, coin, script or clover find) spawns nothing.
- **Ready on the 49th tick**: the age is a single-precision sum of 1/60 steps, so 48 ticks give 0.79999948, and a non-coin drop is first ready on its 49th tick of age (Pickup timing gate). An implementation that keeps the age as a tick count must use "more than 48 ticks" to match.
- **Fade length**: an expiring item fades for 28 ticks and is freed on the 29th (World items, Per-tick step 3).
- **Important-item double give** (reproducible quirk): an important dropped item whose carry limit is not 1 can be added twice (Important-item give).
- **Placed items**: region-load items and all script AddItemToWorld items never age, never expire and are never range-collected.
- **Stat-up flags**: each add applies its flags again, but each stat-up item is accepted at most once per world id (Stat-up items, step 6). A second one in the same area is refused and not spawned. The add applies no cap, and the load-time caps in SPEC-0036 apply only when a save is loaded (Medium). Clean code should not cap the add without a source.
- **Heal streak at the boundary**: a heal at exactly 4.5 s after the last heal of the same pool restarts the accumulator (Open question 34).
- **Heal streak reset**: after the reset to 0, a heal inside the window adds to a zero accumulator, so the count starts again from that heal.
- **Heal streak after stat-up**: stat-up items with health or magic flags update the corresponding maximum and fully refill that pool (Stat-up items). A subsequent consumable heal compares against half of this updated maximum. The stat-up add does not run the consumable heal handler; other effects of the maximum-change callback are not established. Test vector: after a Golden Negg raises Tor's health maximum to 175, the next health-heal streak threshold is 87.5.
- **Armour Mote without magic or charge**: when pool B is below the armour Mote's cost, or its power Q0 is above its remaining charge, the (100 - H) step does not run and nothing is spent; the body item's stat A still applies.
- **Mote spent**: when a use is at least the remaining charge, the Mote leaves the inventory and every equipment word holding it becomes -1 in the same step; a Mote with 1 charge left is removed by any use.
- **Shield charge minimum**: the shield-Mote charge is at least 1 even when the shield's charge stat is 0 (type-2 shields), so every blocked hit whose element passes spends 1.
- **Shop pre-check busy**: the purchase is queued, not dropped. It runs again later without the check, unless the queued request's own flag is set. No price is taken when it is queued (Shop purchase rule, step 2).
- **Empty item slot**: a slot whose pointer is 0 or whose count is 0 or less is empty and is skipped by the queries.
- **Slot beyond the count** (High): cannot occur. Every list's count is a fixed capacity set at boot, and removal clears a slot in place without changing the count (Party inventory lists).
- **Same id in two slots**: the first match in the search order is returned. In categories 7 and 8 the add can create such a pair (Party inventory lists, duplicate-slot quirk); the later slot's count stays hidden until the earlier slot is emptied.
- **Zero from the count query**: it means no non-empty slot holds the id, because a slot with a count of 0 or less is empty.
- **Display index above 110**: a kind-1 id above 110 subtracts 103, so id 111 gives 8. Ids 106 to 110 give kind-2 indexes 0 to 4 only.
- **Enemy drop ordinal -1**: nothing spawns and no coins are paid. A stored -1 is out of range whether it is read as signed or as unsigned (65535), so both readings give nothing.
- **Locator index out of range**: an index below 0, or at or above the locator list's length, spawns nothing (the world spawn call of Coin spawn).
- **Steal amounts that are not multiples of 5**: the coins overpay by 2 to 8 NP (Stolen Neopoints split into coins).
- **Bit set with more than 255 entries**: the count caps at 255. Bit 255 is never written, and bits at or above the count keep their old values.
- **Reach index is not bounded.** For Tor the index is id minus 98 with no upper limit: ids 109 to 131 read the second and third stored copies of table A (so a Tor reading for wand id 111 gives the copy's third entry), and higher ids read the data after them. For Roberta the index is id minus 111 with no lower limit, so sword and shield ids 98 to 110 read the words before table B (the end of the third copy of table A and the words after it), and ids 119 and above read the words after table B, starting with the length factor 0.6 at 0x2DD690 for id 119. These mismatched ids do not enter the weapon list through ordinary inventory add or save-restore paths: types 6/3 route to Tor/Roberta respectively, and the direct weapon-id ranges are 98 to 105/111 to 118. The raw reads remain possible with malformed save state or an external write; no behavior for that case is specified.

## Test vectors

Synthetic and hand-checkable. Random draws are given as the value the draw returns.

1. **Luck gate**: luck 12, gate on. L = trunc(12 × 4.166667) = 50. Draw r = 49: pass. Draw r = 50: vetoed. Pass chance 50/101.
2. **Luck gate edge**: luck 0 gives L = 0; any draw is vetoed. Luck 24 gives L = 100; only r = 100 is vetoed.
3. **Money split**: group 0 (chance 50). Draw r = 49 (below 50): money. Draw amount = 62. Coins: 62 / 25 = 2 rem 12 (two of item 138); 12 / 10 = 1 rem 2 (one of item 137); 2 / 5 = 0; 2 / 1 = 2 (two of item 135). Total 25 + 25 + 10 + 1 + 1 = 62.
4. **Item roll, first band**: group 1 (chance 40). Draw r = 73 (at least 40, so no money). t = 30 (draw from 0 to 59). Entries: (5, 25) covers 0 to 24; (2, 20) covers 25 to 44. So sub-table 2 is chosen. Its only entry is item 152 at weight 100, so draw u = any value from 0 to 99: item 152.
5. **Item roll, second entry**: group 1, r = 73, t = 50. Entry (6, 15) covers 45 to 59, so sub-table 6. Entries (172, 75) and (173, 25). Draw u = 80: u is at least 75, so item 173.
6. **Unreachable entry**: group 0, no money (draw r = 60), item roll t = 45. The cumulative bands are 0 to 19 (sub-table 0), 20 to 49 (sub-table 4) and 50 to 59 (sub-table 5). Here t = 45 picks sub-table 4. Sub-table 5 can never be picked because t never reaches 50.
7. **Equipment veto**: family 1/4 with equipped id 125. Pickup 127: allowed. Pickup 125: vetoed (equal). Pickup 119: vetoed.
8. **Heal, pool A, character 0, item 146** (20 percent): trunc(500 × 20 / 100 + 1) = 101.
9. **Heal, pool A, character 1, item 146**: trunc(400 × 20 / 100 + 1) = 81.
10. **Heal, pool A, item 58** (forced 100 percent), character 0: trunc(500 × 100 / 100 + 1) = 501, then clamped to the missing amount.
11. **Heal, pool B, character 0, item 147** (50 percent): trunc(300 × 50 / 100 + 1) = 151.
12. **Heal, pool A, character 0, item 153** (15 percent): trunc(500 × 15 / 100 + 1) = 76.
13. **Shop, price over wallet**: price 600, NP 500. Result: no purchase, message "I need another 100".
14. **Shop, purchase**: price 300, NP 500, add succeeds. Result: NP becomes 200.
15. **Shop, never affordable**: price 0x7FFF. Result: "I could never afford that!", no purchase.
16. **Placement flags at start-up**: a kind-8 placement with flag byte 2 is not spawned at region start-up, but a scripted add, or a breakable break or enemy death that names it, still spawns it.
17. **Item screen use**: Cross on a cell whose count is 0 plays cue 0xA2 and runs no use call. Cross on a cell whose count is 2 that passes the party check runs the use call for that id once.
18. **Wallet, remove over the balance**: Neopoints 500, remove 800: result 0.
19. **Wallet, remove with a negative amount**: Neopoints 500, remove -50: result 500. Remove 0: result 500.
20. **Wallet, give**: Neopoints 500, give 2000: result 2500. Give -10: result 500.
21. **Wallet, shared**: Neopoints 500. A hero-independent give of 50 followed by a remove of 200: result 350. The same value is seen whichever hero the script names (the hero argument is ignored by these three methods).
22. **Wallet, no object**: the query returns 0.
23. **Coin flight** (single-precision, dt = 1/60, flat ground at y = 0): coin 137 spawned at (0, 1, 0) with direction (0.7071, 0, -0.7071), delay 0, and launch draws giving vy 16, hs 2 and spin 0.4 pi. Tick 1: state 2, y 1.2667, vy 16.0, x 0.0236. Tick 25: state 3 (the apex), y 4.3333. Tick 52: first ground contact, cue 0xA7, vy 11.70, hs 1.96, spin 1.2315, y 0.6283. Later contacts at ticks 88, 112, 129, 142 and 152. At rest by tick 160 (y 0.6000, vy 0, hs 1.6341); tick 190: hs 0.8914. Ready from tick 25 (state 3, age 0.4167 > 0.2).
24. **Coin delay**: the same coin with delay 0.4 s is drawn at scale 0 and does not move through tick 23. It launches on tick 24 (the launch draws are made then), and its age starts growing on tick 25.
25. **Hovering potion**: potion 146 dropped at (0, 1, 0) over flat ground at y = 0, Larg -1 (L = 11.2 s). Reference 0.2. Tick 1: y 2.28 (state 2). Tick 2: state 3. Tick 4: state 4. It then bobs between about 1.92 and 2.28 with a period of about 90 ticks. Ready from tick 49 (after 48 ticks the single-precision age is 0.79999948, not above 0.8). The fade starts on tick 673 (age 11.2167, life 0.9647), and the slot is freed (state 0) on tick 701. The important-item give runs then; item 146 has text flag 0, so nothing is added.
26. **Hopping charm**: charm 172 dropped at (0, 1, 0) over flat ground at y = 0, with spawn draws giving spin +2.0420 and vertical speed 3.0, and a first contact draw giving bounce speed 4.0. Tick 1: y 1.98. Tick 2: state 3. From tick 3 the vertical speed falls by 0.2 per tick. First ground contact on tick 54 (y 0.1767): vertical speed 4.0, spin -2.0420 (the new sign drawn negative), state 4. It then hops about every 40 ticks.
27. **Mote drain** (synthetic): the party holds Mote 163 with charge 100 and Mote 160 with charge 15, both drawn into a mote-eater group. When their entries spawn, 163 falls to 80, and 160 is removed and cleared from the active hero's equipment words (cue 0xA4 if it was equipped).

28. **Coin pickup, id 136** (synthetic): wallet 100, one coin of id 136 is collected. Result: wallet 105, no coin item kept.
29. **Coin pickup, id 139 and id 135**: wallet 100, coin 139 collected gives 200. Then coin 135 with count 3 gives 203.
30. **Coin pickup, id 140** (synthetic, Low): wallet 100 unchanged. Clean code must not guess a value.
31. **Pickup gate**: state 1 with age 0 s is ready. State 2 with age 5 s is not ready. A non-coin item in another state at age 0.8 s is not ready, and at 0.81 s is ready (strictly more than). A coin of id 137 at 0.25 s is ready, and at 0.19 s is not ready.
32. **Heal streak, pool A, hero 0** (synthetic pool maximum 200; half is 100): heal 40 at t = 0 s gives accumulator 40, no reaction. Heal 70 at t = 3.0 s (within 4.5 s) gives 110, which reaches 100: reset to 0 and reaction 0x23. Heal 30 at t = 8.0 s (gap 5.0 s) restarts at 30, no reaction. Heal 30 at t = 11.0 s (gap 3.0 s) gives 60, no reaction.
33. **Heal streak, pool B, hero 1** (synthetic pool maximum 100; half is 50): heal 30 at t = 10 s gives 30. Heal 25 at t = 12 s gives 55, which reaches 50: reset to 0 and reaction 0x3C.
34. **Stat-up items**: hero 1 picks up an item with the first flag: pool A value +20. Hero 0 picks up an item with the second flag: pool B value +15. Hero 1 picks up an item with the second flag: pool B value +25.
35. **Consumable cure**: item 54 used on a hero with status 2 (poison) active clears statuses 1, 2 and 3 and plays cue 0x9F. Item 58 used successfully plays 0xA0 instead.
36. **Armour Mote gate** (synthetic): hero with pool B current 3 and an odd armour Mote (cost 4) in word 0: the Mote does not act and pool B stays 3. With pool B 60: pool B becomes 56, the hit is scaled by 25 percent and 5 charge is used.
37. **Category 1 queries** (synthetic): the category 1 list (capacity 2) has slot 0 empty and slot 1 holding Roberta's wand 112 with count 1. Query 112: the slot query returns slot 1 and the count query returns 1. Query 98: both return 0.
38. **Remove in place** (synthetic, category 7): a slot holds a quest id with count 5. Remove 3: count 2. Remove 2: the slot is cleared (pointer and count 0), the slot stays at its index and the list's count field stays 60.
39. **General-list add** (synthetic): slot 0 is empty and slot 4 holds item 146 with count 2. Adding 146 x1 gives slot 4 a count of 3 (the same-id pass runs first). Adding 153 x1 then fills slot 0.
40. **Duplicate slot** (synthetic, category 8): slot 0 holds Fire Mote 160 and slot 1 holds Water Mote 163 with charge 40. Fire's charge is spent to 0, which clears slot 0. A new Water Mote is picked up: slot 0 gets 163 with 100, slot 1 keeps 163 with 40, and the count query for 163 returns 100. After slot 0 is spent to 0, the query returns 40.
41. **Quick-item panel selection** (synthetic): the panel is open and no entry is selected (value 4). A Right edge selects entry 0 and plays cue 0x9A. A following Cross edge sets the marker to entry 0's id (cue 0xA3) and uses that item. A tick with no direction keeps the selection.
42. **Quick-item panel, no selection** (synthetic): the panel is open with value 4. A Circle edge or a Cross edge does nothing and the panel stays open.
43. **Mote equip cue** (synthetic): the hero holds a weapon, and the Mote panel equips Mote 161 into word 2 while word 2 holds 160: cue 0xA3 plays and the word-2 panel timer is set to 240. Pressing Circle again with 161 selected clears word 2 with cue 0xA4.
44. **Pad mask** (synthetic): the first panel is open and Cross is pressed with a selection, which closes the panel. While Cross stays held, the hero's control step does not see it; once it is released and pressed again, the hero sees it. The analog stick moves the hero throughout.
45. Kind pick: id 0xA3 gives (1, 2). Id 0xA4 gives (1, 1). Id 0xA0 gives (1, 1). Id 0x9F gives (0, 0).
46. Display index: kind 1 with id 110 gives 12, with id 111 gives 8, and with no item gives -1. Kind 2 with id 106 gives 0, with id 110 gives 4, and with id 111 gives -1. Model resources: weapon id 100 loads 546 (display id 100); weapon id 111 loads 552 (display id 111); weapon id 118 loads 559; weapon id 119 (w = 16) loads nothing; shield id 108 loads 562 (display id 108). World item id 55 (range 0 to 97, n = 55) loads 570 with display id 55; world item id 150 (range 98 to 190, n = 52) loads 567; world item id 160 (n = 62) loads nothing. World item id 152 (n = 54) loads 569, which registers under id 54, so item 152 is not drawn with it. Item 36 model: resource 575, display id 36, once.
47. Weapon reach: empty weapon entry gives 1.5. Tor with item 98 gives 1.7 (table A, index 0). Tor with item 105 gives 2.87 (table A, index 7). Roberta with item 111 gives 1.2 (table B, index 0). Roberta with item 119 reads index 8, which is the word after table B (0.6, the length factor), not a table entry.
48. Bit set (synthetic): from all-zero bits, a list of 300 entries where entries 0 and 9 pass gives byte 0 = 0x01 and byte 1 = 0x02. Byte 31 keeps its prior value.
49. Quick select (synthetic): panel open, selection 4 (none), entry 0 holds an owned item. A Right edge selects entry 0 and plays cue 0x9A. A second Right edge refills entry 0 with its next owned candidate and plays 0x9A. A Circle edge then sets the marker to the entry's id and plays cue 0xA3.
50. **Mote facts** (synthetic, from the Motes table): id 164 gives element kind 2 with stat F 15, G 50, H 50. Id 173 gives kind 7 with stat F 0, G 100, H 75. Id 111 (a wand) gives kind 0.
51. **Armour Mote** (from the stored values): word 0 holds Mote 169 (odd family), body item 122 (stat A 40), enemy damage value 60, pool B 10: the Mote acts: 60 x 0.01 x 25 (about 15), then x 0.6 gives 8.999999, damage 8; cost 4 (pool B 6); charge 100 becomes 95. With pool B 3 it does not act: 60 x 0.6 = 36, damage 36.
51a. **Mote charge** (from the stored values, S = 100): a Tor swing that lands with Mote 161 uses 3 (100 to 97); a Tor power attack with Mote 160 uses 24; a Roberta full release with Mote 161 uses 40; a Roberta quick-shot hit with Mote 160 uses 2. A Mote with 3 charge left hit by a use of 3 is removed and its id is cleared from all three equipment words.
51b. **Uses per Mote** (charge 100): Tor swings, lesser 100, greater 34; Tor power attacks, lesser 5, greater 3; Roberta quick-shot hits, lesser 50, greater 25; Roberta full releases, lesser 4, greater 3.
51c. **Shield Mote** (from the stored values): shield 108 (stat A 40), blocked enemy hit of 60: word 1 empty, damage 20 and no charge; word 1 = Mote 161 with an attack element of 0 or 1, damage 10, 1 charge and 2 MP; word 1 = 161 with attack element 4, damage 10, no charge and no MP.
52. **Forced magic heal** (synthetic, starting maxima): Tor, pool B 10 of 90, item 152 (heal 301). The heal is not smaller than the missing 80, so it is refused unless the use is forced. Forced: pool B becomes 90.
53. **Wallet, give past the 32-bit limit** (synthetic): Neopoints 2147483647, give 1: result 0. The sum wraps negative and is clamped to 0.
54. **Shop, negative price** (synthetic): Neopoints 500, stored price 0x8000 (signed -32768), add succeeds. Result: the item is added and Neopoints become 33268.
55. **Region start-up luck draw** (synthetic): gate field nonzero, luck 12 (L = 50). Draw r = 49: passes the luck gate, and the flag test then decides. Draw r = 50: vetoed, nothing spawns.
56. **Scripted add, same numbers** (synthetic): gate field nonzero, luck 12 (L = 50), for a key of 0 or more. Draw r = 49 passes; draw r = 50 is vetoed. Same result as vector 55.
57. **Negative key at region start-up** (synthetic): gate field nonzero, key -5, luck 24 (L = 100). Draw r = 100 is vetoed. Any other draw passes the luck gate, skips the equipment and quest vetoes, and asks the spawn for id -5 directly with no group roll; the spawn refuses it, so nothing appears.
58. **Flag test after the draw** (synthetic): region start-up, gate field nonzero, luck 12, flag test fails. One draw is still taken, and nothing spawns.
59. **No gate** (synthetic): gate field 0. No draw is taken; the vetoes and the flag test run as usual.
60. **Second panel, from none** (boot direction table): selection 255. Left gives selection 12 (Mote 172). Right gives selection 13 (Mote 173). Up+Left gives selection 2 (Mote 162).
61. **Second panel, from Fire** (boot direction table): selection 0 (Mote 160). Right gives selection 1 (Mote 161).
62. **Second panel, button priority** (synthetic): second panel open, selection 0. A tick with Triangle and Right pressed closes the panel with no cue and the selection is not moved. A tick with Cross and Right pressed acts on slot 0 and leaves the selection at 0, because the direction is ignored.
63. **Use hook takes a use** (synthetic): a use hook registered with handled 1; item screen Cross on an owned potion 153. The dispatch returns 0: no heal, the count is unchanged, cue 0xA2 plays, and the hook receives id 153. With handled 0, after the hook returns one more dispatch of id 0 runs and plays cue 0xA2 again; the potion is still not used.
63a. **Hook thread busy** (synthetic): a use hook whose callback waits 2 s; uses of 153 and then 154 within that time. The hook runs once, and its block then holds 154. A third use after the thread has ended starts the hook again.
63b. **Coliseum block**: in pack 1371 with blackboard value (2, 157) = 20, the quick panel's use of potion 153 changes no pool and no count.
64. **Placement id table, pack 685** (from the disc; one region with six entries). Entries 0 to 5 each have region byte 0xFF and ordinal k, so entry k names the kind-8 record that is k-th in walk order. Those records are: ordinal 0 key 169, flag 0; ordinal 1 key 169, flag 0; ordinal 2 key 49, flag 3; ordinal 3 key -1, group -1, count 0, flag 3; ordinal 4 the same as ordinal 3; ordinal 5 key 18, flag 3. Region start spawns items 169 and 169 only. A breakable whose drop id is 3 or 4 gives nothing. A drop id of 5 spawns item 18 directly.
65. **Shop copy and take** (synthetic placements): flag 1, key 153, fixed count 75; Neopoints 500. Buy: Neopoints 425 and the world item stays; buy again: 350. Flag 1, key 114, count 3000, and a second record with key 114, count 10000, in the same pack: buying either one (Q = 1) removes both, and neither spawns on the next visit.
66. **Shop map** (synthetic): flag 1, key 175, fixed count 200, placement byte +0x2C = 0x12. Buying it gives map 19. With map 19 already held, the placement is not spawned.
67. **Once per area** (synthetic): Golden Negg bought in the world of pack 909 (world index 68, the key table of SPEC-0025): both heroes gain the health and magic steps. A second Golden Negg add in that world is refused. An add in another world id is accepted.
68. **Placed potion** (synthetic): a potion spawned by region load is in state 1 at its point. It does not move, spin or age on any tick, is always ready and never expires.
69. **Nearest-item search** (synthetic): the hero's motion record at (0, 0, 0) and one ready item with token 1 at (3, 1.9738, 4). Stored: token 1, unit direction (0.5581, 0.3672, 0.7441), distance 5.3755.
70. **Lifetime defaults** (synthetic): a drop of item 153 with Larg -1, not placed: L = 11.2 s. A drop of Mote 164: 15.2 s. Item 153 at region load: infinite. A script spawn of a coin with Larg 5.0 and placed 0: 5.0 s.
71. **Range collect** (synthetic): a dropped Black Feather (78, important) 10 units from the hero, L = 11.2 s. At age 4.5 s it is collected (4.5 > 4.48 and 10 < 25). At age 4.4 s it is not. A potion at the same place is never collected at 10 units.
72. **Pool full** (synthetic): with 50 live world items in a world, a 51st drop spawns nothing.
72a. **Release gives important drops**: drop a Venom Vial (id 70, important) in transition 691 and walk on into another exterior, so that a later streaming step replaces 691 in the transition buffer: the vial is added to the inventory at that release. A non-important drop left there is gone and does not come back on the next load of 691.
73. **Equipped-model indexes** (synthetic inventories): weapon-family item 100 gives w = 2 (key 0x222, resource 546); item 112 gives w = 112 − 103 = 9 (key 0x229, resource 553); no weapon gives w = −1 and loads nothing. Shield-family item 108 gives s = 2 (key 0x232, resource 562); item 104 in the shield-family entry gives 104 − 106 = −2, outside 0 to 4, so s = −1 and nothing loads.
74. **Quick-item refill** (synthetic inventories, Right list 145, 54, 51, 52, 53): the party owns only 145 and 52. Entry 51: step 1 gives 52 (owned, stop). Entry 53: step 1 gives -1, step 2 gives 145 (stop), so the wrap cost one step. The party owns none of the five, entry 53: the steps give -1, 145, 54, 51, 52, 53, -1, 145, and after 8 steps the entry holds 145 (unowned), not 53. The same with entry 52: the eighth step gives -1, and the entry holds -1.
75. **Item names** (disc check values; read table 0 to check a reader): id 98 gives "Wooden Sword", id 126 "Tattered Cloak", id 175 "Treasure Map". Id 137 has a row with no name pointer: no name. Id 181: no row, no name.
76. **Once-per-area set** (synthetic): world slot 70, Golden Negg (151) set all clear. The add of 151 sets bit 70 (byte 8, bit 6) and succeeds. A second add of 151 in slot 70 is refused. An add in slot 71 succeeds. A test or set with slot 193 reads 0 and changes nothing.
77. **Equipment veto by type** (synthetic): Tor's body entry (category 5, entry 0) holds id 123 and Roberta's (entry 1) holds nothing. A type-4 placement of id 121 is vetoed (121 is not above 123); a type-4 placement of id 125 spawns. A type-1 placement of any body id spawns, because entry 1 is empty. The result is the same with either hero active.
78. **Dialogue grant**: showing line 98 of pack 1017 (Guildmaster Marl) gives one item 23. Showing line 4 of pack 1150 gives one item 38.
79. **Ask-about set**: after any number of buys and pickups the hero's 41-byte set is still all zero.
80. **Luck clamp**: luck 2, an add of -5 gives 0; luck 20, an add of 10 gives 24; a save holding 30 loads as 24.
81. **Draw ranges**: Index(100) never returns 100, so a money chance of 100 always gives money and a chance of 0 never does; Range(0, 100) can return 100, so the luck gate at luck 24 (L = 100) still vetoes 1 draw in 101.
82. **Cloak on and off** (synthetic inventory): Tor (hero 0) active, body entry 123, cloak entries both 126. Square on the Items tab: Tor's body entry becomes 126 and his cloak entry 123; outfit 7 (resource 511); cue 0xA1; the wearing query gives 1; weapon and shield are hidden. Square again: body 123 (outfit 4, resource 508), cloak entry 126, query 0, held models rebuilt. With Roberta active and the same Square, her body entry becomes 126 and she shows outfit 7 (resource 519).
83. **Cloak removed by an attack** (synthetic): Tor wears the cloak and R2 starts the attack step (state 13). The removal task runs: white fade to alpha 128 over 37 ticks, one-shot 0xB1, then the swap (body entry back to the stored outfit), fade back over 8 ticks, and Tor ends in state 1 with no attack. The wearing query gives 0 afterwards.
84. **Body item added while cloaked** (synthetic): Tor active, body entry 126, cloak entry 121; an add of 123 (type 4). The swap runs first (body 121, cloak 126), then 123 replaces 121 because 123 is greater. Result: body 123, cloak entry 126, cloak not worn.
85. **Item 134 add** (synthetic): an add of 134 goes to the general list; both body entries are unchanged and no outfit changes.
86. **Negative key at region start-up** (synthetic): a flag-0 placement with key -1 and a world with 3 live items. Start-up spawns nothing, the live count stays 3 and the spawn counter is unchanged. The same placement through a scripted add spawns the item that its group roll picks.
87. **Clover gate** (synthetic): the hero stands inside a clover entry in action state 1 with the stick released (speed value 0): no roll. The stick is pushed to half length past the dead zone (speed value about 0.5): the entry is tested and rolls 0 to 100 each tick.
88. **Wallet with no object** (synthetic): with no wallet object, a give of 50 and a remove of 20 change nothing, and the query returns 0.
89. **Charm through the use dispatch** (synthetic): Tor's word 2 holds Mote 164 and word 1 holds 160. Cross on the Mote 164 cell of the item screen: timer G2 becomes 240, G3 and G4 are unchanged, the count of 164 is unchanged and cue 0xA2 plays.
90. **Map pickup** (synthetic): a world pickup of a map placement whose byte +0x2C is 4 sets the held flag of map 5, plays cue 159, plays no stinger and leaves the count of item 175 at 0. A script give of 175 with quantity 5 has the same result.
91. **Showcase stinger** (synthetic): a full showcase of a new sword with Tor active requests stinger 42 (dropped), then the pose task plays stinger 8. A full showcase of a shield (item 108) with Tor active requests stinger 42 (dropped), plays pose 0xA6, and the pose task plays stinger 6 (armour 120 with Tor and dress 130 with Roberta also play 6; a wand with Roberta plays 8).
92. **Collect distance** (synthetic): Tor's motion record at (0, 0, 0) and a ready, non-important item at (2.0, 1.7, 0): d = 2.625 < 2.675, collected. At (2.0, 1.8, 0): d = 2.691 >= 2.675, not collected.
93. **Fresh game equipment** (disc): after the new-game kit, Tor's category-5 slot 0 holds item 119 ("Farmer's Clothes", type 4, armour percent 0) with count 1; Tor's category-1 and category-2 slot 0 are empty; Roberta's slot 1 of categories 1, 2 and 5 are empty; the general list holds 150 x3 and 149 x3; the count query for 119 returns 1. Tor's outfit offset is 119 - 119 = 0 (resource 504). A later add of item 120 (type 4, greater id) replaces the entry: body 120, outfit 1, and item 119 is gone.
94. **Act 2 start equipment** (disc ids): Tor holds 119 (body) and a Wooden Sword (98, weapon) and 40 Neopoints. Act 2 starts: the stash receives all of that (including the 40 Neopoints); the party inventory then holds only 150 x3, 149 x3, 127 in Roberta's body entry (category-5 slot 1, count 1) and a wallet of 1150 Neopoints. Roberta's outfit offset is 127 - 127 = 0 (resource 512). Tor's party entries are empty. At the Act 3 start Tor's body entry becomes 119 again (his empty party entry takes the stash entry), his weapon 98 returns, and Roberta's body stays 127.
95. **Neopoints, drop roll** (disc values): group 2 (chance 40, range 10 to 40), fixed count 0. Draws r = 17 and amount 23: coins 10, 10, 1, 1, 1, all launched at once. Draw r = 55, t = 25, u = 45: no money, item 166. Fixed count 299, group -1 (the Thug in pack 888): coins 100, 100, 25, 25, 25, 10, 10, 1, 1, 1, 1.
96. **Neopoints, coin flight** (centre draws): vertical speed 16.0, horizontal speed 2.0, delay 0. First ground contact at tick 47, bounce speeds 9.97, 6.95, 5.01, 3.24, 2.66, rest by about tick 130, ready from about its 13th tick of age, collected at distance below 2.675, expired after 11.2 s of age.
97. **Neopoints, buy** (disc: pack 1136, item 52, price 125): wallet 100 gives "I need another 25$!" and no change; wallet 150 gives wallet 25 and the item stays on sale; wallet 25 gives "I need another 100$!".
98. **Neopoints, steal** (synthetic): wallet 500, chance draw 30 (below 50), amount draw 47: wallet 453, coins 25, 10, 10, 1, 1, 1, 5 (53 NP) at the hero's position; all collected: wallet 506. Chance draw 50: nothing happens. Wallet 3 and amount draw 50: taken 3, wallet 0, coins 1, 1, 5.
99. **Neopoints, kit and merge** (disc): at the Act 2 start with 40 NP held the stash wallet becomes 40 and the party wallet 1150; if Act 2 earns 90 NP the party wallet is 1240; at the Act 3 start the party wallet becomes 1240 + 40 = 1280.
100. **Neopoints, pool limit** (disc): the 25,000 NP chest of pack 1096 asks for 250 coins of 100; with 20 pool slots in use only 30 coins (3,000 NP) spawn.
101. **Neopoints, Gill the Fence** (disc): any of his five items costs 500 NP: wallet 499 refuses, wallet 500 sells and leaves 0, whatever the quoted price.

## Open questions

1. Resolved: a negative-key flag-0 placement at region start-up passes the negative key as the id; the spawn refuses ids outside 0 to 322, so nothing spawns (World items, Spawn routes; High).
2. Ids 208 to 322: the config table has no records for them, and the text table has no rows for them. Which mode-3 models relate to them is not encoded anywhere, so it cannot be known; the rewrite treats these ids as nonexistent (unknowable).
3. Resolved: the buy price at +0x8A is the placement's fixed count, copied into the world-item record at spawn; a record is a shop item exactly when its flag byte is 1 (Shop purchase rule; High). The script price setter only overwrites it. Table 0's catalogue price never feeds it.
4. (Partly) Text table 0 fields: +0x08 is the carry limit Q, +0x0C the catalogue price read by the enemy item steal, and +0x1F the important flag (Item text database; Pickup; Important-item give). Resolved: +0x1C is a progress-flag id that selects +0x2C when clear or +0x30 when set, but only when the selector-preferred +0x24/+0x28 pointer is absent. The class (+0x1E) and the distinct prose roles of +0x24 and +0x28 are names of purpose that are not encoded anywhere (unknowable); the rewrite reads the pointers by the selection rule above and never uses the class. Test vector: row 2, linked flag 116, absent preferred pointer: flag clear selects +0x2C; flag set selects +0x30.
5. Resolved for play (High): config fields. Stats A and B of weapons, shields and body items are resolved (Item config table; High). Charm stats F, G and H are resolved (Motes and elements; SPEC-0036). The third float (+0x08, 1.5) is the pickup distance base (Pickup). Type-3 stat D is the percentage coefficient in the word-1 shield-charge calculation; it is 100 on every type-3 record and leaves the base cost unchanged (Mote charge; High). Partly resolved: type-3 stat C (25 to 100) does not trigger a magic-pool stat-up when the item is added, because type-3 records are excluded from the stat-up predicates. No other consumer of type-3 stat C is known (Medium), so the clean side stores it and uses it for nothing else. The constant block (150, 1 to 7) and the first two floats (1.5 and 1.0) are identical in all 208 records, so they cannot make any two items behave differently: store them and do not interpret them. Test vector: charm record for item 161 (Lava Mote): stat F 25, stat G 100, stat H 75; item 172 (Nova Mote): stat F 0, stat G 50, stat H 50.
6. Resolved: ids 19, 20, 151, 154 and 174 are vetoed by their once-per-area lists of world ids, id 126 when it is held, and id 175 when its map is held (Pickup dispatch and the luck gate; High).
7. Resolved: the drop trigger is the set of events in When a drop roll runs. Region load never rolls.
8. Answered: heal streak window, reset and reaction ids are in Heal streak; the boundary is Open question 34.
9. Resolved: the stack limit is the carry limit Q, text row +0x08 (0 = no limit). The add caps the held count at Q, reaching Q removes every world item of that id in the level, and the spawner skips items held at Q (Inventory; High).
10. Resolved: the charm add count of 100 is the Mote's charge (Mote charge; High).
11. Resolved: luck is always 0 to 24. Its only writers are the add (sum, then clamped to 0 to 24), the set (clear to 0, then the add), the hero set-up clear to 0 and the save load (clamped to 0 to 24), so the luck gate's L runs from 0 to 100 (High).
12. Resolved: the shop pre-check is the buy hook of a level script (host row 260). With no hook it passes. With a hook it hands the purchase to the hook thread, which runs the buy again with the check skipped unless the hook marks it handled; all four registrations on disc pass handled 0 (SPEC-0050 section 3; High). It shows no message.
13. Resolved: there is no general percent rule. The program has two draw helpers on one generator (SPEC-0008): Range(a, b), inclusive of both ends (b - a + 1 values), and Index(n), 0 to n - 1. The drop path's money chance, item band and sub-table band use Index(100), 0 to 99; the luck gate, the clover roll and most boss percent rolls use Range(0, 100), 0 to 100. Each rule in this spec names its own range (High).
14. Resolved: word 0 holds the armour Mote. It scales enemy hits by (100 - H) percent while its cost and charge allow, and the body item's stat A reduces every enemy hit by that percent (Slot 0: armour Mote; High). Words 1 and 2 are the shield Mote and the weapon Mote.
15. Resolved: the quick panel and the item screen use one shared item-use dispatch (notify 1 and 0). A charm id is never consumed and only re-shows the equipment panel of each Mote word holding it. No item class is exempt on either path (Using an equipped item; Medium-High).
16. Resolved: **item field at +0x60.** The u32 at +0x60 of a slot is the held count of the item in every list, and for a Mote it is the charge. The count query returns it (so DoesHeroHaveItem, SPEC-0030 row 189, returns the count), and RemoveItemFromHero (SPEC-0030 row 191) subtracts from it or empties the slot when n is at least the count (Party inventory lists; High).
17. Resolved: the item screen's Circle on the Items tab sets or clears the active hero's quick-item selection marker (+0x155C) and writes no equipment slot; the marker is not an equipment field. The Charms tab equips Motes with the second-panel rules (Cross word 0, Square word 1, Circle word 2) (Equipment slots; SPEC-0035 section 14.2). **[High]**
18. Resolved: there is no shop screen. The buy action runs from the world "Buy" prompt for the nearest ready flag-1 world item (SPEC-0050 section 3; Medium for the choice of item).
19. Resolved (High): the 20 ids at boot-data address 0x2DCF28 are a fixed table in the executable image, read directly by the item-screen path and not loaded from save data. Each of the 20 ids has text-row category 6 in resource 533 table 0. The item-screen grid therefore lists consumables only; this does not establish categories for ids outside this grid.
20. Answered: coins, the shop and the script calls share one wallet value (Wallet identity; Medium-High).
21. Resolved: with no wallet object, remove and give do nothing; the query returns 0 (Wallet; High).
22. Answered: a give past the signed 32-bit limit wraps and is clamped to 0 (Wallet).
23. Answered: the wallet calls ignore their hero argument (Wallet; High).
24. Resolved: a world item ends when its fade takes life below 0: its slot is freed and the important-item give runs (World items, Per-tick step 3; Important-item give). The in-motion life timer with an end routine belongs to the enemy projectile pool (SPEC-0029), not to world items.
25. Resolved: the "thrown item" is the enemy projectile hitting the hero (SPEC-0029, Behavior 9). The bounce reference is the hero's forward axis under the shield block query, and the skip for id 0 is that status 0 (stun) is not applied while the hero's stun timer runs (Enemy projectile hits; High).
26. Resolved: index 12 reads the word after the candidate table, which is the first drawn index (0 to 11); the group add rejects it, so the group holds 7 entries (27.03 % of groups; SPEC-0029, Behavior 9, kind 14; High).
27. Resolved: kind 14 is the mote eater, which fills the global timed group and drains 20 charge per entry from the drawn Motes (Mote drain); kind 15 is the leach, which drains 50 magic and plays effect 4039 (SPEC-0029, Behavior 9; SPEC-0011; High).
28. Resolved: the bounce state gate belongs to the enemy projectile pool's bounce check (SPEC-0029), not to world items.
29. Answered: a stored price of 0x8000 or above reads as negative and raises NP on a buy (Shop purchase rule, step 4).
30. Resolved: a hero collects the nearest ready world item when its 3-D distance is below 1.5 (config record float +0x08) plus the character's radius, or, for an important item whose age is more than 0.4 x L, from up to 25 units (Pickup; High). There is no pull. The 5.0-unit facing test is the Buy chooser (SPEC-0050 section 3). The radius is 1.175 for both heroes (SPEC-0040).
31. **Sources of the enemy drop words** (Resolved, High): the drop placement ordinal is read from the enemy's spawn record (record data offset 0xB0). The two known routes that set a direct item to 172 (Nova Mote) belong to the class-37 Darkest Faerie: the member summon spawns row 128 slot-0 key 2864 (Minion Warrior), and the volley spawns row 128 slot-1 key 2844 (Minion Grunt) (SPEC-0029, Behaviors 7, 18 and 31; High). Each request succeeds only when the summoner's block has an unbound reserved record with the matching key, a free character object and room under the active cap; it does not require an already streamed character in the region (SPEC-0029, Behavior 31; High). The summoned character then carries item 172 for its first death tick. Other known activation routes pass -1 and use the spawn-record placement ordinal instead (SPEC-0038 Behavior 34; High). The carried Neopoints word is added to by the item branch of the steal attack, by the stolen item's catalogue price (SPEC-0029, Behavior 11b; High for the read). Six bounded examples are confirmed. In resource 888, region-0 record ordinal 1 is a Thug whose placement is key -1, group -1, fixed count 299; its first death tick yields 299 NP as eleven coins (two 100, three 25, two 10 and four 1). In resource 939, region-0 record ordinal 0 is a Crokabek whose placement 0 has key 146, group -1 and fixed count 3. Its death placement spawns item 146 directly; count 3 is ignored. The Crokabek row also has an item-steal attack (selector 1, 100 percent chance parameter). If that steal succeeds, the stolen item’s catalogue price is added to carried Neopoints and paid as coins on the Crokabek’s first death tick (SPEC-0029, Behavior 11b and SPEC-0038, Behavior 34). This does not establish a clean-route reachability claim or which item a given player inventory produces. In resource 736, region-0 record ordinal 9 is a Swamp Crokabek and points to placement 9: key -1, group 0, fixed count 3. Its first death tick has a 50% group-0 money roll; success pays a random 25 to 75 NP as coins, while failure pays the fixed 3 NP as three 1-NP coins. Both branches bypass the item roll. If it previously completed an item steal, the stolen item catalogue price is paid separately from the carried-Neopoints field. A fourth example from another family is resource 685, region-0 record ordinal 3, a Skeleton Archer. It points to placement 3, key -1/group -1/count 3, so its first death tick has no random money or item roll and pays three 1-NP coins. A fifth distinct-family example is resource 732, region-0 record ordinal 40, a Sludgy. It points to placement 40, whose key is item 145 (Bagguss), group -1 and fixed count 3. The direct-item branch therefore spawns one Bagguss on the first death tick and ignores the fixed count. Another family shows money/fixed-count priority: resource 823, region-0 record ordinal 2, is an Ixi Archer and points to placement 2 (key -1, group 2, fixed count 3). The 40% group-2 money success pays 10 to 40 NP as coins. On failure, the positive fixed count pays exactly three 1-NP coins, and because coins were spawned the item roll is skipped. Thus this particular placement never reaches group 2 item rolls despite the group having weighted item entries. This static example does not establish clean-route reachability. The enemy drop words therefore have exactly three sources: the placement ordinal in the enemy's spawn record (which selects the placement row by its own data, so every enemy's drop is read from the disc rather than listed here), the direct item 172 passed by the two Darkest Faerie summon routes, and the Neopoints carried by a successful item steal (only an enemy with an item-steal attack row can carry any). The enemy table has no drop column (SPEC-0029).
32. Resolved: **pet acquisition**: no item, shop or drop path writes a pet's type record or the owner's pet entry. A petpet joins only when the hero feeds a wild petpet in the world (SPEC-0047, Acquisition (joining); High). Do not assume a shop or item sells pets. Test vector: a food item given to a wild petpet makes it the live petpet; buying or dropping any item never does.
33. Answered: a buff applies only while its status timer is zero or below (Status test direction; High).
34. Answered: a heal joins the streak only when the gap is strictly less than 4.5 s (Heal streak; Medium-High).
35. Resolved: the age starts at 0 at spawn, grows by dt each tick once the delay has run out (and stays only for a mature item at its target scale), and the ready test is strictly "more than" 0.8 s (0.2 s for coins). In single precision a non-coin drop is first ready on its 49th tick (Pickup timing gate; World items, Per-tick step; High).
36. Resolved: the hero-state test refuses a non-coin item while the action state is 0x19, 0x21, 0x33 or 0x37, and the collect refuses a non-coin non-Mote item while the hero is busy (SPEC-0040 states 7 to 0x3C); a petpet-reserved token also blocks the collect. The old auto-equip step is the quick-item marker auto-select (Pickup; Auto-collect, steps 2, 3 and 5; High).
37. Answered: charm spin U[0.4 pi, 0.9 pi) with a random sign (World items, Spawning step 7; High).
38. Resolved: 11.2 s and 15.2 s are default lifetimes, not launch speeds: 15.2 s for charms and 11.2 s for other items, used when the caller passes a negative lifetime and the item is not placed (World items, Spawning step 5; High).
39. **Luck from stat-up flag B**: the third flag adds to luck, which is clamped from 0 to 24 (High; Stat-up items). Answered in part: gameplay reads use luck in the gated-pickup test and the bush reward tier; see Pickup dispatch and the luck gate and SPEC-0060, Foliage patches. A front-end predicate also returns true when its input number is below current luck. Resolved for that predicate: its inputs are the 25 sprite nodes (argument 0 to 24) of one luck meter on the item screen, shown on all three tabs (Items, Charms, Quest items) under the hero panel group. Sprite n shows while n is below the party luck, so luck L shows exactly L sprites; the 25th sprite (argument 24) can never show because luck is capped at 24. Both heroes share one luck value, so the meter is the same for Tor and Roberta. Test vector: luck 0 shows no sprite, luck 3 shows sprites 0, 1 and 2, luck 24 shows sprites 0 to 23. Resolved (proactive (open close-out)): the readers of luck are complete. The luck word is read through one small getter, and nothing else reads it; it is written only by its clamped add (stat-up flag B) and by the resets to 0. The getter has exactly four callers: the pickup luck gate (region start-up and scripted add), the luck meter predicate of the item screen, the save writer (which copies luck into the saved hero block) and the bush payout tier. No level module calls the getter, and no other boot routine reads the luck word. In the bush routine only the cuttable coin bush (model 262) uses luck; the other models in that routine ignore it (High).
40. Resolved: **Stat-up repeats** are limited per area. Each stat-up item keeps a list of world ids, and a second add in the same world id is refused (Stat-up items, step 6; High). Adds in different areas each apply their flags again.
41. Resolved: no path gives coin ids 140 to 144 a value, and none spawns or grants them; they add 0 if ever collected (Counts; High).
42. Resolved: the armour Mote's stat H scales the hit, and the body item's stat A is the armour percent; the body item's stat B is the armour charge stat (Slot 0: armour Mote; High).
43. Resolved: the lists of categories 1, 2, 5 and 9 have a fixed capacity of 2, so the slot query's test of slots 0 and 1 is the whole list, and no slot lies beyond the count (Party inventory lists; High).
44. Resolved: the counts are fixed capacities set at boot (general 20, category 7 60, category 8 14, categories 1, 2, 5 and 9 two each). The add rule of each list is in Party inventory lists; a remove (RemoveItemFromHero, RemoveAllOfItemFromHero; SPEC-0030 rows 191 and 194) lowers the held count or clears the slot in place and never changes a count, so the stale-slot case cannot occur. The general list's header is the count at +0x04 and the base at +0x08 (High).
45. Resolved: the 193-bit set is the storage of the five once-per-area lists (items 19, 20, 151, 154 and 174), indexed by world slot. The inventory add writes it; the placement veto and the foliage build read it (Flag set for ids below 193; High).
46. Resolved: the selector chooses a world item's material list. The three words are the item-model list, the second boot list and a loaded world's list; the ranges are item ids (World item material list).
47. **Single-global accumulator**: a global stores a count; when the previous count was positive, an amount is added to a running total, otherwise the total restarts with the amount. Resolved (proactive (open close-out), High): this is the price caption's line 1 running total (Neopoints, Display; SPEC-0035 section 15.2). The two globals are the shown number and its counter: a coin gain g with a shown counter above 0 gives shown = shown + g, otherwise shown = g, and the counter is then set to 180 (the second argument of the caption helper). It is unrelated to the per-hero, per-pool heal streak of "Heal streak", and no gameplay rule reads it; only the caption draw does.
48. Resolved: the 0x70-byte allocator is the general-list add of the party inventory: first a non-empty slot with the same id, else the first empty slot (Party inventory lists; High).
49. Resolved: the full gate is the maturity rule of World items, Per-tick step 2 (game-state code above 0, state not 2 or 5, age at least 0.8 s and at least L / 2). Only an item that is not mature fades once its age passes L. The integer life timer belonged to the enemy projectile pool (SPEC-0029), so there is one life value for world items (High).
50. Resolved: at spawn life is 1.0, the age 0, the delay D, the lifetime L by the rule of World items, Spawning step 5, and the spin and speeds by step 7 (High).
51. Answered: the flight constants are fixed program constants; coin flight values are data words (Where it lives; World items, Charms).
52. Resolved: the tests compare the snap step (hover height minus the height above the ground reference) with the window 0 to 4.8 dt and -7.8 dt to 0; the hover height is 1.78 for charms and 2.08 for other items, in world units above the reference. The 3.5 cap applies to shop items (flag byte 1) (World items, Other items and Charms; High).
53. Resolved: there is no per-kind cap. All routes share one pool of 50 slots per world, and a spawn fails only when the id is outside 0 to 322, the carry limit is reached, or all 50 slots are taken. The magnitude test is Larg < 0 (World items, Pool and record and Spawning; High).
54. Resolved: the 2-in-101 roll is the clover patch's per-tick find of item 19 or 20, gated by the two once-per-area world sets (Clover patches; Medium-High).
55. Resolved: there is one charm motion routine, the hop of World items, Charms. The kind-17 branch of the step with a bounce cooldown belongs to the enemy projectile pool (SPEC-0029), not to world items (High).
56. Resolved (High): the one-shot event spawns item id 145 (Bagguss), not a world-item kind (One-shot event; Sources; SPEC-0047 food items).
57. Resolved: the fall of 0.1 per tick belongs to a world-object manager (SPEC-0048), not to items; the charm gravity is 12.0 units per second squared and the coin gravity 40.0 (World items).
58. Resolved: the fall reset below -50 and the radius-plus-1.5 trigger belong to the world-object manager at world +0x2D40 (SPEC-0048). The item "magnet" is the range collect of Pickup: an instant collect of an important item from up to 25 units once its age is more than 0.4 x L, with no pull (High).
59. Resolved: the separate lookup is the Mote attack status code, not an item category, and the float lookups use the Mote parameter table (resource 533, type-42 table index 16) for P, D, A and T. Their mappings and use are in Item ids and categories, Mote parameter lookups, SPEC-0024 Table 16 and SPEC-0036 Mote status words.
60. Resolved: **Slot ids 0x32 to 0x3B and the second kind** are the item display models. Ids 50 to 59 are the world-item potion models (resource 515 + id); the second kind is the weapon model, whose 544 is the first model resource, not an item id, with a bound of 16. The bound of 5 is the shield model count (Equipment display indexes and item display models).
61. **Orbit ring step and sine**: Resolved (Medium-Low for the role). The 30-degree step is a single-precision immediate (0.5235988) inside the layout routine itself, which is why no data word holds it. The sine is the folded series of SPEC-0034 section 12 with its stored constants. The routine belongs to the twinkling point set of the box-record keys 0 and 12, not to the world-item pool (Ring layout and item draw).
62. Resolved: the "velocity" block is a position (w = 1), and the second write only puts w back to 1.0. The routine is a push-out on a pool that is never filled, and nothing reaches it (Dormant reach routines; High).
63. Resolved: the two are unrelated. The dot-product gate is the shield block of the enemy projectile hit (Enemy projectile hits), and the normal-based response is a dormant push-out routine (Dormant reach routines) (High).
64. Resolved (Medium-High): the shield display reads the hero's shield-family entry (category-2 list), not equipment word 2, which holds the weapon Mote (Equipment display indexes and item display models).
65. Resolved: hero combat uses the kind pick outputs for Mote use. The first scales the Mote-specific power value; the second is subtracted from the equipped Mote charge. No animation, sound or drop use is established (Motes and elements; High).
66. Resolved for ordinary inventory paths (High): type-6 weapon ids 98 to 105 are routed to Tor, and type-3 weapon ids 111 to 118 to Roberta, so both reach lookups stay within their respective tables. The unbounded raw lookup can still read outside the table if a mismatched id is introduced by malformed save data or an external write; that case is not established (Weapon reach; Edge cases).
67. Resolved: for Mote ids 160 to 173, the second kind-pick value is 1 for even ids and 2 for odd ids; the first value is 1 throughout the range. The flag selects one of two identical value sets in this disc build, so it does not change the result. Both known direct consumers use the first value for Mote-assisted combat power and the second for charge cost (Kind pick; Mote charge; High). The separate weapon-reach table callers are the swing-length query and Mote object placement (Weapon reach).
68. Resolved: the three pools (world +0x2D40, +0x29F0 and +0x3850) hold no items; they are never filled, and their reach routines are never reached (Dormant reach routines; High).
69. Answered for the marker setter (High for the writes): setting a different id requires a successful item-manager record lookup; success stores the id, sets three shared panel-control words to 240 and plays cue 0xA3, without changing inventory or equipment lists. No separate item-manager refresh or inventory mutation is present. The visible role of the three panel-control words remains unknown (Quick-item selection marker).
70. Resolved: the stat S of the charge amounts is stat B of the weapon-family entry (types 3 and 6), 100 on every weapon, so the amounts do not vary by weapon. The value is Mote charge, not damage (Mote charge; High).
71. Resolved: all three stat-up Neggs are sold in shops as flag-1 placements (Stat-up Negg and shield placements; SPEC-0050 section 10). Starry Negg also has a death source, the Giant Slorg in region 733. The new game's Golden Negg is consumed by the normal add, and the reset undoes its stat-up and its once-per-area bit, so it is never held (Sources, New game).
72. Resolved: flag 1 marks shop stock and the fixed count is the price (Shop purchase rule; High). The Negg prices are listed under Stat-up Negg and shield placements. In pack 1011 they are multiplied by 4 while story flag 298 is clear (SPEC-0050 section 5). Flags 2 to 4 on the same ids are conditional placements (Sources, Conditional placements).
73. Answered: the words at +0x1544 + 4k are the per-hero Mote words (Equipment slots; High).
74. Answered for ordinary game paths (High): a body-family or dress item cannot enter Roberta's weapon-family entry. The category-1 list accepts only weapon-family config types 3 and 6, while body-family types 1 and 4 use category 5; each list entry is assigned to its owner hero. Save loading restores both lists by calling the same normal add path, and the cloak swap changes only body and cloak entries. Therefore ids 119 to 133 cannot reach Roberta's weapon-reach read through pickups, script grants, list restoration or the cloak swap. Reads beyond table B remain possible only if state is introduced outside those known game paths (for example, malformed external save contents); that case is not established. See Edge cases and Weapon reach.
75. Resolved (Medium-High): the weapon family is the category-1 list, the shield family the category-2 list and the body family the category-5 list, each indexed by the hero index; the Mote words are separate (Equipment slots).
76. Answered: item 50 revives the partner to full health (Consumable effects by item id; SPEC-0053).
77. Resolved: the map number is placement byte +0x2C plus 1. The byte is copied into the world-item record (+0x8C) with the rest of the placement data at spawn (Treasure maps; High).
78. Resolved: **every map has a source.** Script grants give 21 maps at 26 sites in five modules; modules 991 and 1353 give maps 21, 17, 35 and 36, 40, 38. The other 23 treasures have map placements, 20 as shop stock and 3 as flag-4 placements (Treasure maps; High). Seven of those (maps 2, 4, 9, 12, 25, 32 and 33) are shop stock in packs 749 and 787, which are reached (Medium; Open question 90).
79. Resolved: a world pickup, a purchase and a script give of item 175 reach the same add, which sets map n's held flag and leaves the count unchanged (Treasure maps; High).
80. Resolved: **items 126 and 134.** Item 126 (config type 15, text category 9) fills both cloak entries and is worn only through the cloak swap (Square on the Items tab), which puts it in the active hero's body entry; it then shows outfit 7 for either hero and the wearing query sees it. Item 134 (config type 1, text category 0, no name) goes to the general list, never reaches a body entry and has no source (Outfit items; Tattered Cloak; High).
81. Resolved: the flag-1 placements of outfits 121, 123, 125, 130, 131, 132 and 133 are shop stock, sold at their fixed counts (Outfit items; SPEC-0050 section 10; High). Item 130 is the stock of pack 859, whose purchase hook reacts to it. Item 129 is stock only in pack 776, which is reached. Item 119 comes from the new game. Item 127 comes from the Act 2 start kit and item 128 from a dialogue line. Item 124 has only a conditional source.
82. Answered: the panel and the item screen always force the use (Consumable healing; Using an equipped item).
83. Answered: the luck gate applies to region start-up and scripted adds alike (Pickup dispatch and the luck gate).
84. Answered: the second panel is a Mote picker (Second quick panel and Mote equipping; Open questions 95 to 104).
85. Resolved (High for the position and the damage call, Medium-High for the geometry): item 57 places the game's single item-effect object (effect records 4020 and 568) 2.0 units ABOVE the hero, not ahead of him; both effects start when its 60-tick timer ends; the object is a hit volume (cylinder of height 4.0, radius 0 during the timer and then 0.5 growing to about 14.3 over 41 ticks) that deals 100 damage to Minion-corps characters (spawn-record keys 2801 to 2877 in the listed groups) and breaks breakables; the free test fails while the timer is 0 or above or the main effect's particle lives (Consumable effects by item id).
86. Answered: the charm-only gate is set by SetItemBlockMode (SPEC-0030); no level module calls it (Open question 108).

87. Resolved: the use-request gate is the use hook of a level script (host row 269); see Use-request gate and charm-only flag (High).
88. Answered: an entry left on an unowned candidate or -1 has count 0, so its cross icon is hidden (SPEC-0023 section 11).
89. Answered: the item screen forces the use like the panel, with notify 0 (Using an equipped item; Open question 116).
90. **Pack reachability** (Medium): of the 191 world packs, 174 are classed as reachable and 17 as unreachable (SPEC-0006, Reach; a static upper bound that ignores story gates): 678, 683, 684, 686, 720, 742, 879, 970, 1003, 1004, 1055, 1077, 1082, 1170, 1259, 1298 and 1323. Packs 742, 879, 1055 and 1323 have no normal incoming route (SPEC-0066); the remaining links are a static bound, and which are reachable under story gates needs a runtime check. The classes affect the 4 ids with sources only in unreachable packs (68, 101, 105 and 111) and the placements cited as reachable in Sources and Outfit items.
   Default if unresolved (rewrite choice): treat every pack the reach of SPEC-0006 marks as reached as a place where its placements and shop stock can appear, and give the four ids above no obtainable source.
91. **Group -1 drop placements** (static part resolved, proactive (open close-out); live trigger reach of each placement needs a runtime check): all 394 are referenced. 325 are in world packs whose placement-id table lists every placement of the pack, so that table is a complete list rather than proof of use. 69 are in character regions and are reached only through enemy drop ordinals. 131 are the target of an enemy drop ordinal (the 69 included), and 67 of those have a fixed count above 0 and give coins only. A complete placement-id and breakable-reference crosswalk finds one breakable reference to a key -1/group -1/fixed-count-0 placement: pack 816, breakable entry 11, placement ID 11, current-region ordinal 11. Its drop path has neither a money roll nor a fixed-count payout nor an item roll, so it yields no reward when triggered. The one distinct id-form ItemActivate call site in the listed level modules passes ID 12; pack 800's table maps it to slot 5, resource 806, ordinal 0, whose key is 9, so it is not one of these placements. The other listed ItemActivate calls use the world form with region -1 and return without spawning. These are static call/reference findings; they do not prove live route or trigger reachability (Sources, Drop placements).
92. Resolved: flags 2, 3 and 4 partition the conditional placements by trigger: flag 2 breakable drops, flag 3 enemy drops, flag 4 type-40 chest, dig and chain rewards (Sources, Conditional placements; SPEC-0050 section 12; High).
93. Resolved for the module and direct boot call paths: the module-1382 spawn field has one direct writer, called six times to set the first Mote id of each pair (Sources); the paired id is stored separately. The module's zero-fill acts on a three-byte local buffer, not this record field. The boot grant handler has no direct caller or registered table entry and is not an identified item source. Indirect runtime entry or external writes remain unestablished.
94. Resolved: the twelve removed items, and six more no-source ids, are granted by dialogue give-item commands (Counts, Dialogue grants; High for the commands).
95. Resolved (Medium-High): kind 1 is the weapon family and kind 2 the shield family. Square fills word 1 (the shield Mote) when a shield is held, and Circle fills word 2 (the weapon Mote) when a weapon is held; the display reads the same families (Equipment display indexes).
96. Resolved: both panels can never be open together, because each open sets its flag only when the other flag is 0 (Panel visibility and clears, item 2; High).
97. Resolved: the validity test is the inventory slot-empty test: Square needs the hero's category-2 slot and Circle the category-1 slot to be non-empty (count 1 or more) (Second quick panel and Mote equipping, items 13 and 14; High).
98. Resolved: the equip stores the id, re-arms that word's equipment-panel timer to 240, starts Tor's shield or weapon Mote glow for words 1 and 2, and plays cue 0xA3 when the show flag is 1, which the Mote panel always passes; no slot has its own cue (Equipment slots; High).
99. Resolved: the "animation" is two HUD icon tweens (new icon in at 800 units per second, old icon out at 100), not the portrait swing (Second quick panel and Mote equipping, item 19; SPEC-0023 section 11.1; High for the triggers, Medium for the motion).
100. Resolved: the Mote must be held: its inventory slot is found and not empty (count 1 or more) (Second quick panel and Mote equipping, item 16; High).
101. Resolved (High): the owned count and the name are drawn by the HUD Mote panel, group node 392 with its label at canvas (30, 404) (SPEC-0023 section 24), and on the item screen by the two name texts (Second quick panel and Mote equipping, items 5 and 5b).
102. Resolved: both panels are never open together (Open question 96), so a Triangle press closes only the panel that is open (High).
103. Resolved (High): the direction table has 15 rows (selections 255 and 0 to 13) and every byte is 0 to 13 or 255, so there is no row for selection value 14 and none is needed (Second quick panel and Mote equipping, item 6).
104. Resolved: the second caller is the pause item screen's Charms tab, and its input block is the menu block, which has the pad-record layout with each word the OR of all plugged pads. Directions come from its newly-pressed word that includes the left-stick direction bits, buttons from its newly-pressed button word. In play the selector passes the hero's own pad record, and the handler reads directions and buttons from its newly-pressed button word only, so the left stick does not step the in-play picker (Second quick panel and Mote equipping; High).
105. Resolved: the word is the registered use-item callback; RegisterHeroUseItemCallback writes it and the level's manager reset clears it (High).
106. Resolved: the default dispatch uses id 0, which has no use effect; it plays cue 0xA2 and changes nothing. It runs only when the hook leaves the handled word at 0, which pack 1350's hook does and the hooks of packs 1176 and 1371 do not (Medium-High).
107. Resolved: the gate receives the used item's id as its second argument and stores it as word 0 of the hook's request block; the callback reads it there. The default dispatch does not use it (High).
108. Resolved: no level module calls SetItemBlockMode; the boot program writes the word only through that call. The word stays 0 for the whole game, so the charm-only rule never applies on disc (High).
109. Resolved: the busy test's conditions are listed in Panel visibility and clears, item 6, and the second menu test is the pad-slot-0 usable test, so the pause page also opens with no Select when pad 0 is unplugged or not ready (SPEC-0035; SPEC-0033 auto-pause; High).
110. Resolved: code 0x17C is the overlay box host (state 23; SPEC-0035). Each in-play pass with the overlay box busy clears both panel flags and goes there; normal play reaches it whenever a script opens the box (Panel visibility and clears, item 5; High).
111. Resolved: the common clear is the HUD reset, the body of the FrontendHUDReset script call. On disc it runs at scripted camera play starts of the PlayCamera family, the element-sealed chest open, the dig, the menu openers, the quit-hold state and the one script call (the arena win path of module 1176); SetAiHotspotDistCamera and the static and look-at play starts also reset but are never reached (Panel visibility and clears, item 5; High).
112. Resolved: mode 8 is set by the script call SetHeroSceneCamera (one level module, pack 1250) and by the hero death step when no partner can take over and there is no respawn point (the death camera at distance 15.0); modes 9 and 12 have no entry that runs; mode 10 is set only by SetAI_DistCamera and SetHeroDistCamera, which no level module calls. ReturnToNormalCamera and SetToNormalCamera put modes 8 to 10 back to 0 (Panel visibility and clears; High).
113. Resolved: while the camera mode is nonzero, an Up edge that passes the in-play gate runs the open again (flag set, cue 0x9A, entries stepped, selection 1), and the panel shows when the mode returns to 0 (Panel visibility and clears, item 3; High).
114. Resolved: the third reader is front-end predicate slot 37, the Mote panel's draw gate (second flag set and camera word +0x50 = 0) (Panel visibility and clears, item 4; High).
115. Resolved: the selector calls the first panel's clear on Cross or Circle with a selection and on Triangle; the hero tick calls it whenever game-globals +0x20 or the game-state code is nonzero. The second panel's second clear has no reference and never runs (Panel visibility and clears, item 5; High).
116. Resolved: the notify flag plays the item's use effect at the hero after a successful use, and for item 50 it turns on the fade and pad cue of the swap to the revived partner (Use-request gate and charm-only flag, item 1a; High).
117. Resolved: **Region ids in the placement id table**. The region byte is a slot number j: 0xFF names the world pack's embedded region, and any other value j names the first-created loaded region slot whose most recent load recorded j. For the road encounter, the operative world is pack 691; its placement table has entries for slot 0, ordinals 0 to 2, and slot 3, ordinal 0. Module 678 loads set 0 into slot 0 (resource 692) and later set 3 into slot 3 (resource 695). Resource 692 has three kind-8 placements and resource 695 has one, matching those entries. The single embedded kind-8 placement in duplicate packs 678 and 686 does not constrain those slot-region entries (SPEC-0006, Region slots and Tagged tables; SPEC-0042, module 678; High).
118. Answered: the steal coin split and the item branch's catalogue-price value are in Stolen Neopoints split into coins and SPEC-0029 Behavior 11b (High).
119. Resolved: **Item 36 model.** Hero state 27 sets item 36 (the Rope Gun) as the hero's held entry, and that state can run at any time, so resource 575 is loaded on the first hero update. The held entry is drawn with resource 575 at the hand while the weapon is sheathed (Equipment display indexes and item display models; SPEC-0046; High for the state-27 write, Medium for its meaning).
120. Resolved: **World-item model quirk.** A world item with id 148 to 157 loads the model of potion 50 to 59, which registers under id 50 to 59, not under 148 to 157. So the loaded model is never drawn for that item. There is only one world-item slot, so the load can also evict a resident potion model (Edge cases; Medium-High).
121. Answered: the weapon and shield models are drawn attached to the hero (SPEC-0046, Held items and character attachments).
122. Resolved: the veto tests the party's category-list entry fixed by the item's config type, entry 0 (Tor) for types 4 to 6 and entry 1 (Roberta) for types 1 to 3, whatever hero is active (Pickup dispatch and the luck gate; High).
123. Resolved: every large flag-4 stash has a named type-40 trigger. The 25,000 NP and 6,000 NP stashes of pack 1096 are unlocked Square-press chests, the 1,487 NP of pack 1363 is an element-sealed chest, and the others are dig treasures (Sources, Conditional placements; SPEC-0050 section 12.1; High).
124. Resolved: the 323-bit set is written only on an ask-about-item branch that the game never takes and is never read. The Buy prompt does not set it, and it is not an ownership record (Inventory; High).
125. Answered: the new-game Golden Negg is consumed as a stat-up item and its effects are undone (Sources, New game; Medium-High).
126. Answered: the hero radius is 1.175 for both heroes, so the normal collect distance is 2.675 (Pickup; SPEC-0040 body size).
127. Resolved: the patch gate value is the hero's stick speed value; above 0 means the player is pushing the stick (Clover patches; High).
128. Resolved: no record on disc can fire the hook. All 27 registrations name ids 57 to 174 against a lookup that accepts only ids 0 to 9, and no module registers an item-proximity callback; prompt 9 is the Square caption label 9 (Pickup, step 2; High).
129. Resolved: none of them belongs to items. The "thrown item step" and the cue bag are the enemy projectile pool's hit and the Faerie Sisters' mote eater (SPEC-0029), and the 0x160 and 0x140 pools are never-filled world pools whose reach routines are never reached (Enemy projectile hits; Mote drain; Dormant reach routines; High).
130. Resolved for the mechanism (High): the six ids come only from flag-3 enemy drops, and every pickup adds 2 (capped at Q), for example 0, 2, 4, 5 for a Black Feather (Important-item give). The visible result still needs a black-box check (Medium).
131. Resolved (Medium): modes 1 to 3 anchor the model to the screen at E + z·F − 640·(x − 0.5)·(z / S)·R − 480·(y − 0.5)·(z / S)·U, turned by the showcase angle, given the camera's orientation and scaled by s (Item showcase). The sign of each screen axis needs a black-box look.
132. Resolved: the showcase stinger request goes to the same stinger player and is dropped (Showcase; High).
133. Resolved (High): **Starting equipment.** The new game puts Farmer's Clothes (119) in Tor's body entry as a real item with count 1; weapon, shield, cloak and Motes start empty for both heroes, and Roberta's entries start empty until the Act 2 start kit gives her Diplomat Dress (127) as a real item in her body entry. The rags are listed on no item-screen tab; they appear only as the body icon of the equipped-item strip (SPEC-0035 section 14.1; Starting equipment).
134. Resolved (High, Neopoints chapter): the wallet is one signed 32-bit word on the party container with no cap; the coin values 1, 5, 10, 25 and 100 are fixed in the add; ids 140 to 144 add nothing; remove and give floor at 0 while the coin add and the buy do not; a death takes no NP; the steal, the buy refusals and the act-start values are as stated there.
135. Resolved (High): coin splits are limited by the 50-slot item pool, so a 25,000 NP chest pays at most 5,000 NP (Sources, 50-slot limit). Medium: the number of slots in use when each chest opens.
136. Partly resolved (proactive (open close-out)). Resolved (High): Sowter's weeding pay-outs (50, 100, 200, 400) cannot be reached. They are given only by Sowter's talk (entity 90 of pack 1023), whose character region (1048) is never loaded, so the entity never exists in play (SPEC-0045, Meridell village); the rewrite omits all four lines. The 1172 and 1176 entries of the pay-out table are not known to have any other live source (Medium). Still open (needs a runtime check) (Medium): which of the other dialogue pay-out lines with no named call are shown in play (their order is story-gated). Default if unresolved (rewrite choice): show every pay-out line of a reached pack that is tied to a quest or arena result; omit the arena-announcer lines of packs 1176, 1350 and 1371.

# SPEC-0050: Shops and economy

- Answers: proactive (open question sweep, world and talk); proactive (open close-out), none requested (money, shop purchases and buy hooks); R-0225 (the shop copies beside 1427); R-0030 (points 1 to 3; Open question 29 resolved); proactive (open question sweep, script details); proactive (open question sweep, item pickup recheck); proactive (spec consistency pass); proactive (disassembly confirmation)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001). Check the coin denomination and coin id words listed under "Where it lives on the disc" before trusting the coin values.
- Confidence: per section. Checked against the disc: the coin denominations and coin ids, the wallet rules, the buy path order and messages, the buy-hook registrations, the price multiplier (call sites, gate, cap and the field it scales), the script spend amounts, the boot grant codes and the price caption. Coin ids, denominations and blank rows: High. Wallet rules: High. Buy path order and refusal messages: High; the queueing when a request slot is busy: High; the free-item path when the add is busy: Medium. Negative prices: Medium-High. Buy hooks: High for the registration gates and item tests; Medium for the meaning of the level-line ids. Price multiplier: High for the call sites, the gate, the cap and the field it scales; Medium for its effect on items without a set price and for repeat calls. Spend amounts: High for the packs listed. Grant codes: Medium (no known caller). Price caption: High for which line shows what. No selling: High, as a negative result from the code and the text. Shop stock and prices (flag-1 placements, price = placement fixed count, take or copy, carry limit, once-per-area items, treasure map numbers): High. Buy candidate choice: Medium. Multiplier timing and its flag setter: High for the setter, Medium-High for the order after spawn. Act and hero grouping of the shops: Medium-Low, except Market Town (act 3, reached before the Black Knight): High. New-game wallet and items: High. Wheel game: rules in SPEC-0059 (High). Conditional placement flags (section 12) and the large-stash triggers (section 12.1): High.
- Settled (open close-out): a multiplier call is made once per build of the interior and never compounds (Open question 6). Grant code 12 is never reached in play, so the method it calls is moot (Open question 10). The act in which each non-Market-Town shop is first entered is a grouping label inferred from story stage ids; play does not use it (shop stock and prices are fixed per pack), so it is a presentation aid only (Open question 27).

## Summary

Neopoints (NP) are one shared party wallet: one signed 32-bit value. It starts at 0 on a new game. Four kinds of change reach it. Coins picked up in the world add their value. A shop purchase from a world prompt takes the item's price. Level scripts remove fixed amounts behind yes/no prompts. Boot-level event codes add fixed amounts and grant items, but no caller is known. No path sells items for Neopoints. Shop stock and prices are disc data: a shop item is a kind-8 item placement whose flag byte is 1, and its price is the placement's fixed count. Equipment, stat-up items and treasure maps are removed when bought; other items stay and can be bought again. Four level-script hooks react when the hero buys one of four specific items. A price-multiplier call in five interior level scripts multiplies the buy price of live spawned items that carry flag byte 1 by 4, capped at 32767, until story flag 298 is set. Power growth has no experience points: it comes from stat-up Neggs and clovers and from equipment ladders (section 12). The clean side can build the wallet, the coin pickups, the shops, the buy path, the buy hooks, the price multiplier and the new-game start from this spec. The wheel game and the betting race are specified in SPEC-0059.

Related specs: SPEC-0021 (items, coins, pickups and the buyable object's layout), SPEC-0022 (the "Buy" world prompt), SPEC-0030 (host rows 192, 193, 195, 197 and 260), SPEC-0035 (the price caption) and SPEC-0059 (the wheel game, the betting race and the other side activities).

## Where it lives on the disc

- Coin denominations: five 32-bit little-endian words in the boot program's data at virtual address 0x30F6F0, in order 100, 25, 10, 5, 1 (from disc).
- Coin item ids: five 32-bit words at virtual address 0x30F708, in order 139, 138, 137, 136, 135 (from disc). Word i of the ids is the coin for word i of the denominations.
- Coin and Tal rows: rows 135 to 144 of the item text table (SPEC-0021, Item text database). Their name and price fields are catalogue data. This spec does not use them to set coin values.
- Shop stock and prices: kind-8 item placements with flag byte 1 in the world packs of section 10 (placement layout in SPEC-0006 and SPEC-0021). Each placement gives the item id (its key) and the price (its fixed count).
- Carry limit and catalogue price: item text table rows, fields +0x08 and +0x0C (SPEC-0021, Item text database).
- Wheel game pay-outs: data words of the level-script modules of packs 1023, 758 and 1358 (SPEC-0059).
- Wallet and buyable object: run-time memory. They have no disc location.
- Buy-hook registrations and price-multiplier calls: inside the level-script modules of the packs named in sections 4 and 5 (SPEC-0030, SPEC-0043).
- Script spend amounts: inside the level-script modules named in section 6.
- Boot grant codes: in the boot program's event handling. They have no data location (section 7).

## Layout

### Wallet

| Field | Size | Type | Meaning |
| --- | --- | --- | --- |
| NP value | 4 bytes | signed 32-bit | Shared Neopoints for the whole party. Both heroes read and write the same value. |

### Coin ids

| Item id | Value on pickup (NP) | Notes |
| --- | --- | --- |
| 135 | 1 per coin | Adds the coin's count, one per coin. |
| 136 | 5 | |
| 137 | 10 | Catalogue name blank and catalogue price 0. The value comes from the denomination word, not the catalogue. |
| 138 | 25 | |
| 139 | 100 | |
| 140 | 0 | Tal row. Catalogue price 1. Adds nothing. |
| 141 | 0 | Tal row. Catalogue price 5. Adds nothing. |
| 142 | 0 | Tal row. Catalogue name blank and catalogue price 0. Adds nothing. |
| 143 | 0 | Tal row. Catalogue price 25. Adds nothing. |
| 144 | 0 | Tal row. Catalogue price 100. Adds nothing. |

### Buyable object and buy-info record (run-time)

| Location | Size | Type | Meaning |
| --- | --- | --- | --- |
| buyable object +0x8A | 2 bytes | u16, read as signed | Buy price in NP. The buyable object is a spawned world-item record (next table), and this field is that record's copy of the placement's fixed count. A script price-setter call can overwrite it. The buy path reads it as signed, so a stored value of 0x8000 or more is negative. 0x7FFF is the never-affordable value. The price multiplier scales this field (section 5). |
| hero object +0x970 | 4 bytes | pointer | The current buyable object. Null when there is none. |
| buy-info record, first word | 4 bytes | u32 | Item id of the item bought. Other fields are not described here. |

### Spawned world-item record (fields used by shops)

Confidence: High. When a region spawns an item placement (at region load or by a scripted add), the placement's 48 data bytes (SPEC-0006 kind 8, data offsets 0x00 to 0x2F) are copied unchanged into the world-item record at +0x60 to +0x8F.

| Location | Size | Type | Meaning |
| --- | --- | --- | --- |
| record +0x60 | 48 bytes | copy | The placement's data block, unchanged. |
| record +0x80 | 1 byte | u8 | Flag byte (placement data +0x20). Value 1 marks a shop item: a buy candidate and a price-multiplier target. |
| record +0x84 | 2 bytes | s16 | Item key (placement data +0x24): the item id of a direct placement. Every flag-1 placement on disc has a key of 0 or more. |
| record +0x8A | 2 bytes | s16 | Fixed count (placement data +0x2A). For a flag-1 record this is the buy price in NP. The multiplier scales it (section 5). |
| record +0x8C | 1 byte | u8 | Placement data byte +0x2C (the first byte of the field SPEC-0006 lists as fill). For a treasure map (item 175) the map number is this byte plus 1. |

## Behavior

### 1. Wallet

Confidence: High.

- Add a: when a is above 0, the value becomes value + a, computed as a signed 32-bit sum. If the result is negative, the value becomes 0. A sum that passes the 32-bit limit wraps negative and so also becomes 0. When a is 0 or below, nothing changes.
- Remove a: when a is above 0, the value becomes the larger of 0 and value - a. The wallet does not need to hold a. When a is 0 or below, nothing changes.
- Query: returns the value, or 0 when no wallet exists.
- Add and remove with no wallet: nothing changes.
- The same value is read and written whichever hero a script names.

### 2. Coin pickups

Confidence: High for the values; Medium-High for the pickup path (SPEC-0021).

- A coin in the world is not kept as an item. Picking it up adds its value to the wallet (table in Layout), and it plays the coin cue of SPEC-0021 (cue 0xA5). It also shows the gain on price caption line 1 (section 9).
- The values 100, 25, 10, 5 and 1 are the denominations of the coins. A money amount is split into coins in that order (SPEC-0021, drop and money flow). A coin of 100 has id 139, a coin of 1 has id 135.
- Ids 140 to 144 are coin-class rows that add nothing when picked up. Nothing in the code spawns or adds them (Open questions).

### 3. Shop purchase from a world prompt

Confidence: High for the order and the messages. Medium for the free-item path when the add is busy.

The world prompt "Buy" with the item name is shown for the hero's nearest buyable object (SPEC-0022, Interaction prompts). Confirming it runs the buy action.

Choosing the buyable object (Medium). Each tick the prompt looks at the ready world items (SPEC-0021, Pickup timing gate) and keeps those with a vertical gap under 5.0 units, a horizontal distance under 4.0 units and a facing cosine above 0.8 toward the item. Score = distance minus 2 times the cosine; the lowest score wins. If the winner's flag byte is not 1, there is no buyable object. Only a flag-1 item can become the buyable object.

Steps of the buy action, in order:

1. No buyable object: stop. No message is shown. A diagnostic log entry is written only (text not recorded).
2. Buy hook (SPEC-0021, shop pre-check; High). If no level script in the current level has registered a buy hook (section 4), go on. If one is registered, the purchase is handed to it: the item id is stored in the hook's request block, the block's handled word is set to the registration's handled value, a hook thread is started if none is running, and this call stops. No price is taken here. When the hook returns and the handled word is 0, the thread runs the buy action again from step 1 with this step skipped. All four registrations on disc pass handled 0, so in play the purchase completes right after the hook's line is started. A second press while the thread still runs overwrites the stored id and sets the handled word back to the registered value; it starts no second thread, takes no price and shows no message, and the later re-run buys whatever shop item is targeted at that moment. No message is shown at this step.
3. Read the price as a signed 16-bit value. If it is 0x7FFF: show the never-affordable message, and stop. This test comes before the balance test, so the value applies even when the balance is large.
4. If the price is above the balance: show the need-more message, and stop. No add is attempted and the balance does not change. The amount in the message is the price minus the balance.
5. Otherwise (the price is no more than the balance), add the item to the hero's inventory by the same add routine that world pickups use, in shop mode.
   - The add is told whether to take the world item or copy it (section 8). It takes it when the item is equipment (config types 1 to 6), a stat-up item (stat A, B or C at least 1 under the SPEC-0021 stat-up type rule) or a treasure map (175). Otherwise it copies it.
   - If the add succeeds: the balance falls by the price. No message is shown. The stock rules of section 8 then apply.
   - If the add fails: show the "can't buy that now" message. The balance does not change.

Add gate. The add routine has its own hook check: the pickup hook of a level script (host row 265). When one is registered the add is handed to it and fails here, and the hook thread later runs the add again with the check skipped, with no price check and no charge. Five packs register a pickup hook, each in its init: 816, 893, 906, 1096 and 1238 (SPEC-0021, Auto-collect, step 4; High). The list is settled by the modules' import tables and call sites. None of the packs named has a flag-1 placement, so this free-item path cannot occur with the disc's data (High). An implementation may still keep the rule.

The three refusal messages are the never-affordable message (step 3), the need-more message (step 4) and the can't-buy-now message (step 5). Exact strings (High): "I could never afford that!"; "I need another " followed by the shortfall, the NeoPoints glyph and "!"; and "I can't buy that now!". The prompt is "Buy %s". All three are shown in the shared box with no wait, in the hero's name and colour; the hero goes to state 25, and stinger 9 plays on the same pass in all three. The need-more message is the only one with a number. The buy routine's text references are not table lines, so the messages are fixed strings in that routine. The messages are spoken by the active hero: "Tor" for hero flag 0, and "Roberta" otherwise. The need-more message is the text "I need another " followed by the amount, then character code 36 and "!" (see Open question 29).

### 4. Buy hooks

Confidence: High for the registration gates and the item tests; Medium for the meaning of the level-line ids.

A level script can register a callback for one item (host row 260, SPEC-0030). The callback receives the buy-info record. Four registrations exist on disc:

| Pack | Item id | When registered | Flag that must be clear | Level line shown |
| --- | --- | --- | --- | --- |
| 856 | 113 | always, in the pack's init | 220 | 19 |
| 859 | 130 | always, in the pack's init | 220 | 34 |
| 1140 | 116 | always, in the pack's init (flag 10 only picks a character set) | 220 | 0 |
| 1433 | 109 | always, in the pack's init | 248 | 20 |

Behavior of each callback: it reads the item id from the buy-info record. If the id equals its item and the test flag is clear, it shows its level line. Otherwise it does nothing. All four hooks also write 1 into the record's flag word in that case, which cancels the buy (SPEC-0030 row 260): item 113 cannot be bought in 856, item 130 in 859 and item 116 in 1140 while flag 220 is clear, and item 109 (Golden Shield) cannot be bought in 1433 while flag 248 is clear (High). The callback body sets no flag, and every purchase made in the pack passes through the hook, so the line shows again on each buy of that item while the test flag stays clear (Medium-High). Flag 220 is set by the Act 2 start and flag 248 by the Act 3 start (SPEC-0049, Act-start setup; High), so the hooks stop cancelling from those points on.

Each hook item is stock of its own pack: item 113 is a flag-1 placement in pack 856, 130 in pack 859, 116 in pack 1140 and 109 in pack 1433 (High).

The hook of pack 1433 takes effect in play. 1433 is the Vorsh shop, entered from 1300 as an interior, so its Init runs. Packs 1429, 1430 and 1431 hold the same shop and the same module, but they are only resident in the exterior buffer beside 1427, their Init does not run there, and the hero cannot stand in them (SPEC-0006, Edge cases, Interior-kind packs in E). They are not shops in play (High for the module rule and the placement of the data; Medium for never in play).

### 5. Price multiplier

Confidence: High for the call sites, the gate, the field it scales, the cap and the setter of flag 298. Medium-High for the order after the region items spawn. Medium for repeat calls.

Five level-script modules (packs 1005, 1008, 1011, 1014 and 1020) each make one call to the price-multiplier host function (host row 197, SPEC-0030), with factor 4.0.

- **Gate.** The level thread first loads the shopkeeper's character set (set 1 when flag 86 is set, else set 0); both sides of that branch reach the call. The call is made only when flag 298 is clear.
- **Order (Medium-High).** World load spawns the region's items before the level thread runs, so the stock on display is already in the list and is scaled.
- **When flag 298 is set.** The Black Knight arena script (pack 989) sets flags 298, 86 and 51 together after the fight's end movie. No shopkeeper line or other level script sets flag 86 (the ask-about lines only read it; pack 989 is the only module with a fixed-number set of 86, and the sites that take the flag number from a table carry other numbers; High), and flag 86 does not affect the price. The stage table also sets flags 298 and 86 for stage ids 2 and 3, but no dialogue line applies a stage and a plain new game applies none; only the start-point chooser applies them (Act 3 Start, Act 3 Middle and the later act starts; SPEC-0035). So these five interiors charge 4 times the placement price before the Black Knight is beaten and the plain price afterwards.
- **Reach before the Black Knight (High).** In a playthrough from a new game the x4 prices are seen. The Crossroads script (pack 880) opens its road into Market Town (pack 991) once flag 296 is set, which it sets when its three Dark Knights are dead. Market Town's script registers its eight doors on every visit with no story test: doors 0 to 3 lead into packs 1005, 1008, 1011 and 1014, doors 4 and 5 into the Trader's Guild (1017), door 6 into 1020 and door 7 back to the Crossroads (880). The Trader's Guild sets flag 297 (the start of the Black Knight quest), and the fight in the Black Keep (986 into 989) sets flag 298 later in the story. No other script sets flag 298. Market Town is first entered in act 3 with both heroes. A game started from the start-point chooser at Act 3 Start or later never sees the x4 prices, because flag 298 is already set.
- **Prices before and after (High; placement fixed counts).**

| Pack | Item (id) | Before the Black Knight (NP) | After (NP) |
| --- | --- | --- | --- |
| 1005 | Golden Shield (109) | 8,000 | 2,000 |
| 1005 | Royal Armour (123) | 18,000 | 4,500 |
| 1005 | Noble Dress (131) | 26,000 | 6,500 |
| 1008 | Battle Blade (102) | 16,000 | 4,000 |
| 1008 | Magician's Wand (115) | 16,000 | 4,000 |
| 1011 | Starry Negg (154) | 4,000 | 1,000 |
| 1011 | Red Negg (174) | 5,000 | 1,250 |
| 1011 | Treasure map 26 (175) | 1,400 | 350 |
| 1011 | Treasure map 5 (175) | 1,300 | 325 |
| 1011 | Treasure map 31 (175) | 1,700 | 425 |
| 1020 | Unguent of Curing (54) | 1,400 | 350 |
| 1020 | Meerca Speed Potion (56) | 2,000 | 500 |
| 1020 | Starlight Potion (57) | 1,400 | 350 |
| 1020 | Kauvara's Potion (58) | 3,200 | 800 |
| 1020 | Fyora's Potion (59) | 2,400 | 600 |

| 1014 | Ergyfruit (147) | 500 | 125 |
| 1014 | Striped Negg (155) | 1,100 | 275 |
| 1014 | Peachpa (food) | 700 | 175 |
- **Not debug-gated.** The call is not behind the debug-mode query. It runs in play when flag 298 is clear, in the level thread that each module starts from its init. For packs 1008, 1011 and 1020, the thread is started in play (SPEC-0043); pack 1014's init starts its thread the same way. The debug query on the path is inside a helper that returns at once in play, so it does not block the call.
- **Effect.** For each active record in the level's list of spawned items whose flag byte (+0x80) is 1, the buy price (s16 at +0x8A) becomes the smaller of 32767 and the value times 4.0, truncated to a whole number. A negative price is multiplied and is not clamped at the bottom. The call is one pass over the list at the time of the call; records added to the list later are not changed by it. If the list does not exist, the call does nothing.
- **Records with other flag bytes.** Records whose flag byte is not 1 keep their price. Flag 1 is the only value that marks a record as a multiplier target and as a buy candidate.
- **What the field is.** The scaled field is the record's copy of the placement's fixed count, which is the buy price (Layout). Host row 197 of SPEC-0030 ("scales item prices in shops") matches this. The placement data on disc is not changed; the next spawn of the same placement copies the unscaled count again.

### 6. Script spending

Confidence: High for the amounts and sites listed. Medium for the branch meanings.

Level scripts remove fixed amounts with the RemoveNeopoints host call (section 1 rule) behind a prompt. A yes answer removes the amount; a no answer changes nothing. Examples of amounts by pack:

- Pack 722: 50 (ten sites), behind a four-option prompt (the betting race, SPEC-0059).
- Pack 758: 25 (two sites; the wheel game fee, SPEC-0059).
- Pack 1358: 100 (two sites), behind a yes/no prompt (the wheel game fee, SPEC-0059).
- Pack 880: 250 (three sites), 300 (three sites) and 400 (two sites). No 500 is used. The amount is chosen by branch, which depends on which maps the hero holds.
- Pack 1230: 250 (three sites), 300 (two sites) and 500 (one site), by branch in the same way.
- Pack 1023: 15 (two sites; the wheel game fee, SPEC-0059), 200 (five sites), 300 (one site) or 1000 (one site), by branch in a cloak branch.
- Pack 1139: 500 (five sites), behind a five-option prompt. Quoted prices 200/650/350/450/550, but 500 is tested and removed for each. It also has one call with amount 0.
- Pack 787: 500, when flag 447 is set.
- Pack 1148: 100, when flag 140 is clear and flag 139 is set.

Removal floors at 0. A removal larger than the balance leaves 0, not a debt. One prompt boundary is established: pack 722's 50-NP betting-race prompt sends a balance below 50 to line 75; exactly 50 reaches the paid branch and removes 50 (SPEC-0059, section 3.4; High). Every other script spending site that tests the balance uses the same strict rule: the purchase is refused when the balance is below the price, and a balance exactly equal to the price goes through and leaves 0 (High). The sites are packs 758 (25), 787 (500), 880, 991 (300, 350 and 400), 1023 (15, 1000 and the branch prices), 1139, 1148 (100), 1230, 1353 (200, 200 and 300) and 1358 (100); in packs 880, 1139 and 1230 the price is the branch's own amount. The wheel fee in packs 758, 1023 and 1358 is tested once, before the start prompt, and the removal after the yes answer repeats no test (SPEC-0059). Test vectors: pack 1148 with 99 Neopoints refuses, with 100 sells and leaves 0; pack 787 with flag 447 set refuses at 499 and sells at 500, setting flag 448.

### 7. Boot grant codes (caller not found)

Confidence: Medium. The amounts and item grants were taken from the boot program's event handler. No caller was found.

The boot program's event handler, which is on the wallet object, changes the wallet and grants items for these event codes:

- Codes 1 to 8: each adds 1,000 NP, by the add rule of section 1, and grants items. The item grants include ids 160 to 173 at a count of 10,000. The per-code item lists are not tabulated here.
- Code 9: adds 1,000 NP, by the add rule of section 1.
- Code 10: grants 100 of item 56.
- Code 11: grants item 175. No Neopoints are added.
- Code 12: adds 10,000 NP and grants item 98. It also calls one method on a sub-object; that purpose is not encoded anywhere readable and the whole handler is unreachable in play (Open question 10), so it needs no rewrite behaviour. No sound cue is played in the handler.

No caller was found for any of these codes, in the boot program or in the level modules. Do not assume any code is reachable in play. Implement these only once a caller is established.

### 8. No selling; stock

Confidence: High unless marked.

- No sell routine, sell text or sell host call exists. No routine converts an owned item into Neopoints, and there is no buy-back. This is a negative result (High).
- **Stock is placements.** There is no separate shop table. A shop's stock is the set of kind-8 item placements with flag byte 1 in its world pack. At region load each one spawns as a world item (unless a spawn veto below applies), its 48 data bytes are copied into the world-item record, and its fixed count becomes its price (Layout). On disc every flag-1 placement has a key of 0 or more, a price from 20 to 10,000 and no luck gate.
- **Take or copy.** A purchase of equipment (config types 1 to 6), a stat-up item (Golden, Red and Starry Neggs and the two clovers) or a treasure map (175) takes the world item: it is removed from the level for the rest of the visit. Every other purchase (potions, food, Silver, Speckled and Striped Neggs, Motes) adds a copy and leaves the world item in place, so it can be bought again and again.
- **Carry limit Q.** Q is field +0x08 of the item's text-table row (SPEC-0021). Q = 0 means no limit. (a) An add never raises the held count above Q. (b) After a successful add (a purchase or a pickup), if the party holds at least Q of that item, every world item of that id in the current level is removed (sold out). (c) The world-item spawner does not spawn an item whose held count is already at least Q (Q above 0). On disc Q is 1 for every equipment item (98 to 133 and 176 to 180) and most quest items, 5 for items 70 and 78, 4 for 79 and 82, 3 for 80, and 0 for every consumable, Negg, Mote, coin and map. So equipment that is owned never shows in a shop again.
- **Equipment that is not an upgrade (High).** Region load does not spawn an equipment item whose id is not greater than the id already held in its family entry (SPEC-0021, Pickup dispatch). The entry is fixed by the item's config type, not by the active hero: types 4 to 6 test Tor's entry and types 1 to 3 Roberta's. With Q = 1, shop equipment shows only upgrades not yet owned.
- **Once per area.** Items 19 (Four-Leaf Clover), 20 (Twelve-Leaf Clover), 151 (Golden Negg), 154 (Starry Negg) and 174 (Red Negg) each keep a list of world ids. An add of one of them is refused when the current world id is already in its list; otherwise the world id is added to the list. Region load does not spawn them for a world id already in the list. A new game empties the five lists. So each of these is gained at most once per area, whether found or bought.
- **Treasure maps.** A map bought from a shop gives map number (placement byte +0x2C) plus 1. A map placement is not spawned when that map is already held.
- **Map sources (High for the data).** Every one of the 44 real treasures (SPEC-0051) has at least one map source on disc, of three kinds:
  - Script sellers: the level modules of five packs sell maps behind yes/no prompts (section 6), 26 grant sites in all, covering 21 distinct maps. Pack 880 sells maps 1, 8, 30, 27, 18, 3, 6 and 11. Pack 1230 sells maps 1, 8, 30, 27, 18 and 10. Pack 1023 sells maps 13, 14, 24, 28 and 46 at 200 NP and map 16 at 300 NP. Pack 991 sells map 21 at 300 NP, map 17 at 350 NP and map 35 at 400 NP. Pack 1353 sells maps 36 and 40 at 200 NP and map 38 at 300 NP.
  - Shop placements: 21 flag-1 map placements (21 maps), priced from 120 to 425 NP.
  - Chest rewards: 3 flag-4 map placements are the rewards of chest groups (SPEC-0048, Lock chests): map 20 in a kind-1 chest in pack 1122, and maps 23 and 34 in kind-2 chests in packs 947 (Root Cellar) and 1247 (Crypt of the Fallen).
- **Maps only in the door-reached shops (Medium).** Maps 2, 4, 9, 12, 25, 32 and 33 are sold only in the shop packs that section 10 lists as the group reached through the doors of packs 722 and 758 and through pack 1427. Those packs are in the new-game reach of SPEC-0006, so the seven treasures can be dug in normal play once the player has found those doors.
- **Catalogue price.** Field +0x0C of the item's text-table row is not a shop price. It is read in one place only: when an enemy's steal attack takes an item, the enemy's carried Neopoints grow by that item's catalogue price (the per-item value of SPEC-0029, Behavior 11b), and they are paid out as coins when it dies (SPEC-0021).

### 9. Shop prompt and caption

Confidence: High for which line shows what, the colours and the coin-gain total. The rest of the section follows the buy path (section 3).

- The shop is the world prompt "Buy" with the item name, chosen by the interaction chooser (SPEC-0022). It is not a front-end screen, and the chooser does not open a shop screen.
- The prompt shows "Buy" with the item name. On each tick the prompt applies, the balance is written to caption line 2 and then the price to line 1, each with a shown counter of 1, so the caption shows on exactly the ticks the prompt is up. Confirming runs the buy action (section 3).
- The full caption layout is in SPEC-0035 section 15.2: font 0, both lines right-aligned with their right edge at x 410, line 1 at y 115 and line 2 at y 140. A line is drawn while its shown counter is above 0, and each draw lowers the counter by 1.
- Line 1 (the upper line) shows the price as a number. When the price is no more than the balance, it is drawn in the default caption colour (160, 160, 90, 127). When the price is above the balance, line 1 shows the price in colour (128, 60, 90, 127). There is no cannot-pay label. When the price is 0x7FFF, line 1 shows an "Exorbitant" label in (128, 60, 90, 127).
- Line 2 (the lower line) is the number only: the Neopoint balance, in (128, 128, 128, 127), with no label.
- Coin gains: every coin add (items 135 to 139) sets caption line 1 to the gain with counter 180. While line 1 is still shown, gains add up (new value = shown value + gain), so coin pickups show one running total for 3 s (180 drawn passes) after the last coin. Pickups play cue 0xA5.

### 10. Shops on disc

Confidence: High for the counts and the rule. Medium-Low for the act and hero of each group: the act is inferred from the stage ids of the story flags each module tests, from act-tagged movies in nearby packs and from the buy-hook flags. The hero follows the act-start swap mode (SPEC-0053): Tor only in act 1, Roberta only in act 2, both heroes from act 3.

The disc holds 201 flag-1 item placements in 43 world packs. All 43 packs are in the new-game reach of SPEC-0006 (174 of the 191 game packs), which counts packs that can load, not places the hero can stand. 32 of them are grouped by act below. Eight are reached through the doors of packs 722 and 758 (packs 745, 749, 750, 776, 783, 786, 787 and 789); their act is not inferred. Packs 1429, 1430 and 1431 hold the same stock as 1433 but are only resident beside 1427: the hero cannot stand in them, so nothing can be bought from them. Each pack's stock is read from its flag-1 placements by the rule of section 8; no per-shop list is needed.

| Group | Hero | Packs | Placements | Price range (NP) | Sum of prices, one of each record (NP) |
| --- | --- | --- | --- | --- | --- |
| Act 1: Meridell village interiors, pack 984 and the pack 842 area interiors | Tor | 12 | 46 | 20 to 7,000 | about 30,400 |
| Act 2: Brightvale hub and its luxury interiors (two packs with the same stock) | Roberta | 3 | 22 | 20 to 8,000 | about 59,000 (about 30,000 counting one of the two twin interiors) |
| Act 3: Meridell district interiors (five with the x4 multiplier), the treasure-map exchanges and the Werelupe area | both | 8 | 40 | 50 to 10,000 | about 45,300 (about 127,700 with x4 before the Black Knight) |
| Act 4: the city districts | both | 10 | 46 | 20 to 6,000 | about 34,200 |
| Door-reached (act not inferred) | not inferred | 10 | 47 | 20 to 8,000 | |

Pack numbers by group: act 1: 855 to 859, 984, 1131, 1133, 1136, 1140, 1143 and 1172; act 2: 1300 and 1433; act 3: 880, 1005, 1008, 1011, 1014, 1020, 1230 and 1251; act 4: 1350, 1353, 1405, 1407, 1409, 1417, 1419, 1421, 1423 and 1425; door-reached, act not inferred: 745, 749, 750, 776, 783, 786, 787 and 789; never visited: 1429, 1430 and 1431.

What the groups sell (summary): every group sells potions and food, and the act 1, act 3 and act 4 groups also sell Motes. Equipment rises in step with the acts (SPEC-0021, Equipment families): act 1 shops sell the first sword, shield, armour, wand and dress upgrades; act 2 sells the Golden Shield, the Sorcerer's Wand and the Faerie Dress; act 3 sells the middle ladder steps (some at x4); act 4 sells the top swords, wands, armour, dresses and shield. Golden Negg is sold in act 1, act 2 and act 4 shops; Starry Negg in act 1 and act 3 shops; Red Negg in one act 3 shop. 21 placements sell treasure maps (section 8).

### 11. Income, new game and the wheel game

Confidence: High unless marked.

- **New game.** The game reset clears the party inventory, sets the wallet to 0, and adds Farmer's Clothes (119) x1, Red Juppie (150) x3, Purple Juppie (149) x3 and Golden Negg (151) x1 through the inventory's normal add, the same add a pickup uses. That add consumes the Golden Negg as a stat-up item, so it is never held. Later steps of the same reset empty the five once-per-area lists (section 8) and set the pools to their start values, which undoes its stat-up and its list bit (Medium-High; SPEC-0021, Sources, New game).
- **All income is coins.** Coins (ids 135 to 139) add their value on pickup (section 2). They come from drop placements (breakables, enemy deaths, scripted adds), fixed-count placements, the carried Neopoints of an enemy that stole an item (its catalogue price), the Neopoint steal's coin spawn at the hero (SPEC-0021), script coin spawns, the wheel game and the betting race (SPEC-0059). No script reaches the give host function in play (Open question 18). Dialogue lines pay too: 67 lines give item 135 (SPEC-0021, Neopoints, Sources).
- **Pack 1397 coin spawns (act 4).** One trigger, a button press that opens a mechanism, spawns 21 coins of 100 NP (2,100 NP). A second trigger spawns 12 coins of 100 NP (1,200 NP) and sets flag 349. Each runs once per visit. Total 3,300 NP.
- **Wheel game (packs 1023, 758 and 1358; High).** One rule set with fees of 15, 25 and 100 NP; its spin, stop, pay-out tables, flags and odds are specified in SPEC-0059, section 2. Its pay-outs are coins (ids 139 to 135 by the greedy split) given through the hero's add path. The betting race of pack 722 (SPEC-0059, section 4) also pays in coins (item 139).

### 12. Power growth

Confidence: High for the rules; Medium-Low for the counts by act.

- There are no experience points. Power grows only through (a) the stat-up items, Golden Negg (health and magic), Red Negg (health), Starry Negg (magic) and the two clovers (luck), with the pool steps of SPEC-0021 (Stat-up items) applied to both heroes, at most one of each item per area (section 8); and (b) equipment ladders bought in shops (section 10) or given by scripts (SPEC-0021, Sources).
- Without the conditional placements, a hero can gain at most 5 health steps (four Golden Negg areas and one Red Negg area) and 8 magic steps (four Golden Negg areas and four Starry Negg areas). The cap of SPEC-0036 needs the conditional placements (Medium-Low).
- The conditional placements are a clean partition by flag byte (High; survey of every world pack's current region; SPEC-0021, Sources):
  - flag 2: breakable drops. 1,239 are named by a breakable's drop id; 2 are named by nothing.
  - flag 3: enemy drops. 347 are named by an enemy spawn record's drop ordinal.
  - flag 4: type-40 group rewards, all 226: 138 Square-press chests (kind 1), 39 element-sealed chests (kind 2), 47 dig sites (kind 6) and 2 breakable chains (kind 4).

### 12.1 Large Neopoint stashes (High)

Every flag-4 Neopoint placement of 1,000 NP or more (key -1) has a named trigger. "Group 3" marks a placement that has the group-3 money roll (Open question 26). Keys are the collected keys of the chest or dig site.

| Pack | Count (NP) | Trigger |
| --- | ---: | --- |
| 1096 | 25,000 | Square-press chest (kind 1), collected key 190, no lock |
| 1096 | 6,000 | Square-press chest (kind 1), key 186, no lock |
| 1363 | 1,487 (group 3) | element-sealed chest (kind 2), key 136 |
| 1363 | 1,892 (group 3) | dig, key 241 |
| 699 | 4,118 | dig, key 210 |
| 886 | 4,118 | dig, key 209 |
| 1353 | 3,044 | dig, key 245 |
| 714 | 2,580 | dig, key 246 |
| 1350 | 2,256 (group 3) | dig, key 237 |
| 1066 | 1,853 | dig, key 218 |
| 1358 | 1,676 | dig, key 238 |
| 1238 | 1,643 | dig, key 226 |
| 772 | 1,327 | dig, key 205 |
| 956, 970 | 1,312 each | dig, key 216 |
| 1230 | 1,214 | dig, key 224 |
| 1225 | 1,116 | dig, key 222 |
| 934 | 1,057 | dig, key 230 |
| 986 | 1,024 | dig, key 217 |

- No level module of pack 1096 locks a chest, so both of its large chests open on a Square press.
- The 1,000 NP and 2,500 NP placements of pack 1096 are flag-2 breakable drops (breakables 25 and 32), not flag-4 stashes.

## Edge cases and error handling

- Add, remove or give with an amount of 0 or below: no change.
- Remove larger than the balance: the balance becomes 0.
- Give or remove with no wallet: no change. Query with no wallet: 0.
- Price 0x7FFF is tested before the balance. A price of 0x7FFF is never affordable, even with a large balance.
- A price stored as 0x8000 or above reads as negative. Such a price passes the balance test. If the add succeeds, the balance rises by the size of the price, and the price line shows the negative number (Medium-High).
- A sum that passes the 32-bit limit wraps negative and the wallet becomes 0 (section 1).
- A coin row with a blank name: show the item id when a name is needed. Do not invent a name. Its value still comes from the denomination word.
- A multiplier on a negative price: the value is multiplied and is not clamped at the bottom. The clamp applies only at the top (Medium).
- A multiplier call with no spawned-item list: nothing happens.
- Disc price range: the 152 shop placements hold prices 20 to 8,000 NP (48 distinct values); the largest after the x4 multiplier is 32,000, so no disc price is negative or capped at 32,767 by one call (a second x4 call on 8,000 would cap at 32,767).
- A buy-hook callback whose item id is not one of its four items: nothing happens.
- The nearest ready world item has a flag byte other than 1: there is no buyable object, even when a flag-1 item is also in range (section 3).
- Two flag-1 records of the same equipment id in one pack (for example two prices for one wand): buying either one reaches Q = 1 and removes both.
- A taken item reappears on the next region load unless a spawn veto applies (carry limit, once-per-area list, held map, equipment that is not an upgrade).

## Test vectors

Wallet (value 500 unless stated):

- Remove 800: 0.
- Remove -50: 500. Remove 0: 500.
- Give 2000: 2500. Give -10: 500.
- No wallet: query returns 0. Give 50: no change; query returns 0.

Coin pickups:

- Pick up id 139, then 137, then 142: +100, +10 and +0. Total +110.
- Pick up id 135 with a count of 3: +3.
- Pick up id 140: +0.

Buy path (balance 500 unless stated):

- Price 0x7FFF: never-affordable message. Balance 500.
- Price 0x7FFF with balance 40000: never-affordable message. The price test runs before the balance test.
- Price 600: need-more message for 100. Balance 500.
- Price 300, add succeeds: balance 200. No message.
- Price 300, add fails: can't-buy-now message. Balance 500.
- Price 300, add gate busy: can't-buy-now message; the queued add later gives the item with no charge. Balance 500 (Medium).
- Price -50 (stored as 0xFFCE), add succeeds: balance 550. The balance test passes because -50 is no more than 500.
- Purchase request slot busy: no price taken now; the purchase runs again later without the gate. Balance unchanged at this step.
- No buyable object: nothing shown.

Buy hooks:

- Pack 856 loaded. Hero buys item 113 with flag 220 clear: line 19 shown and no purchase. With flag 220 set: nothing shown.
- Pack 1140 loaded, flag 220 clear (flag 10 either way, since the hook is registered in both cases): buying item 116 shows line 0 and the buy is cancelled. With flag 220 set: nothing shown and the purchase runs.
- Pack 1433: buying item 109 with flag 248 clear: line 20 shown and no purchase. With flag 248 set: nothing shown and the purchase runs.

Price multiplier (factor 4.0; flag 298 clear):

- Active record with flag byte 1 and buy price 30: becomes 120.
- Active record with flag byte 1 and buy price 20000: becomes 32767 (clamped).
- Active record with flag byte 1 and buy price -1000: becomes -4000 (Medium).
- Active record with flag byte 0 and buy price 30: stays 30.
- Inactive record with flag byte 1 and buy price 30: stays 30.
- Flag 298 set: no call, values unchanged.
- Once per build: leave pack 1005 and re-enter it with flag 298 still clear. The interior is built again from the placement count 2000, the call is made once and the price shows 8000, not 32000. A record added to the list after the call keeps its placement count.
- Synthetic placement in pack 1005: flag 1, key 109, fixed count 2000. Flag 298 clear: the Buy prompt shows price 8000. Flag 298 set: price 2000.
- Pack 1005 in a playthrough from a new game, first visit (flags 296 and 297 set, 298 clear): Royal Armour shows 18000. After the Black Knight fight: 4500.
- Pack 1011, flag 298 clear: Red Negg 5000, map 31 1700. Balance 4999: the Red Negg buy shows the need-more message for 1.
- Pack 1014, flag 298 clear: Striped Negg 1100, Ergyfruit 500. Flag 298 set: Striped Negg 275, Ergyfruit 125.
- New game started at Act 3 Start (stage ids 0, 1, 2): flag 298 is set, so pack 1005 shows Golden Shield 2000 on the first visit.
- Buy hook in pack 856, flag 220 clear, Neopoints 500, buy of item 113 at 300: the hook's line 19 starts, the hook writes 1 into the record's flag word, and the buy is cancelled; balance stays 500. A second buy of 113 shows line 19 again and is cancelled again. For a hook that leaves the flag word at 0 (none exists on the disc), the same buy at 300 would complete after the line starts; balance 200.

Stock (balance 500 unless stated):

- Buying Speckled Negg (153, price 75): balance 425, and the item stays on display. A second buy: balance 350.
- Pack 1251 holds two Witch's Wand (114) records priced 3000 and 10000. Buying either one (Q = 1) removes both.
- Buying Golden Negg (151) in pack 984 (its own world index, SPEC-0025) with enough NP: both heroes gain the health and magic steps, and the item is taken. Re-entering pack 984: no Golden Negg on display. Buying a Golden Negg in pack 1423 (another world id) works once more.
- Pack 1136's map placement has byte +0x2C = 0x12, so buying it gives map 19. With map 19 already held, the item is not on display.

New game:

- New game: balance 0; inventory holds item 119 x1, 150 x3 and 149 x3, and no Golden Negg; the pools are at their start values (the Golden Negg's stat-up is undone by the pool reset); a Golden Negg bought afterwards in any area is accepted (its once-per-area list is empty).
- Wheel game and betting race: see the test vectors of SPEC-0059.

Spending and boot codes:

- Balance 60, removal of 100 (yes answer): balance 0.
- Balance 500, boot code 9: balance 1500. Balance 500, boot code 12: balance 10500. The caller is unknown, so these show only the arithmetic.

### Test vectors: real shop items from placement data (disc)

Pack 1005 (one of the five x4 interiors) holds three item points in its embedded region's record table, each of kind 8 with flag byte 1. Their data blocks share these values: facing (0.99994, 0, 0.01100), flag byte 1, snap-to-ground byte 0, drop group -1 (bytes 255 and 255), luck gate 0, treasure-map byte 255 (unused, the items are not maps), fill bytes 205.

| Record | Position (x, y, z) | Key (+0x24) | Second key (+0x26) | Fixed count (+0x2A) |
| --- | --- | --- | --- | --- |
| 1 | (5.862, 2.380, 5.869) | 131 (Noble Dress) | 1485 | bytes 100 and 25: 100 + 25 x 256 = 6500 |
| 2 | (5.917, 2.380, 0.896) | 123 (Royal Armour) | 185 | bytes 148 and 17: 148 + 17 x 256 = 4500 |
| 3 | (5.862, 2.380, 11.041) | 109 (Golden Shield) | 1811 | bytes 208 and 7: 208 + 7 x 256 = 2000 |

- Spawn, record 3: the 48 data bytes are copied into the world-item record. Its flag byte (+0x80) is 1, its key (+0x84) is 109 and its buy price (+0x8A) is 2000. Item 109's text-table row has carry limit Q = 1 and catalogue price 3000; the catalogue price plays no part (the shop price is 2000, and for record 2 the catalogue price 7500 differs from the placement's 4500).
- Multiplier with story flag 298 clear: the multiplier call (section 5) scales every flag-1 record: Golden Shield min(32767, trunc(2000 x 4.0)) = 8000; Royal Armour 4500 x 4.0 = 18000; Noble Dress 6500 x 4.0 = 26000. All three are below the cap, so none is clamped, and they match the before-the-Black-Knight prices of the table in section 5. With flag 298 set the call is not made and the prices are 2000, 4500 and 6500.
- Shown: standing at the Golden Shield with balance 9000 and flag 298 clear, caption line 1 is 8000 in the default colour (160, 160, 90, 127), because 8000 is no more than 9000, and line 2 is 9000 in (128, 128, 128, 127).
- Charged: confirming reads the price as a signed 16-bit value (8000, not 0x7FFF), which is no more than the balance. The item is equipment, so the add takes the world item; the add succeeds, the balance falls by 8000 to 1000 and no message is shown. The party now holds Q = 1 of item 109, so every world item of id 109 in the level is removed; records 1 and 2 stay.
- Balance 7999: line 1 shows 8000 in (128, 60, 90, 127) and confirming gives the need-more message for 8000 - 7999 = 1; the balance stays 7999.
- Flag 298 set, balance 9000: the prompt shows 2000 and the purchase leaves 7000.

## Open questions

Numbers of removed questions are not reused.

1. Section 3: The balance test runs before the add. An item whose price is above the balance is never added, so it is not kept. High.
2. Resolved for the disc: A stored price of 0x8000 or more is negative. It passes the balance test, and a successful add raises the balance by the size of the price (Medium-High). No disc data reaches it: all 152 shop placements (the buyable kind, in 34 packs) carry fixed prices from 20 to 8,000 NP, and with the price multiplier of 4.0 the largest is 32,000, below both the 32,767 cap and the sign bit. The only script call that rewrites prices is the price multiplier (SPEC-0030 row 197), which caps at 32,767 and so cannot make a positive price negative; a negative price needs a negative multiplier, and no known call passes one (High for the disc prices).
3. Resolved: step 2 is the buy hook; with a hook registered the purchase runs again after the hook unless the hook marks it handled, and no message is shown. The add's own check is the pickup hook, which no shop pack registers, so the free-item path does not occur with the disc's data (section 3; High).
6. Resolved (High for the thread shape; Medium-High that a finished thread is never run again): in each of the five modules the thread that makes the multiplier call runs straight through with no loop. It queries the shopkeeper flag, loads the character set, queries flag 298 and, when that flag is clear, makes the call once, then returns. The module's Init starts that thread once and nothing else starts it, so the call is made once for each build of the interior, and records added to the list later are not reached by it. A shop re-entered later is built again and is scaled again from the placement price (section 5, Effect), so the factor never compounds.
7. (Partly resolved) Buy-hook callbacks: the line repeats on each buy of the hook's item while its test flag is clear (section 4; Medium-High). Resolved (High): flag 220 is set by the Act 2 start (the act-start setup called with 2 by pack 1174, SPEC-0049, Act-start setup) and flag 248 by the Act 3 start (the same setup called with 3 by pack 1327), so in a normal game 220 is on from the start of Act 2 and 248 from the start of Act 3, and the four shop hooks (856, 859, 1140 and 1433) stop cancelling the purchase from those points. The stage table is a separate namespace: its stage 0 rows also set 220 and its stage 1 rows also set 248, but those rows are applied only by the developer start-point chooser (start indices 1, 2, 3, 6, 7 and 8 for stage 0, 2, 3, 6, 7 and 8 for stage 1), which the shipped title screen never opens. Pack 1174 also sets 220 when its trigger fires. No level script sets 248 with a flag call; the act start is its only setter in play (SPEC-0049, list of flags with no script setter). The trigger condition of pack 1174's set of flag 220 is tracked in SPEC-0049, section 4 (not this spec's item; flag 220 is also set by the act-start stage 0 row, so the shop hooks do not depend on it). Flag 248 also picks the find-game target set of pack 934 (SPEC-0059, section 6).
10. Resolved (High): no grant code is reachable in play. The event handler of section 7 has no direct call, no jump and no pointer to it anywhere in the boot program's code or data, and none of the 173 distinct level modules forms a boot-program data address, so no script reaches it by pointer either. Its Neopoint additions and item grants never run.
11. Resolved (High for the data): the item text resource holds name strings for the coin rows 135, 136, 138 and 139 (1, 5, 25 and 100 Neopoints) and for the Tal rows 140, 141, 143 and 144 (1, 5, 25 and 100). It holds no string for a ten-Neopoint or a ten-Tal coin, so the name pointers of rows 137 and 142 are null and nothing sets them elsewhere. Show the item id when a name is needed (Edge cases).
12. Resolved (High for the data, Medium-High for the code): no placement, drop table, script grant or boot grant names item ids 140 to 144, and no code path adds them. They are a second coin set with the values 1, 5, 25 and 100, with a config type of 0 and all stats 0, and are unused data. The coin id words (section 2) list only ids 135 to 139.
15. Resolved in part: pack 722's 50-NP race prompt uses a strict below-50 insufficient-funds check; exactly 50 reaches the paid branch (SPEC-0059, section 3.4; High). Resolved (High): every other script spending prompt refuses only when the balance is below the price, so an exact balance buys (section 6).
16. Resolved (High): the steal minimum and maximum fields are live. An attack slot with the steal bit and selector 0 rolls its steal chance, draws a whole amount between the two fields (swapped when reversed), removes it from the wallet and spawns it as coins at the hero (SPEC-0029, Behavior 11a). On disc the Thief takes 10 to 100 and the Thug 20 to 200 at chance 50; the Mountain Crokabek's 0 to 0 takes nothing. Selectors 1 to 3 take an item instead and raise the enemy's carried Neopoints by the item's catalogue price.
18. Resolved (High for the module sites; Medium-High for the boot program): 28 module call sites of the give host function exist, in 27 level modules. 27 of them sit in each module's shared debug cheat routine, which starts with the debug-mode query that is false in play. The 28th (module 842, an amount of 2000 behind a button-combination test) sits in a routine that no call and no pointer reaches. In the boot program the give wrapper is entered only through its host row. So no play path gives Neopoints by script, and income stays coins only (section 11).
19. Section 5: SPEC-0030 row 197 is correct. The multiplier scales the buy price of flag-1 items, capped at 32767.
21. Resolved: the veto tests the entry fixed by the item's config type, Tor's for types 4 to 6 and Roberta's for types 1 to 3, whatever hero is active (section 8; High).
23. Resolved: yes. Market Town and its shops open when flag 296 is set at the Crossroads, before the Black Knight quest starts (flag 297) and before the fight sets flag 298, so a playthrough from a new game sees the x4 prices. They are first entered in act 3, with both heroes. A start-point chooser start at Act 3 Start or later sets flag 298 and skips them (section 5; High).
24. Resolved: the Buy prompt does not set it. The 323-bit set is written only on an ask-about-item branch of the world prompt that the game never takes, and nothing reads it; it is not an ownership record (SPEC-0021, Inventory; High).
26. Resolved: a dig pays the flag-4 placement that the dig site names (SPEC-0051) through the ordinary drop action. Of the 44 real treasures, 25 pay items and 19 pay Neopoints as coins (fixed counts 653 to 4,118 NP, 33,499 NP in all; three with a group-3 money roll, a 30 % chance of 400 to 1,000 NP). A treasure row's first word is its name text, not a world-pack number.
27. (Partly) The exact act of each shop pack (section 10). Market Town's packs 1005, 1008, 1011, 1014 and 1020 are act 3 (High; section 5). The other groups are inferred from flag stage ids and movies, not from an area graph; play does not read the act of a shop, so the labels are a presentation aid and need no further check (class D, open close-out). Resolved in part (Medium-High): the door-reached packs behind pack 722 (Bogshot Village) and pack 758 (Brightvale Castle) are first used on the main path in act 3, with both heroes: the story enters pack 758 at step 29 and pack 722 at step 31 of SPEC-0058, after the act 3 start at step 27, and its apothecary and bridge builder interiors at steps 32 and 34. No door gate of their own is known. Packs 1429, 1430 and 1431 are not visited in play, and their scripts never run there.
28. Resolved (SPEC-0059, section 2; High): the numbering is self-consistent. The cell scored at a stop, 7 - floor(a / 45), is the cell whose lit art shows, and the player can time the stop.
29. Resolved. (1) The need-more text is the prefix, the amount, then one suffix string of two characters: character code 36 (the dollar sign in the text table, which the font draws as the currency mark; Medium for the drawn form) and "!". (2) The buy path writes no equipment word (High: the buy and add calls reach no equipment-word setter). A bought equipment item of types 1 to 6 is added to the hero's category list, which is where the weapon, shield and body entries live (SPEC-0021, Equipment slots), so it is in use as soon as it is added (Medium-High). A bought charm or Mote is not equipped by buying; it is equipped only from the Mote picker (High).

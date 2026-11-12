# SPEC-0030: Level script modules and host API

- Answers: R-0279 (look-ahead is not limited to the region's list); proactive (open question sweep, round 10); proactive (open question sweep, round 9); proactive (open question sweep, round 5); proactive (open question sweep, leftovers); proactive (none requested); proactive (open question sweep, world and talk); proactive (open question sweep, slot -1); proactive (open question sweep, script threads); proactive (path walkers); extends R-0006; R-0127 (rows 111, 113 and 339: the path index and the path list); R-0049 (thread end and hold exits answered; CTL sources open); R-0048 (object state writers 5 and 6 in part); R-0063 (movable block pass timing); R-0064 (activation filter of module 985); R-0081 (switch-key boundary for pack 951); R-0104 (aim-scan vectors, range, sight-line, return values and stored world-space shot aim point); R-0116 (Baggus list chunk, runtime layout, shared model selection, pack-735 source-to-record mapping, placement-height input and bounds; numeric ground heights for all five sources (16 to 20) and the source-to-transform mapping are resolved; selected template basis values resolved for pack 735); R-0121 (aim-sight sample spacing and tested segment; outbound world speed and phase gravity remain open); R-0119 (type-29 handle-array layout and pack-735 candidate resolved; exact target mesh, unbroken draw, burst selector and trigger-pair roles remain open); R-0135 (a grab or animation call on an unbound entity does nothing); R-0147 (SetAI_Agro on a waiting state-8 enemy); R-0175 (row 165 use in pack 1428's boss-room return), R-0192 (row 90 SetExternalMissileTarget); R-0200 (row 221 CharacterSetAvoidPoint: record layout, the one caller and why it has no effect); R-0223 (row 222 CharacterIndex: the counter is the record ordinal; section 3 fire gate for pack 1398 trigger 34); R-0224 (rows 165, 275 and 281: what the boss refill does and its pack 1246 use, the dead Update call, the start pose having no effect); proactive (open question sweep, flag watcher re-arm); proactive (open question sweep, script details); proactive (open question sweep, item pickup recheck); proactive (open question sweep, round 6); proactive (disassembly confirmation); proactive (open question sweep, round 12); R-0231 (answered from the code; not run in the original); R-0239 (answered from the code; not run in the original); R-0257 (the return door of pack 1098 is context 13; answered from the code, not run in the original); R-0270 (row 124 follow rule, host and call sites; answered from the code, not run in the original); R-0273 (row 126 sight sample points; answered from the code); R-0282 (which bodies fire plain trigger volumes: pack 1335 triggers 8 and 9 are fired by the hero only; answered from the code, not run in the original)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001). Module counts below are counted from the disc: 173 modules, 163 distinct payloads, 192 world packs in archive resources 532 to 1433.
- Confidence: per section (High, Medium, Medium-Low or Low). Interface names are listed as written in the game's own calls.

## Summary

Each world pack (SPEC-0006) may carry one level script module: a top-level chunk of type 37 whose payload begins with the ASCII bytes "SNR2". The module is a relocatable machine-code image with a small symbol table. It exports three entry points that come in three families (Exterior, Interior, Transition). The module calls a fixed host API. The catalogue in section 2 of Behavior has 388 rows, one per host overload: rows 1 to 285 for the first group of host names, rows 286 to 294 for host names that had no row, and rows 295 to 388 (categories 16 to 23) for the rest. Every host name that has a body in the boot program now has a row. Those calls set game flags, start animations, walk characters, play cameras and sounds, register triggers and callbacks, start script threads, load character regions and start world transitions.

The clean side will not run the module's machine code. Instead, each level's behavior is written in later per-level specs as plain lists and small state machines that use the action vocabulary of this API (section 7 of Behavior). This spec defines the container, the three families and their entry roles, the host API catalogue, and the timing, trigger, slot and transition rules that those per-level specs will rely on.

## Where it lives on the disc

- World packs are archive resources (SPEC-0001). Of the 192 resources in the range 532 to 1433 whose first chunk is the world header (type 31), 173 carry a level module. The other 19 carry none.
- The module is the payload of the top-level chunk of type 37 in that pack. Chunk framing follows SPEC-0002. The payload starts at the chunk's offset + 16 + the chunk's "extra" field, and the module length is the chunk size - 16 - extra. The length check holds for all 173 modules.
- Repeated modules: 10 packs repeat another payload exactly. The repeat groups are 678 = 686 = 691; 798 = 861 = 1004 = 1082; 956 = 970; 1077 = 1129; 1429 = 1430 = 1431 = 1433. So 163 distinct payloads exist.
- Reading module data tables (rule used by the per-level specs, e.g. SPEC-0044 pack 1091; High). Level-script specs give a data table as a module offset, an element type, a count and a role, and do not copy the values. Every such offset is counted from the first byte of the module payload (the "SNR2" bytes), which is also the link base of the code, so a table at offset X is at payload start + X in the pack's resource. Data words that hold plain numbers (counts, ids, 16-bit times, flags) are the same in the file and in memory; only the words named by the fixup table (addresses and call targets, section 1.3) change at link time. A reader should check a table against the test vector given in its spec (distinct-value counts, sums, first and last element) before using it.
- No tunable values are read from the boot program's data for this spec. Every number below is a container field, an interface constant, or a rule written in words.
- Host name table (boot program data, High, confirmed on disc). The boot program holds one table of 406 records of 12 bytes, starting at ELF data address 0x3030A0. Record 0 is an empty placeholder. Records 1 to 396 are host functions, and records 397 to 405 are the nine family entry names (Init, Update and Exit of the three families; Open question 48). Each record holds, in order: the address of its name string, the code address of its bound function (zero for the nine family names), and a key. The key's low 16 bits are the link number, and bits 16 to 23 are the kind (2 for host functions, 1 for family names). Every import and export of the 173 modules matches a table record by name, and its link number equals the key's low 16 bits (all 5,171 imports and 509 exports; Open question 47 is answered). Record 1 is the library memset routine. Every host record has at least one catalogue row. Overloads were matched to rows in table order; Every overload now agrees with its host name (Open questions 22 and 23).

## Layout

### 1. Module container (Confidence: High unless marked)

All offsets are from the start of the module (the "SNR2" bytes). All fields are little-endian 32-bit unsigned integers unless noted.

| Offset | Size | Field | Notes |
| --- | --- | --- | --- |
| +0x00 | 4 | Magic | ASCII "SNR2". Present in all 173 modules. |
| +0x04 | 4 | Fixup table offset | Table of 12-byte patch records (section 1.3). |
| +0x08 | 4 | Fixup record count | 24,368 records in total across the 173 modules, all of one patch kind. |
| +0x0C | 4 | Symbol table offset | Table of 12-byte symbol records (section 1.1). |
| +0x10 | 4 | Symbol record count | Record 0 is an empty placeholder. Symbol records total 5,171 import and 509 export entries across the 173 modules. |
| +0x14 | 4 | Module name offset | Points at the null-terminated string "levelDLL.elf" in all 173 modules. |
| +0x18 | 4 | Load hook reference | High (corrected 2026-11-09, script-runtime pass): one routine, not a table. The loader adds the load base to it and calls it once with no arguments, after the module has been registered, when it is nonzero. The load hook runs the module's load-time constructors (Which level routines run in play); their effects are part of the per-level behavior, and families are still taken from the export names. |
| +0x1C | 4 | Unload hook reference | High (2026-11-09): one routine, relocated like the load hook and called once before the module is unlinked, when nonzero. Not needed by the clean side. |
| +0x20 | 4 | End pointer | Equal to the value at +0x38 in all 173 modules. Refined 2026-11-09: the loader treats it as the base of an "extra word" table whose count is the word at +0x24; every nonzero word of that table gets the load base added, and the loader hands this pointer back to its caller only when the count is nonzero. With the count zero in every disc sample the table is never walked. |
| +0x24 | 4 | Extra word count | Zero in all samples (High, 2026-11-09: confirmed as the count of the table at +0x20, so the table is never walked on disc). |
| +0x28 | 4 | Alignment | 0x80 in all 173 modules. |
| +0x2C | 4 | Module size | Equal to the payload length, or up to 3 bytes less (padding). Confirmed for all 173. |
| +0x30 | 4 | Reserved | Zero in all 173 modules. Corrected 2026-11-09: not a data field. The loader writes zero here itself and later uses it as the run-time link to the next loaded module; each load appends the module at the end of the chain of loaded modules. A reader of the file can ignore it. |
| +0x34 | 4 | Compressed fixup stream offset | Section 1.4. |
| +0x38 | 4 | End pointer copy | Equal to +0x20 in all 173 modules. |

Machine code starts at +0x100 (the first non-zero word after the zero pad, aligned to 0x80). The code section ends where the data begins; data follows the code and holds the module's constant strings, the symbol names, the fixup table, the symbol table and the compressed stream, in that order of position in the samples (Medium). The clean side does not need to know where the code ends.

#### 1.1 Symbol record (12 bytes)

| Field | Size | Meaning |
| --- | --- | --- |
| +0 | 4 | Offset of the symbol's name (a null-terminated string, module-relative). |
| +4 | 4 | Value. For an export, the module-relative location of the entry. For an import, 0. |
| +8 | 4 | Low 16 bits: link number used to bind the import to a host function. Bits 16 to 23: kind. Exports carry kind 2. The binding uses the link number only. Refined 2026-11-09 (High): bits 24 to 31 (byte +11) are a run-time "bound" byte that the loader clears at load and sets to 1 for kinds 2 to 4. Kinds 0 to 4 exist: 0 is a plain local (value gets the load base), 1 an import (value is the looked-up host address, no base added), 2 an export (base added; its registration must succeed), 3 a second export kind (base added, no failure path) and 4 a bound value with no base added (registration must succeed). The disc modules use only 1 and 2. A kind above 4 aborts the load. |

Import names are interface names of the host API (section 2). Export names are the three entry names of section 1.2. High for the link-number rule (confirmed on every import and export on the disc).

#### 1.2 Entry names and families

| Family | Init (receives the world handle) | Update (no arguments) | Exit (no arguments) | Packs (of 173) |
| --- | --- | --- | --- | --- |
| Exterior | Init entry | Update entry | Exit entry | 87 |
| Interior | Init entry | Update entry | Exit entry | 50 |
| Transition | Init entry | Update entry | Exit entry | 36 |

On disc, Init and Update are exported in every module of a family. Exit is missing in 10 packs: 8 Interior, 1 Transition and 1 Exterior. The distinct payload counts are 85, 47 and 31 (Medium, from the same survey). The Init entry a module exports names the module's own family (Exterior, Interior or Transition). At run time the entries that are called are chosen by the buffer the pack is loaded into, not by the export (Behavior section 1.1).

#### 1.3 Fixup records (12 bytes, module-relative table at +0x04)

| Field | Size | Meaning |
| --- | --- | --- |
| +0 | 4 | Site: module-relative location of the word to patch. |
| +4 | 4 | Low byte: patch kind. Upper bits: symbol index of the import the site refers to. |
| +8 | 4 | Addend, zero in all records. The patched value is the symbol's value plus the addend (confirmed 2026-11-09); a failed patch aborts the load. |

All 24,368 records use one patch kind (a 26-bit jump field), and each names an import. This describes the module's calls into the host API. The clean side does not patch or run the code, so it does not need these records.

#### 1.4 Compressed fixup stream (at +0x34)

A byte-coded stream of further patch operations (low half, high half, jump field and full word) that ends with a zero cursor. Confirmed and detailed 2026-11-09 (High): the low 2 bits of each byte are the kind (0 low half, 1 high half, 2 26-bit jump field, 3 full word) and the upper 6 bits the distance to add to the running site address; a zero distance means a 32-bit absolute site follows (a zero site ends the stream). A high-half entry looks ahead to the next low-half entry and uses that low half, sign-extended, for the carry; a missing low half before the end aborts the load. The loader also adds the load base to header words +0x04, +0x0C, +0x14, +0x18, +0x1C, +0x20, +0x34 and +0x38 when each is nonzero, applies the symbol-record rules of section 1.1, then the fixup table, then the stream, and finally appends the module to the chain; its order is header check, relocation, registration, load hook, and the family's Init runs afterwards. Counts seen: 24,451 low-half, 12,997 high-half, 6,240 jump-field and 1,236 full-word operations (Medium). Not needed by the clean side. Listed only so that the container is fully described.

#### 1.5 Host-call binding (Confidence: High for the direct binding and the unchanged arguments; Medium-High for the link-time order)

A call from a level module into the host API is a direct call. Its target is not known when the module is built; the loader resolves it when the module is linked, before the family's Init runs. The loader takes the import's link number, finds the host record with the same name and link number in the boot program's host table, and writes that record's bound function into the call. The bound value is used as it is, with no module base added. No adapter, thunk or jump table sits between the module and the host function, and none changes the argument values: the host function receives exactly the values the module placed in its arguments. The clean side does not patch or run the module, so this fact matters only for reading argument values as they are passed. Any rule that depends on an argument value (for example the slot rule in row 205) applies to the value as the module built it.

#### 1.6 Locating a level module's data tables (Confidence: High for the addressing rule and the data area; Medium for where the code ends)

Every per-level spec that says a value is "held in module data" gives it as a module offset. This is how the rewrite finds such a table, list, text, colour or state word from the pack.

- **One flat image, linked in place.** The module is the payload of the pack's type-37 chunk (Where it lives on the disc). The loader reads the pack into the level buffer, checks the signature, adds the buffer address to every address the module stores about itself (the header fields at +0x04, +0x0C, +0x14, +0x18, +0x1C, +0x20, +0x34 and +0x38) and to every site named by the fixup table and the compressed stream, registers the module and runs the load hook. Nothing is copied, split or reordered: code, tables, text and state words stay at their payload positions.
- **Module offset.** The module is built as if it were loaded at address 0, so every address in its code and tables is a position in the payload. "Module offset X" in a level spec means the byte at position X of the chunk payload (position 0 is the first byte of the ASCII "SNR2"). The rewrite reads the table at X in the payload and never needs the buffer address. A pack that repeats another pack's payload (Where it lives on the disc) has the same offsets. The data area has no section table in the header: the code starts at +0x100, the data follows the last routine and runs to the size field at +0x2C (constants, text, routine tables, read-only tables and then the zero-initialised state words), and a table is identified by its offset alone. Every table and word named in the level specs lies below the size field.
- **Cell types.** All cells are little-endian. Ids, flags, counts, slot numbers and indices are 32-bit signed integers (-1 is the usual "none" or "empty"). Angles, times and speeds are 32-bit IEEE floats. A colour is three bytes (red, green, blue, 0 to 255) at the offset given. A record table gives its stride in bytes and its field offsets inside a record; a list gives its length, and a list that ends at a marker says so (for example "ends at -1").
- **Pointer cells.** A cell that holds the address of a routine, table or text (routine tables, constructor and destructor lists, list pointers inside a record) stores a module offset in the payload, and the loader adds the buffer address at link time. The rewrite uses the stored value as a module offset and does not add anything. A level spec does not list routine offsets; it names the routine by its role.
- **Two kinds of table.** (1) Disc tables: the values are in the payload and the rewrite reads them from there; the level spec gives offset, stride, count, field roles and a check value, and does not copy the values. (2) Code-filled tables: the payload holds zeros (or -1) and the module's Init, constructor or set-up routine writes constants into the table at load. These values are not on the disc as data; they are code constants, so the level spec prints them in full and gives the offset only so that the table, its stride and its fields can be recognised. A level spec says which kind each table is.
- **State words.** A counter, latch, done word, slot word or stored world pointer starts at its payload value (usually 0, sometimes -1) and is live state afterwards (Module data words are live state, section 7). Where a spec says "file value", it means the payload value.
- **Slot tables.** The array passed to SlotCreate is a list of n region indices at a module offset. Its values only size the slot buffer (row 207) and are never loaded, so a level spec lists its offset, count and the word that receives the result, and the rewrite needs only the count.
- **Check.** Each "Module data tables" entry in a level spec ends with a small check value (a count, first and last value, or a sum) that the rewrite can verify after reading the table.

### 2. Name encoding of host calls (Confidence: High)

Each import name is the function name, then two underscores, then the encoded parameter list. The class scope is part of the name and is the same for all host calls. The clean side should match calls by function name and by argument list. The parameter codes used in this spec are:

| Code in name | Type in this spec |
| --- | --- |
| i | int |
| f | float |
| b | bool |
| (string class codes), ... | string pointer, or variable argument list (debug print only) |
| (world-assets class code) | WORLD (world handle, written W) |
| (wait-option class code) | WAIT, a wait mode |
| (control-option class code) | CTL, a control mode |
| (camera-wait class code) | CAMW, a camera wait mode |
| (world-identifier class code) | eW, a world identifier |
| (colour class codes) | RGB colour by value / RGB pointer / RGBA pointer |
| (position-vector class code) | VEC, a three-float position pointer |
| (hero class codes) | hero selector (which hero), HERO (hero object) |
| (callback class codes) | callback; written callback(X) where X is the argument type the callback receives |
| (output class codes) | output references (float outputs), written "out" |
| T n | same type as argument number n |
| e | variable argument list |

#### 2.1 Enum values used by the level modules (Confidence: High)

Every host-call site in the 163 distinct modules was evaluated. Values seen, with site counts, and how the host reads each:

| Enum | Values on disc (sites) | How the host reads it |
| --- | --- | --- |
| Wait option (WAIT) | 0 (1,484), 1 (1,742) | Tested as nonzero (nonzero = wait), except PlayEntityAnim (row 105), which waits only when the value is exactly 1. No other value occurs. |
| Control option (CTL) | 0 (436), 1 (710) | Compared with 1. 1 = when the scripted action ends, the character returns to free control (state 1). Any other value = it stays under script control (state 0x21). The choice is stored on the character when the call starts and read when its scripted state ends. MoveHero, MoveEntity and the walk and play calls apply it. |
| Camera-wait option (CAMW) | 0 (36), 1 (481), 2 (6) | End mode of a scripted camera play (Behavior section 2, Note on camera calls). It is not an entity index. |
| Hero selector | -1 (51), 0 (31), 1 (29) | -1 = the active hero; 0 = Tor; 1 = Roberta. |
| Camera shake type | 0 (16), 1 (2), 2 (2) | Shake preset (0 to 5 are valid; SPEC-0027). |
| Boss-fight stage | 0 (14), 1 (14); instances 0 to 3 and 5 to 14 | Row 165. |
| Swap mode | 0, 1, 2, 3, 5 | Row 167. |
| Axis selector | 2 only (17) | Row 179: 0 and 1 select the first two stored direction vectors, any other value the third. |
| Blackboard (message) type | 1 (7), 2 (191), 3 (9) | Rows 149 and 150. Type 2 is the arrival tag that activate-world callbacks read. |
| Button code | 0, 1, 2, 3, 5, 7, 8, 10, 11, 13 | Row 95; 45 sites pass code 7, which is always false. |
| World identifier (eW) | 5 to 192 | World slot ids (SetWorldLoadOverRide accepts values below 193). |

### 3. Callback information record (Confidence: High for the writes and init values; Medium for which callers run in play)

Trigger callbacks receive a pointer to the collision block of the trigger record. Offsets below are from the start of that block (consistent in the sampled modules):

| Offset | Size | Meaning |
| --- | --- | --- |
| +0x0C | 4 | Actor: the actor passed to the fire step; zero when there is none. |
| +0x10 | 4 | State word: 0 = idle; 1 = first fire; 2 = repeated fire; 3 = decaying toward 0. |
| +0x14 | 4 | Trigger index (the index it was registered under). |
| +0x18 | 4 | Frame stamp: the frame counter at the last fire; -1 before the first fire. |
| +0x1C | 4 | Object: the object value passed to the fire step; null when the caller passes none. |
| +0x20 | 4 | Shot word: 1 when a hero shot fired the trigger (the hero shot pass, SPEC-0028 Behavior, Trigger pass), else 0 on every other fire path. Written only when the record is written (see the update rule below). (High) |
| +0x24 | 4 | Attack word: on a body-contact fire, the value of the owner's class attack query at that moment. For the selected hero (object 1) this is its attack-active word: 1 while the current animation frame carries event bit 0x1 (SPEC-0036, Attack window), else 0. For any other character (object 0) it is 1 when any of the five low bits of its motion state word is set. 0 on the shot, movable block, projectile and forced paths. A weapon strike that the hero's body does not accompany never reaches a script trigger. (High) |
| +0x28 | 4 | Last fire argument: -1 for hero fires; the movable block's own number when a movable-block contact fired the trigger. (High) |

Consequences (High): a callback that requires +0x24 = 1 runs only when the selected hero's body is inside the volume during a hit-window frame; a callback that requires +0x20 = 1 counts only hero shots; a callback that requires +0x20 = 0 and +0x24 = 0 ignores shots and attacking bodies (a non-attacking hero, an idle character, a movable block or a forced fire still passes); a callback that requires the object word (+0x1C) to be 0 ignores the selected hero's body contact altogether, and one that requires it to be non-zero counts only that contact. Scripts use +0x28 to tell which movable block arrived.

A trigger record holds this collision block at its offset +0x14. Registering a trigger writes the callback, an owner value of -1, a word of 0 and the trigger index into the trigger record. Registering the same index twice keeps the later routine, because the second registration overwrites the stored callback (pack 758 registers index 5 twice with different routines; pack 1023 registers index 13 twice with the same routine) (Medium-High). Table init sets the object to null, the state to 0 and the stamp to -1.

The trigger's fire step starts the callback when a volume carrying that trigger's index is hit. The fire step writes the record: the actor, the frame stamp (the frame counter), the object (the caller's value) and the state.

State changes (High):
- Update rule: the fire step writes the record (actor, the three block words, the object, the lookup word, the stamp and the state) only when the object it is given is non-zero, or the record's state is 0, or the record's object is 0 and the last argument is 0 or above. Every volume path passes -1 as the last argument except the movable block pass (the block's number). So a fire with object 0 from a shot, a character, a projectile or ForceTriggerScriptIfNotRunning changes only an idle record; it does not refresh the stamp of an active one.
- When the record is written with a non-zero object while its object was 0 and its state was not 0, the state becomes 1 again.
- The first fire moves the state from 0 to 1.
- A later written fire moves the state from 1 to 2 when the stamp differs from the current frame; a state of 2 or 3 is kept, and the stamp becomes the current frame.
- Decay, in the script step after the record's thread is resumed: when the state is not 0 and the stamp s satisfies 0 <= s < frame - 1, the state toggles: 3 becomes 0 and any other value becomes 3. A record last written on frame s therefore reads 3 from the script step of frame s + 2 and 0 from frame s + 3. An object-0 firer that stays inside a record it started cycles 1, 1, 3, 0, 1, ... with a fresh write on every fourth frame.

The callback runs as a short-lived thread. The fire step starts it whenever the record's thread handle is empty and a callback is registered, whether or not the update rule wrote the record. It receives the collision block and runs once per thread life. The object value is 1 (non-null) only when the selected hero's own body contact fires the trigger. Every other fire path passes 0: the hero shot pass, the movable block pass, the world projectile step, every non-hero character's body contact and ForceTriggerScriptIfNotRunning (row 275). High (Behavior, section 3, Fire paths).

### 4. Slot record (runtime) (Confidence: High for offsets, counts and the region lists; Medium for the other field meanings)

A slot is a fixed-size record of 2960 bytes, kept in a per-world table. A slot holds one character region's load, a buffer sized for that region, up to 15 active characters (set up from the region's character records when the read completes), and the region's other lists, including the item placements that activation spawns. The table sits behind the world-assets object, which holds the pack number, the slot capacity, the created-slot count (the cursor that SlotCreate advances), and pointers to the slot records, their handles, their buffers and their region numbers. Only the fields that the slot calls use are listed.

World-assets object (offsets from the start of the object):

| Offset | Size | Meaning | Confidence |
| --- | --- | --- | --- |
| +0x00 | 4 | Pack number; region = pack + 1 + j, or j + 1 when the pack number is negative | High |
| +0x04 | 4 | Archive handle, passed to region size and load | Medium |
| +0x14 | 4 | Heap handle, passed to allocations | High |
| +0x24 | 4 | Id base; slot record i gets id base + 50 (i + 1) | Medium (meaning Low) |
| +0x30 | 4 | Init parameter, copied into each slot record at load start | Medium |
| +0x38 | 4 | Character descriptor table: 16-byte records, byte 0 = region index | High |
| +0x3C | 4 | Count of descriptors | High |
| +0x6C | 4 | Created-slot count (cursor): the next slot number SlotCreate hands out; reset to 0 by a world-level reset | High |
| +0x70 | 4 | Slot capacity n (from SlotAllocate) | High |
| +0x74 | 4 | Pointer to the slot records (stride 2960; a 16-byte header before record 0 holds n) | High |
| +0x78 | 4 | Pointer to n handles of 20 bytes (slot i at +0x78 + 20 i) | High |
| +0x7C | 4 | Pointer to n buffer pointers, written by SlotCreate | High |
| +0x80 | 4 | Pointer to n region numbers; -1 = empty | High |

Slot record i (base = slot table + 2960 i):

| Offset | Size | Meaning | Confidence |
| --- | --- | --- | --- |
| +0x04 | 4 | Archive handle (copied from the world-assets object) | High |
| +0x0C | 4 | Result of the last region read (row 205, step 6): set to 0 by the release of every SlotLoad and SlotReset, then 1 when a read completes with the IDM signature | High |
| +0x10 | 4 | Load state: nonzero while busy (1 while a load runs), 0 when done | High (busy use), Medium (other states) |
| +0x14 | 4 | Pointer to the slot's 20-byte handle | High |
| +0x18 | 4 | Pointer to the loaded region data; the completion step checks that it starts with the IDM chunk signature | Medium |
| +0x24 | 4 | Id base of this slot | Medium |
| +0x30 | 4 | Init parameter (copied from the world-assets object) | Medium |
| +0x38, +0x3C | 4 each | Back pointer to the world-assets object; slot number i (-2 after a release, until SlotLoad writes the slot number) | High |
| +0x40 | 4 | Flag word: 0x80000000 when a handle was attached, else 0x40000000 | Medium |
| +0x44 | 40 x 4 | Character records (kind-3 entries of the region's type-36 records; count at +0xAE4, cap 40) | High |
| +0x184 | 8 x 4 | Checkpoints (kind 4; count at +0xAE5, cap 8) | High |
| +0x1A4 | 120 x 4 | Item placements (kind-8 entries; count at +0xAE6, cap 120); not character records | High |
| +0x384 | 40 x 4 | Animation tables (kind 2; count at +0xAE7, cap 40) | High |
| +0x424 | 40 x 4 | Effect records (kind 0x10; count at +0xAE8, cap 40) | High |
| +0x4C4 | 15 x 4 | Character object pointers (cap 15), indexed in parallel with the character-record pointers of the same block; the object's field +0x3B0 is read to find its animation-table entry | High |
| +0x988 | 4 | Handle of the kind-0x11 fixup data | High |
| +0xAE4 to +0xAE8 | 1 each | Counts of the character records, checkpoints, item placements, animation tables and effect records, in that order; cleared by the release | High |
| +0xAE9 | 1 | Active character count (cap 15): the number of successful character setups at read completion; SlotCharacterCount returns it | High |
| +0xAF1 | 1 | Character cap: 6 after slot construction (the construction sets 15 and then runs the reset, which sets 6), after SlotReset and after every release; 15 after a load with keep false; a load with keep true leaves the 6 set by its release; never above 15 | High |
| +0xAF2 | 1 | Count of characters attached so far (counted up by activation routines; confirmed 2026-11-09: the force-bind routine refuses to bind when it has reached the cap at +0xAF1 and adds 1 after each successful bind) | High |
| +0xAF3 + i | 1 each | Id byte of record i, stored when a forced bind succeeds (added 2026-11-09) | High |
| +0xB04 + 4 i | 4 each | Bound flag of record i (0 = free; a forced bind with index -1 takes the first record whose flag is 0; added 2026-11-09) | High |
| +0xB44 + 4 n | 4 each | Attached object list: the force-bind appends the new object at entry n = the attached count (added 2026-11-09) | High |
| +0xB80, +0xB88 | 4 each | Reset to 0 and to -1 by a forced bind (meaning not established; added 2026-11-09) | Medium |
| +0xB40 | 4 | Keep (streaming) flag: 1 = the per-tick update runs the full streaming pass (releases dead and far characters and binds the nearest eligible records, SPEC-0006); 0 = the per-tick update only releases dead characters whose death countdown ended and binds nothing (section 5). 1 after construction and SlotReset | High |

Limits: there is no upper bound on the capacity n in the code. At most n buffers exist per world, because the created count stops at n. A slot holds at most 15 active characters. A region read counts up to 40 character records, 8 checkpoints, 120 item placements, 40 animation tables and 40 effect records. There is no free call: SlotReset and every SlotLoad release the slot's content, and the buffers are kept until a world-level reset empties every created slot and sets the created count to 0. The slot table and its buffers are carved out of the world's linear memory arena (a bump allocator that hands out the next aligned block and has no per-block release; it can only be rewound as a whole), so nothing is ever freed one slot at a time and the memory lives as long as the world's arena does (High). A rewrite may therefore allocate the table once at the first SlotAllocate and drop it with the world.

## Behavior

### 1. Module families and entry roles (Confidence: High for roles and for the family rule)

- The host selects the family for each world by the buffer that holds it. Kind 1 (Exterior) is the exterior buffer E, kind 2 (Interior) the interior buffer I, and any other world, the transition buffer T, is kind 3 (Transition) (High; section 1.1).
- **Init(world)**: when the world's module is attached, the host resolves the family's Init, Update and Exit exports by name, stores them on the world and calls Init(world) once, at once, as a plain call (not a script thread), inside the pack's load-completion step, before the arrival placement and before the world is made current (section 6, Init order). It sets up what the level needs: slots (section 5), control threads (section 4), triggers and callbacks (section 3), animation objects, and any initial flags. Init has a body in 167 of 173 modules. High.
- **Update()**: runs as a one-shot script thread. The world's per-tick script step (below) creates a new Update thread whenever the world has an Update entry and no Update thread is alive; that thread first runs in the next tick. So Update runs once per tick while it never yields, and an Update that yields suppresses new Update runs until it ends. It is empty in 96 of 173 modules. Where it has a body, it typically runs one per-tick routine that checks flags and starts or stops objects. High.
- **Exit()**: called when the world's script manager is torn down, just before the module is unloaded. It is empty in 160 of 173 modules, missing in 10 (section 1.2) and not empty in three. Pack 800's Exit gives the hero item 9 (quantity 1, no popup) when the hero does not hold it; pack 934's Exit stops the on-screen timer, sets it to zero and deactivates it; pack 1248's Exit sets game-state flag 79 when its objects 0 and 2 are inactive and its object 1 is active. These effects must happen at every release of the world, including a release by a load into that buffer and the teardown on quit to the title (High). Unless a per-level spec says otherwise, the clean side may treat a missing Exit and an empty Exit the same (Medium).

**Per-tick script step (Confidence: High, except where marked).** Each game tick the script step of the current world runs after the camera, the world tick, the region passes, the projectiles, the world items, the selected hero and the petpet, and before the front end (SPEC-0026, main loop step 8.1.12). The script step of the loaded second world follows it and runs step 1 only. The script step is part of the world branch, so it does not run at all, step 1 included, while the game-state code is 9 (pause and every other freezing screen; SPEC-0026, Pause and freezing screens) or the world is disabled. Its steps, in order:
1. Special threads: the pending teleport prompt is started if one is pending (the job it starts is the transition-scene job, started from a pending world-swap value that is then cleared to -1; corrected 2026-11-09); then eleven special threads are each stepped once, in this slot order: the teleport-prompt thread (0), the enter-interior sequence (1), the leave-interior sequence (2), a further sequence thread (3), the Update thread (4) and six further sequence threads (5 to 10). Corrected 2026-11-09 (High): the Update thread is slot 4 and the sequence thread before it is slot 3; the earlier text had them the other way round. The step is always run first, whether or not the world is current. In the manager's layout the slots hold, in order, the swap/prompt thread, the world callback, two further hook threads, the Update thread, the hero-death callback, the activate-world callback, the on-enter callback, the use-item, pickup and buy callbacks. Stepping a thread means running it until it yields or ends; an ended thread is freed. For a world that is not current, only special threads 0, 1, 2, 6 and 7 are stepped, as chosen by an 11-byte table at data address 0x2EB3D8 (one byte per thread, 1 = stepped); its Update thread is not stepped (High).
2. Every trigger callback thread is stepped, and every trigger record's state decays (2 or 1 to 3 two frames after the last fire, 3 to 0 one frame later; Layout section 3).
3. The other callback threads are stepped, in this order (refined 2026-11-09, High): the load-override bookkeeping (below), breakable, teleport, the single item-proximity slot, the proximity entries, the item ask-about records, the game-flag callbacks, the level-flag callbacks, then the per-placed-object sub-lists (these are stepped and decay like trigger records). The activate-world callback is not stepped here; it is one of the special threads of step 1. The state decay runs only for trigger records and the per-placed-object sub-list records, not for the other callback kinds. A callback entry is released and its slot cleared in the same pass when its task step reports "not running" or the thread's first word is nonzero. Load-override bookkeeping (2026-11-09): before the trigger pass the manager remembers the world's load-override word (world +0x57C4); after the trigger pass it stores -1 when the word is unchanged and the new value when it changed.
4. Control threads are stepped in slot order. An entry whose thread has ended is freed in this same pass (section 4, Control threads).
5. If the world has an Update entry (the world pointer must also be non-null) and no Update thread is alive, a new Update thread is created; it first runs in step 1 of the next tick (confirmed 2026-11-09, High).

Steps 2 to 5 (and so new Update runs) are skipped while the hero-swap in-progress flag (game-global word 0x35C980 + 0x44, game-state code 5; SPEC-0053) is set, and for a world that is not current (High). This is the same flag that makes IsButtonPressed return false. It is not pause: under pause the whole script step is skipped (above). During a swap, the special threads of step 1 keep being stepped. Added 2026-11-09 (High): while the swap word is set on the current world, the only other work of the tick is that the world's load-override word is restored from the value saved by the last normal tick, when that saved value is not negative. Consequences: a new Update thread runs before that tick's trigger and control threads; a callback fired by the hero's trigger pass in a tick starts running in that same tick's step 2; a script waiting with Sleep(1) resumes in the next tick at the same position in this order.

**Threads of a world that is not current (from the code; not run in the original; High for the rule, Medium-Low for how visible it is).** A world's callback threads (trigger, teleport, activate-world, proximity, item, flag, script-group, breakable), its control threads and its Update thread are stepped only while that world is current and no hero swap is in progress. A thread waiting when its world stops being current keeps its place and its record. It resumes where it stopped when the world is current again, and its record cannot start a second thread meanwhile. Example: an exterior door's enter-interior callback waits 3.0 s after starting the entry, and the part of that wait left over is spent after the hero returns, during which that door cannot fire again (38 door callbacks, packs 842, 956, 1023, 1300, 1350, 1353, 1358, 1363; the size of the lockout depends on the load time, which is unmeasured). 28 exterior and 3 transition trigger callbacks mix waits with no hold on the hero (for example in packs 1090, 1091, 1171, 758 and 1344): if the hero leaves, their remaining animations and flag writes wait until the world is current again. 35 of 150 control-thread roots mix waits with hero-affecting calls; a build that runs them whichever world is current would run camera, dialogue and hero moves while the hero is in the interior. Special threads 0, 1, 2, 6 and 7 are stepped for a non-current world. Releasing a world ends its threads.

### 1.1 Module lifetime and family call (Confidence: High)

- **Fresh on every load.** A level module lives inside its world pack (type 37). Every load of a pack into a buffer rewinds that buffer's memory pool, reads the whole pack from the disc into it, relinks the module image in place from those bytes, runs its load hook and then calls the family Init. The module image is therefore the disc image when it is linked, and its data words hold their file values when Init runs, except for words that the module's load-time constructors write (the load hook runs them just before Init; Which level routines run in play).
- **Family call.** Once per load, after the chunk walk and the module link, the host calls Init(world) of the family that belongs to the buffer: E calls the Exterior entries, I the Interior entries and T the Transition entries. It stores the same family's Update and Exit on the world. The entries are bound by name to whatever loaded module exports them; when none does, they are null and nothing is called. The host calls the family entries from one place only: all 24 import relocations of the boot program for the nine family entry names sit in that load-completion step. A module whose family differs from its buffer's family therefore gets no Init, no Update and no Exit.
- **Interior modules in the exterior buffer.** Packs 1429, 1430 and 1431 export the Interior family and are streamed into E from pack 1427. They get no Init, Update or Exit there, so their scripts never run: no proximity talk, no buy hook, no teleport callback and no control threads. Reproduce: do not run these modules while they are in E.
- **Order.** Init runs in the load-completion step, after the chunks and the module link, before the post-load placement, before make-current and before the enter-interior job places the hero (section 6, Init order).
- **Lifetime.** The module and its data words, triggers, callbacks, threads and slots last until that buffer is next released. At release, Exit runs (when bound), the module is unlinked (its unload hook runs and its exported names are unbound) and the pool is rewound. A buffer is released only by a load into that buffer or by the buffer releases of SPEC-0006.
- **What survives, for an exterior in E.** Its data words survive (a) entering and leaving an interior (neither interior job releases E, and its Init does not run again on return; only the activate-world and on-enter callbacks run when E becomes current again), (b) swaps between E and T, (c) streaming steps whose target pack is already resident, and (d) a movie played while E is current with no return world (only T is reloaded).
- **What resets.** Any driver load of E (an exterior replacement load, a script reloading its own pack, the after-movie reload of E, loading a save), streaming to a different exterior (E released, the old pack gone) and a new game. Interiors and transitions are fresh on every entry: the enter-interior job releases T and I, the leave-interior job releases I and T, and streaming into T releases T.
- **Frozen while not current.** While E is resident but not current, only its special threads 0, 1, 2, 6 and 7 are stepped (section 1, Per-tick script step): its Update, triggers, other callbacks and control threads do not advance during an interior visit, and a wait they were in the middle of is finished only after the hero returns (section 1, Threads of a world that is not current).

### 2. Host API catalogue (Confidence: per row)

There are 388 rows, one per host overload, grouped into 23 categories. Each row gives the name, the arguments (section 2 of Layout, Name encoding), the effect on game state in plain words, and a confidence. Where a name appears more than once, each row is a separate overload, told apart by its argument list.

Confidence codes:
- **H**: the body was confirmed and the effect is established (confirmed against more than one call site or against the predicate it sets).
- **M**: the body was partly confirmed, or the effect is consistent with how the level calls it. Some details (a flag meaning, a return value) are not established.
- **L**: name and arguments only. The body was not confirmed. The effect is what the name says, and must not be relied on beyond that until the row is upgraded.
- **Medium-Low**: between M and L. Used where a row's host record was matched among overloads by table order and the argument list is not confirmed by the host name (no row is in this case now; Open question 23). It is also used for the argument-ignoring quirks in Edge cases.

Numbers run 1 to 388 in category order. Rows 286 to 294 are in Category 15. Rows 295 to 388 are in categories 16 to 23.

**Note on entity ids (Confidence: High).** An entity argument is the descriptor id that IsAI_Dead and the walk calls use. It is looked up in the current world (or in W where a row says so): the lookup finds the spawn record, then its region (the pack's embedded region, or a created slot whose record is in use), and in that region's character list returns the live character with that descriptor id. When nothing matches, a setter does nothing; each query row gives its own missing-entity value. This per-region lookup is only for resolving the script's entity argument: the steering look-ahead of a walking character (SPEC-0029 Behavior 2a, source 4) is not limited to its region and takes every character of every loaded region (R-0279). Base state 33 (script hold) is described in section 4 (Script control and the entity's AI).

**Category 1: Animated objects, locators and effects**

| No. | Name | Arguments | Effect in plain words | Conf. |
| --- | --- | --- | --- | --- |
| 1 | InitObjOn | W, int, bool | Prepares a numbered animation object for this world. The object is marked active and set to the stopped state at time zero. If the bool is false, the object stays at time zero (its start pose). If the bool is true, it is moved to the end of its animation and enters the finished state, so IsObjAnimFinished is true. It also sets the object's drawn and draw-gate bytes, and sets its last-update change (+0x150) to (0, 0, 0, 1). | M-H |
| 2 | PlayAnim | int object, WAIT, float speed | Plays the object's animation once, in play-once state, at the given speed factor (1.0 normal, 0.5 half, 2.0 double). A negative speed first moves the object to the end of its animation and then plays it backwards (27 sites pass -1.0). The call also clears the object's additional offset and rotation (rows 16, 19, 23 and 27) and its motion accumulator (the root-offset reset, which also receives the speed value; confirmed 2026-11-09). With WAIT nonzero the caller yields each tick while the object stays in play-once state; the state is tested before the first yield, and there is no unconditional first yield (confirmed 2026-11-09). An object index outside the world's animated-object list does nothing. | H |
| 3 | ActivateObj | int object, bool end | Same as InitObjOn (row 1) on the current world: marks the object active; false puts it at time zero (stopped), true at the end of its animation (finished). Pack 1096 calls ActivateObj(14, false) at the fight start; it leaves the object at time zero and does not hide it. | H |
| 4 | DeactivateObj | int object | For a valid object of the current world: animation state 4 (finished), drawn byte 0, draw-gate byte 1, mode byte 0 (additional offset and rotation off), colour bytes reset to 128, 128, 128, 255 (the data word at 0x2EB1B8; the disc value is 255 in all four, so a shown-again object is drawn with colour scale 1 instead of about 2, roughly half as bright: SPEC-0028, Run-time object changes, Looks), and the cull value of every part and frame record set to -200.0 (SPEC-0028). An index outside the list does nothing. | H |
| 5 | StopEffect | int, W | Stops effect slot: clears its active flag. | H |
| 6 | StartEffect | int, W, bool b | Uses W, or the current world when W is 0. For a valid effect slot, starts it. With b set, a copy of the camera object's 4x4 block is passed as the start frame; with b clear, no start frame is passed. Refined 2026-11-09 (High): the copied block is the camera object's first four 16-byte rows (+0x00 to +0x30); the placed-effect records are 0x90 bytes each, starting at block offset +0x10; a valid effect index is 0 up to the block's count. | H |
| 7 | IsObjAnimating | int | True when the object is in loop, play-once, or a related animating state (codes in Edge cases). | H |
| 8 | IsObjActive | int object | True when the animated object exists and its drawn byte (+0x1B6) is set. InitObjOn and ActivateObj set the byte; DeactivateObj clears it. | H |
| 9 | LoopAnim | int object, float speed, W | Loops the object's animation at the given speed factor (loop state), clearing its additional offset and rotation. Speeds on disc: 1.0 (36 sites), 0.2, 0.1, 0.5, 0.4, 0.0 and -1.0. | H |
| 10 | SetObjToTime | int, float | Moves the object's time to the given seconds, clamped to 0 .. duration. At or past the end it ends in mode 8 (finished, time = duration). Below the end it always ends in mode 10 (stopped at that time; the next tick does not advance), whatever the earlier mode (SPEC-0005, Clip time and mode). Added 2026-11-09 (High): the call also resets the object's translation change (+0x150) to (0, 0, 0, 1), as the switch-on helper does, and, when the current world has a script manager, runs the sound-bank window cues over the span from the old time to the new time. | H |
| 11 | StopAnim | int | For a valid object of the current world (index 0 to the animated-object count - 1; otherwise nothing): the state becomes 10 (stopped at its time) unless it is 8 (finished), which is kept. The last-update translation change (+0x150, SPEC-0028) is set to (0, 0, 0, 1), so no root motion is carried into the next tick. The time and the play speed are kept. | H |
| 12 | MoveEffectToAnimatedLocator | int object h, int effect e, int locator k, bool face | For h in the animated-object list: face false moves effect e to the current position of h's locator k (formula A, Note on animated locators), in point form; face true also passes the locator direction rotated by h's current rotation (formula B), in direction form (Note on MoveEffect). There is no check of k: an invalid k or a missing locator list gives the zero point. Disc: 15 sites. | H |
| 13 | IsObjAnimFinished | int | True when the state is finished (codes in Edge cases). | H |
| 14 | SetSpeedFactor | int, float | For a valid object of the current world: stores the float as the object's play speed (+0xB8, SPEC-0028), the factor of its per-tick time step (time += dt x speed). A negative factor plays backward and 0 holds the time; the state is not changed. A missing object does nothing. | H |
| 15 | GetObjMaxTime | int | Returns the length of the object's clip in seconds: (F - 1) x 0.06666667 in single precision, with F the key count at +0x1E4 (SPEC-0005, Clip length unit). Returns -1.0 when the object is missing. | H |
| 16 | SetAnimObjPositionOffset | int, float x, float y, float z | Stores (x, y, z) as the object's additional position offset and marks the offset in use. If the object is stopped or finished, its pose is evaluated again at once. | H |
| 17 | GetLocatorPositionPointer | int k | Returns a pointer to the +0x30 point of locator k of the current world's locator table (section 3), or 0 when k is out of range. | H |
| 18 | ActivateMovableBlock | int block, bool b, W | Uses W, or the current world when W is 0. For a valid block index, writes b to the block's active word (SPEC-0048, Movable blocks, Activate). | H |
| 19 | SetAnimObjAdditionRotLocatorForward | int object, int locator, float degrees | Sets the object's additional rotation to a rotation by the given angle in degrees about the locator's direction vector (locator record +0x40; locator table in Behavior section 3). The quaternion is built as in row 23. A locator index outside the locator table does nothing. | H |
| 20 | GetObjCurrentTime | int | Returns the object's time in seconds (+0x1B0, SPEC-0028). Returns -1.0 when the object is missing. | H |
| 21 | SetAnimObjColorMultiplier | int object, uchar r, uchar g, uchar b, uchar a | For a valid object of the current world, sets its four colour bytes (object +0x1A4 to +0x1A7) to r, g, b and a, in that order. The draw multiplies the object by r/256, g/256 and b/256 and its alpha by (a + 1)/256 (SPEC-0028, Draw gate), so 128 halves a channel. An index outside the object list, or a world with no list, does nothing. Disc: 5 live sites in modules 1174 and 1300; every one passes 128 for r, g and b and a varying a, so these calls fade an object in or out. | H |
| 22 | MoveMovableBlockToLocator | int block, int locator | Both indices are checked against the current world. As SPEC-0048 (Movable blocks, Move to locator). | H |
| 23 | SetAnimObjAdditionRotY | int object, float degrees | Sets the object's additional rotation to a rotation about the vertical axis by the angle in degrees. The rotation is the unit quaternion (axis x sin(a/2), cos(a/2)) with a = degrees x pi/180 (the factor is the data word at 0x2DB480, 0.0174533). It replaces any earlier additional rotation; it does not add to it. | H |
| 24 | ResetAnimObjOffsets | int | Clears both the additional position offset and the additional rotation of the object. | H |
| 25 | GetObjCurrentFrame | int | For object t (time in seconds), the frame index is k = trunc(t x 14.999999) and the fraction is (t - k x 0.06666667) x 14.999999, both single precision; returns k + fraction as a float, about t x 15. Returns -1.0 when the index does not name an object of the current world. | H |
| 26 | GetAnimatedObjPositionPointer | int object | Returns a pointer to the object's reference centre (+0x180, SPEC-0028 section 7.1), not its base position. The pointer stays valid and the point is updated every tick. Returns 0 when the index is out of range. | H |
| 27 | SetAnimObjAdditionRotZ | int object, float degrees | As row 23, about the depth axis. | H |
| 28 | MoveEffect | int effect, int locator, bool face | Moves the effect to the locator's position (locator record +0x30). When face is set, the locator's direction (record +0x40) is also passed, so the effect takes the direction form (Note on MoveEffect); otherwise the point form. A locator outside the locator table does nothing. Disc: 3 sites. | H |
| 29 | SetObjToMayaFrame | int, float frame | Same as SetObjToTime (row 10) with time = frame / 30: the authoring frame rate is 30 frames per second (the data word at 0x31F5F0 holds 1/30). | H |
| 30 | AttachEffectToMovableBlock | W, int, int, float, float, float | Attaches a placed effect to a movable block at an offset: AttachEffectToMovableBlock(world, block, effect, a, b, c). The block index is checked against the movable-block count and the effect index against the world's placed-effect count; an index out of range does nothing. The offset vector is built as (b, c, a): the third float is the x offset, the first float is the z offset (SPEC-0048, Movable blocks, Attach effect). Example: block 2, effect 0, a = 2.0, b = 5.0, c = 1.5 gives the offset (5.0, 1.5, 2.0). | H |
| 31 | GetAnimObjWorldPos | int, float outputs | Writes the object's world-space position into float outputs. | L |
| 32 | MoveEffect | int effect, float x, float y, float z | Builds the point (x, y, z) and runs the shared MoveEffect routine in its point form (face off): the effect's position becomes that point. The effect index is checked first; an index outside the effect table writes nothing (Note on MoveEffect). Disc: 2 live sites, both in module 1398. | H |

**Note on MoveEffect (rows 32 and 324; Confidence: Medium).** One host routine sits behind the MoveEffect name and has two forms. The point form sets the effect's position to the given point. The direction form takes a direction, normalises it, and builds an orthonormal frame from it with cross products, then writes that frame and the position into the effect. The frame needs a helper axis that is not parallel to the direction. When the direction is nearly vertical, the routine uses one of two fixed unit vectors as the helper axis in place of the default one. The nearly-vertical test is two-step: one component of the direction is first compared (absolute value) with 0.1, and the fix applies only when it is below 0.1; the absolute values of two components are then compared to choose which of the two fixed vectors is used (Medium: which components are meant is not settled and only changes the roll of the effect's frame; Default if unresolved (rewrite choice): switch to the fixed helper vector when both |x| and |z| are below 0.1, the only case where the default helper is nearly parallel). The default helper axis is the world up vector (0, 1, 0); the two replacement vectors are engine constants that the startup code fills in at run time, so they cannot be read from the disc image (Medium). The routine's debug log line runs only when the debug flag is on, which never happens in play (section 8). Row 32 uses the point form; row 324 and row 28 with face set use the direction form; row 28 with face clear uses the point form (High for the forms; Open question 40 keeps the helper-axis details).

**Note on animated locators (rows 12, 124, 128, 131 and 174; Confidence: High).** An animated object h can carry an inline locator list (its type-11 header, SPEC-0028). The current world position of h's locator record k is computed from: L = the record's +0x30 point; T = h's rest translation (+0xF0); R = the 3x3 matrix whose columns are h's vectors at +0x00, +0x10 and +0x20; r = h's root offset (+0x80).
- Formula A (position): d = L - T with d.w = 0; P = T + r + R d.
- Formula B: the position as in A without forcing d.w to 0 (on disc the locator's w is its radius, 0.625 in the sampled packs, and T's w is 1, so d.w is -0.375; the result is still the same because the fourth row of the host's pose matrix is always (0, 0, 0, 1), so d.w has no effect on x, y, z; R-0270). The follow link of row 124 uses this form with the locator point added to the root offset: base = L + r + (R d - d) with R the same 3x3 matrix, plus a direction: h's current rotation quaternion (+0x160) applied to the record's +0x40 vector.
- A locator link (rows 124 and 130 with follow or link set) makes the object update recompute the base from the linked record every tick (SPEC-0028 section 7.1, step 5); the other form writes the rest translation once.
- Synthetic check: T = (0, 0, 0), r = (0, 1, 0), R = identity, L = (2, 0, 0) gives P = (2, 1, 0); with R a 90-degree turn about y that maps (1, 0, 0) to (0, 0, -1), P = (0, 1, -2).

**Category 2: Cameras**

| No. | Name | Arguments | Effect in plain words | Conf. |
| --- | --- | --- | --- | --- |
| 33 | SetToNormalCamera | int preset, bool release | Identical to ReturnToNormalCamera (row 35). One disc site passes (10, false). | H |
| 34 | PlayCamera | int track, WAIT, CAMW end mode, float speed | Plays camera track `track` of the world with no target object. The CAMW value is the play's end mode (Note on camera calls); it is not an entity index. The float is the playback speed factor: the track time advances by (tick step x factor) per update, so a track of length L seconds lasts L / factor seconds. Factors on disc: 1.0 at 425 sites, 0.5 at 17, others from 0.3 to 2.0; 0.0 at 3 sites freezes the play. A track outside the world's list does nothing and does not wait. With WAIT nonzero, and when the track has samples, the caller yields each tick until the play is ended or idle; it checks before the first yield. | H |
| 35 | ReturnToNormalCamera | int preset, bool release | Stops any running play and returns the camera to the normal follow mode (SPEC-0027). When the preset index is inside the world's locator table, the camera is placed at that locator's position (record +0x30); -1 (243 sites) means no preset. When release is set, the active hero is put in state 1 (free); 263 of 267 sites pass true. | H |
| 36 | StartCameraShakeMode | shake preset, float duration, float strength | Starts a shake of the given preset (0 to 5; other values do nothing) lasting `duration` seconds, with amplitude strength x 0.3 (the factor 0.3 is the data word at 0x2DB2F0); the preset supplies the other shake values (SPEC-0027, Camera shake). Every preset, including 0, receives the shake record. Disc sites (preset, duration, strength), 20 sites in 13 distinct modules: preset 0 at 16 sites, preset 1 at 2, preset 2 at 2. Preset 0: (0.5, 1.0) x4, (1.0, 1.0) x2, (1.0, 3.0) x2, and once each (2.0, 3.0), (0.6, 0.5), (0.3, 0.3), (0.5, 0.3), (2.0, 1.0), (3.0, 1.0), (1.0, 0.3), (0.7, 0.3). Preset 1: (5.0, 1.0) x2. Preset 2: (1.0, 1.0) and (3.0, 1.5) once each. The (0, 0.3, 0.3) site is in module 1080, where the strength passed is the same value as the duration. Resource 970 is a byte-identical copy of module 956 and adds one more (0, 0.6, 0.5) when copies are counted. | H |
| 37 | PlayTrackHeroCamera | int track, WAIT, CAMW end mode, float speed | As row 34, with the active hero as the target. The CAMW value is read as the end mode. Behaves exactly like row 39. | H |
| 38 | PlayLookAtAnimObjCamera | int track, int object, WAIT, CAMW end mode, float speed | As row 34, with animated object `object` as the target, using the object's look-at anchor (object offset 0x190). If the object index is outside the list or the object is missing, the play runs with no target. | H |
| 39 | PlayLookAtPlayerCamera | int track, WAIT, CAMW end mode, float speed | Same as row 37: the active hero is the target. | H |
| 40 | IsAnimatedCameraActiveAndFinished | int track | True when both hold: IsAnimatedCameraActive (row 294) is true for the track (a play is in progress or held, the index is inside the current world's camera track table, and that entry is the track being played), and the play has ended (run state 2, Note on camera calls). End modes 0 and 2 make a play idle in the same update that ends it, so the call is true only for a play started with end mode 1, from the update its track ends until ReturnToNormalCamera, SetToNormalCamera or another play. Disc: 3 live sites (modules 757, 864 and 985). | H |
| 41 | StartCameraShake | float T, float A, float S, float p4, float R, float phi, float Z | Starts a camera shake from seven floats in this order: duration T, amplitude A, wave rate S, attack time p4, fade-out time R, angle phi and aim factor Z. All seven go unchanged into the one shake record (SPEC-0027, Camera shake): T is the end time, A the start amplitude, S the wave rate of every key, the key times are 0, p4, T - R and T, phi feeds the two cosine terms, and Z is the aim factor. The start replaces any running shake. Disc: 3 live sites in modules 864, 1091 and 1391. | H |
| 42 | PlayTrackAICamera | int track, int entity, WAIT, CAMW end mode, float speed | As row 34, with AI entity `entity` (looked up by id) as the target; no target when the entity is missing. | H |
| 43 | SetToNormalCamera | float distance, float angle, bool release | Stops any play and returns the camera to the normal mode, then places the eye on the orbit of SPEC-0027 (Return to normal with an angle), with the first float as the orbit distance and the second as the angle in radians. Release as in row 35. Disc: 132 sites in 99 modules; distance 5.0 (55), 10.0 (36), 15.0 (22), 3.0 (10), 0.3 and 6.0 (2 each), others once; angle 0.0 (105), 3.14 (16), 3.14159 (6); one site in module 1077 passes 180.0, which the host reads as radians (an authoring slip; reproduce it as radians); release true at 129 sites. Added 2026-11-09 (High): the eye is hero position + unit(forward x cos(angle) + right x sin(angle)) x distance, with forward the hero's body-forward and right the hero's right vector; the unit vector's fourth lane holds its length, so the eye's w is the hero's w plus that length x distance (whether the pose setter uses that lane is unknown). | H |
| 44 | SetHeroSceneCamera | float a, float b | Sets a scene camera parameter for the hero (two floats). Refined 2026-11-09 (High for the argument plumbing): the two floats reach the camera's orbit setter swapped (b first, then a), with height 1.5, three fractions of 0.01 and a pivot step of 0, so a is the yaw offset and b the radius if the setter's first parameter is the radius (SPEC-0027). | M-H |

**Note on camera calls (rows 33 to 43 and 318; Confidence: High).** The shake record fields and the six shake presets are described in SPEC-0027 (Camera shake); this list gives the call shapes and the play rules below.

- **One shared play core.** The six camera-play calls (rows 34, 37, 38, 39, 42 and 318) all start a play through one shared core. Their arguments are always: track, WAIT, CAMW end mode and speed factor, plus a target argument for rows 38, 42 and 318. The target is chosen by the call, not by an argument value: none (row 34); the active hero (rows 37 and 39); an animated object by its index (row 38 with the object's look-at anchor at object offset 0x190, row 318 with its track anchor at object offset 0x180); an AI entity by its id (row 42). Track and look-at forms differ only for the animated-object target; the hero forms (rows 37 and 39) behave identically. This matches SPEC-0027 (Scripted cameras, Starting a play).
- **Play state.** A play stores its kind (no target or target), its track, its target anchor, its track time (starts at 0), its run state (1 = playing), its speed factor and its end mode. Each update: track time += tick step x speed factor; the track is sampled at that time; when there is no sample, the run state becomes 2 (ended). While the play is running, a hero that is in none of the scripted states (0x21, 0x30, 0x32, 0x33, 0x34, 0x35, 0x37) and not in state 0x12, 0x13 or 0x19 is put in state 0x21 and made to play its animation 0.
- **End modes (the CAMW value).** When the run state is 2: end mode 1 does nothing, so the play stays ended and holds its last pose until ReturnToNormalCamera, SetToNormalCamera or another play (481 sites use mode 1, followed by one of those calls); end mode 0 sets the hero to state 1 if it is not in a scripted state, and the play becomes idle (0); end mode 2 sets the hero to state 1 whatever its state, and the play becomes idle.
- **Wait loop.** With WAIT nonzero, a play call yields each tick while the play is neither ended nor idle.
- **Shake.** StartCameraShakeMode (row 36) takes (preset, duration in seconds, strength) with amplitude 0.3 x strength; StartCameraShake (row 41) takes its seven floats directly.
- **Return to normal.** ReturnToNormalCamera (row 35) and SetToNormalCamera (int, bool) (row 33) have identical behavior; SetToNormalCamera (float, float, bool) (row 43) takes an orbit distance and an angle in radians.

**Category 3: Combat and boss hooks**

| No. | Name | Arguments | Effect in plain words | Conf. |
| --- | --- | --- | --- | --- |
| 45 | IsAI_Dead | int id | Passes the id to a world routine and returns its answer unchanged (nonzero = dead); 0 when the id is not a character descriptor of the world. The world routine answers as follows: for a summon-reserved record (SPEC-0029 Behavior 31), by the live state of the character bound to it; for any other record, true when its bound character is in state 16 with a countdown above 0, and otherwise by its kill bit (SPEC-0006); a bound living character does not suppress the kill bit (corrected 2026-11-08, enemy code pass). All 56 module sites test the result. | H (return); High (world routine for summon-reserved records; Medium-High for the rest) |
| 46 | Break | int | Breaks a breakable object of the current world. The index must be 0 to the count - 1 of the world's breakable list; otherwise nothing breaks. The three break source words are copied first on every call, even with a bad index (Behavior section 2.2). It acts only when the object is intact and breakable (state 1): it becomes broken, spawns its drops and break pieces, and plays its break effect and sound. In any other state it does nothing. | H |
| 47 | SetBreakableActive | int, bool, W | Enables or disables a breakable object (Behavior section 2.2). True sets it to intact and breakable (state 1) and sets its sub-record; false sets it to disabled (state 0). | H |
| 48 | IsBreakableIntact | int | Query: true when the breakable is in state 1 or 3 (intact). | H |
| 49 | SetAI_HitPointsPercentage | int, float | Sets the entity's hit points to the given fraction of its maximum (fraction times maximum, truncated to a whole number). Does nothing when the entity is missing. | H |
| 50 | GetAI_HitPoints | int | Entity lookup in the current world (Note on entity ids). Returns the entity's current hit points, an integer (the numerator of row 87). Returns -1 when the entity is missing. | H |
| 51 | TriggerActStart | int | Act-start setup by stage number (SPEC-0053, section 3.7). Stage 1 sets the swap mode to 0 (locked to Tor) and switches to Tor by a normal switch request. Stage 2 sets mode 1 (locked to Roberta), selects Roberta directly, stores progress value 220 and heals Roberta only. Stage 3 sets mode 2, selects Tor directly, stores 248 and heals both heroes. Stage 4 sets mode 2, stores 332 and heals both. Other values set mode 2 and heal both heroes, and tear nothing down. No act-state word is written here. Added 2026-11-09 (High): stage 2's fresh party kit is 3 of item 0x96, 3 of item 0x95, 1 of item 0x7F and 1150 of item 0x87 (Neopoints) in the party inventory; stage 3 merges the stash into the party inventory, reloads hero slot 0 and selects Tor. Stages 3 and 4 tear down combat, effects and the petpet before the heal; stages 1 and 2 do so after it. Stages 2 and 3 also re-spawn both heroes at the current world's spawn vector (normally entry record 0; SPEC-0040, Behavior 1B); stages 1 and 4 place nobody (Medium). | H (flow), M (names) |
| 52 | GetHitPointsPercentage | hero selector | Query: returns the hero's current hit points divided by its maximum, as a fraction (0 to 1 in normal play; not multiplied by 100). Hero 0 is Tor and hero 1 is Roberta (SPEC-0036). A maximum of 0 is not handled. | H |
| 53 | SetHitPointsPercentage | hero selector, float | Sets the hero's hit points to the maximum multiplied by the given fraction, truncated to a whole number, then applies the set rule of SPEC-0036 (Hit points). A fraction above 1 gives the maximum. A negative fraction is not confirmed (Open questions). | H |
| 54 | GetHitPoints | hero selector | Query: returns the hero's current hit points as an integer. The value can be negative after a negative add (SPEC-0036). | H |
| 55 | MakeBreakable | int, bool, W | Sets whether an object can be broken (Behavior section 2.2). False moves an intact breakable object to intact and not breakable; true moves it back. Other states are unchanged. | H |
| 56 | IsActStartDone | none | Always true. The act start check does not look at any state. | H |
| 57 | IsAI_Active | int id | Returns the character's object when the character with that descriptor id is currently spawned in its slot, else 0 (use it as a boolean). | H |
| 58 | OverRideSecondaryHeight | float | Overrides a secondary height value. | L |
| 59 | IsAI_Agro | int | Entity lookup in the current world. True when the entity's AI mode word (+0xA34) is 19, the alert mode whose entry posts the group alert (SPEC-0029, class requests). False in any other mode and when the entity is missing. | H |
| 60 | IsAI_Dead | W, int slot, int n | Dead test of character n of slot `slot`: 0 when the slot is outside the created slots or holds no loaded region; otherwise the same dead test as row 45, returned unchanged. | H |
| 61 | InitMiamouseBehavior | W, int m1, int m2 | Sets the Miamouse field mesh id to m1 and the cage mesh id to m2, turns catching on and sets the cage count to 0. A Miamouse released by the hero gets nav mesh m2 when its own held position (the hero's attach point) is on mesh m2, else m1; the hero's position is not tested. It is placed at its held position + 1.0 unit along the hero's forward direction (SPEC-0029 Behavior 28). Disc: module 1163 passes (0, 1), module 1398 passes (15, 16). | H |
| 62 | SetMaxActiveAI | int | Sets the active cap of the pack's embedded character region (the most characters it keeps bound at once by streaming, SPEC-0006), clamped to 15. The default cap is 6. On disc only packs 1176 and 1246 call it, both with 12. Slots are not affected (their cap is set by SlotLoad and SlotReset). | H |
| 63 | ReacquireMovableBlockHeightPatch | int block, float, W | W 0 means the current world. For a block index 0 to the world's movable-block count - 1 (SPEC-0018, movable-block list), the block is centred on its own current position and placed again (SPEC-0048, Reacquire height patch): its ground height is probed again, its home becomes its position and its displacement from home becomes zero. An index out of range does nothing. The float is used: it is the ceiling offset of the ground probe, so the probe starts at the block's current height plus the float (SPEC-0048, Reacquire height patch). Disc values: 3.0 at the three sites of pack 1324, 1.0 at the three trigger sites of pack 1397 and 500.0 at its per-tick site. Returns nothing. | H |
| 64 | TriggerBaggus | int i | Breaks Baggus record i in the current world's runtime table; an absent table or out-of-range index does nothing. The runtime table is built from kind-3 entries in the variable list of chunk type 40. That chunk stores an entry count at payload offset +0x594, a payload-relative list offset at +0x598, and 0x44-byte entries. The runtime header stores its record count at +0x10 and record base at +0x14; records are 0x4160 bytes. Trigger-used record fields are the 4x4 transform at +0x100, position at +0x130 (the transform's translation), state words at +0x3E00 and +0x3E0C, and effect group at +0x3E20. The list has one shared model pointer. Kind-3 setup selects it from valid handles named by the +0x2C/+0x30 candidate list; indices are checked against the 32-entry model-handle array in chunk type 29, and the last valid candidate processed remains selected. In pack 735, entry 20 selects candidate 3. Its separate +0x1C/+0x20 list contains 32 values (66–97), which use a different 99-entry reference table and do not select the shared model. In the type-40 order, entries 3–7 each reference one type-11 object index (16–20); their callbacks initialize the first unused runtime records in order, so source entries 3–7 map to runtime records 0–4. Entry 20 has no type-11 object index and its callback does not initialize a record; the sixth record has no source placement assigned by this callback path. In pack 735, trigger 21's helper passes indices 3 through 7: indices 3 and 4 address the records sourced by entries 6 and 7, index 5 addresses the sixth record without a source placement from this path, and indices 6 and 7 are out of range. Each source object contributes two four-float placement vectors from its 0x50-byte type-11 record. The first vector's X and Z are used for a world-height query whose upper Y bound is its source Y plus 4.0; the returned height replaces its Y while its other components are preserved. The adjusted first vector and original second vector are submitted to pooled-group initialization; the initializer uses the first as translation and does not read the second argument. Pack 735 source records 16–20 have first vectors (3277.117, 113.739, -514.102, 0.625), (3262.460, 115.113, -531.386, 0.625), (3260.012, 114.368, -553.355, 0.625), (3281.036, 115.415, -553.103, 0.625), and (3296.854, 114.193, -537.636, 0.625); each second vector is (0, 0, 1, 0). For source type-11 index 16, the type-19 cell at (181, 48) has 11 ordered candidates, all in top-level type-5 data with owner and gate mask zero. The documented list test and sampler select top-level type-5 entry 159 at approximately 115.466, below the 117.73947 ceiling; the preceding higher accepted sample is 112.521 from entry 146. For source type-11 index 17, the type-19 cell at (180, 48) has 12 ordered candidates, all in top-level type-5 data with owner and gate mask zero. The documented list test and sampler select top-level type-5 entry 148 at approximately 113.766, below the 119.11268 ceiling. For source type-11 index 18, the type-19 cell at (180, 47) has 12 ordered candidates, all in top-level type-5 data with owner and gate mask zero. The documented list test and sampler select type-5 entry 129 at approximately 114.794, below the 118.36850 ceiling. For source type-11 index 19, the type-19 cell at (181, 47) has 11 ordered candidates, all in top-level type-5 data with owner and gate mask zero. The documented list test and sampler select type-5 entry 148 at approximately 114.538, below the 119.41533 ceiling. For source type-11 index 20, the type-19 cell at (181, 47) has 11 ordered candidates, all in top-level type-5 data with owner and gate mask zero. The documented list test and sampler select type-5 entry 146 at approximately 112.521, below the 118.19288 ceiling. Source object index 20 maps to runtime Baggus record 4; the sixth record is unplaced through this callback path, as noted above. The callback passes the ground-adjusted first vector and the second vector into pooled-group initialization. Each initialized child takes its 3x3 basis from its selected template; its translation is the first vector with Y replaced by the ground result and W set to 1.0, stored at +0x130. Source W=0.625 is not retained as position scale, and the second vector (0, 0, 1, 0) is not read by this initializer. Numeric orientation and scale therefore come from the selected child template. In pack 735, type-40 entry 20 supplies ordered type-29 first-table indices 66–97 for the 32 child slots; each referenced template stores the identity 4x4 basis, so every child uses unit scale and no rotation before its translation is replaced with the source position. The templates are selected before the pooled groups are initialized. (The shared mesh may still contain its own internal transforms.) The trigger starts a burst 2.0 units above the record position and resets the effect group and both state words regardless of the shared-model pointer. Only when that pointer is present does it move the shared model to the record transform, play cue 178 and apply the breakable-object path. It then spawns one item 145 at the transform through the item-drop spawner (SPEC-0021). | M-H |
| 65 | SetWhinnyPenmesh | W, int inner, int entrance, int locator k | Sets up one pen: pen count 1; pen 0 gets inner-area id inner, entrance id entrance, pen point = the +0x30 point of locator k (all four lanes), and occupied 0. There is no check of k. Disc: module 909's Init passes (0, 5, 8). | H |
| 66 | IsAI_Active | W, int slot, int n | As row 57 for character n of slot `slot` (the slot is checked against the slot capacity). | H |
| 67 | IsDoingLungeAttack | none | True when the selected hero is Tor (hero index +0x14C = 0) and Tor's melee state word is 9 (lunge). Always false while Roberta is selected, and false when there is no hero. | H |
| 68 | IsDoingPoweredUpMoteAttack | none | True when the selected hero exists, its hero index word is 0 (Tor) and its attack sub-state word equals 10. Sub-state 10 is the power step of Tor's attack step, the charged Mote swing (SPEC-0036, State 13, Tor's arm table). Always false while Roberta is selected. Only pack 1056 (the arena lessons) imports it. Test vector: Tor selected in sub-state 10: true; Tor in sub-state 2 or 9: false; Roberta in sub-state 10: false. | M-H |
| 69 | IsDoingSpinAttack | none | True when the selected hero is Tor and Tor's melee state word is 7 or 8 (spin). Always false while Roberta is selected, and false when there is no hero. | H |
| 70 | IsThereADeflectedMissile | none | True when one of the first ten projectile slots holds a deflected missile. False when none does. | M |
| 71 | IsDoingChainAttack | none | True when the selected hero is Tor and Tor's melee state word is 4 (chain). Always false while Roberta is selected, and false when there is no hero. | H |
| 72 | IsThatAFireMote | int item id | True when the id is a Fire-element Mote: 160 (Fire Mote) or 161 (Lava Mote). The test is the Mote element lookup of ids 160 to 173, where element 1 is Fire (SPEC-0021); any other id, -1 included, gives false. Disc: 4 live sites, all in module 1056, each passing the result of GetHeroAttackAlignment (row 176) or GetHeroShieldAlignment (row 181). | H |
| 73 | IsDoingMeleeAttack | int | True when n is 1 to 13, the selected hero is Tor (hero index 0), a hero class query (method table slot +0xE4) returns nonzero, and Tor's melee state word equals n. Always false while Roberta is selected. | M-H (the role of the class query Medium) |
| 74 | IsDoingAnyMeleeAttack | none | True when the selected hero is Tor and its melee state word is 2, 3, 5, 6, 10, 11 or 12, or when row 67, 69 or 71 is true. So it is always false while Roberta is selected. | H |
| 75 | IsOnLastMeleeJab | none | True when the selected hero is Tor (hero index 0), the hero class query (method table slot +0xE4) returns nonzero, Tor's melee state word is 6 (the jab chain) and the jab counter beside it (+0xF94) is above 1. False while Roberta is selected or when there is no hero. There is no lock test. | M-H (the role of the class query Medium) |
| 76 | LaunchFireMote | int, int, float, bool | Launches a fire-mote projectile from a start locator. The float sets the flight speed. The fourth argument selects the target: true aims at the active hero, false uses the end locator. If the end locator is missing, hero aim is forced. When the start locator is missing it does nothing and reports an error. Corrected 2026-11-09 (High): the two ints are locator ids, matched against the 16-bit id at record offset +0x1E (the first record with that id), not list positions. The aim point is the end locator's position, or the hero's position with y raised by 1.5. The direction is the unit vector from start to aim; the float is the distance moved per tick (velocity = direction x speed); the lifetime is trunc(distance / speed) + 20 ticks. The projectile uses setup kind 0x17, effect ids 0x210 and 0x3DF and the floats 3.0 and 2.0, and is submitted into a free projectile slot; with none free nothing launches. | H |
| 77 | IsBreakableInvulnerable | int | Query: true when the breakable is not in state 1, so it cannot be damaged (Behavior section 2.2). | H |
| 78 | Draik_SetBreakableIndices | int x9 | Stores nine breakable ids in the order: left leg, right leg, tail, chest, left arm, right arm, left wing, right wing, head. The Draik index queries read them back. | H |
| 79 | MiamiceCatchDisable | none | Turns Miamouse catching off; the cage count is kept. | H |
| 80 | DFS_Act1_SetIndices | int x5 | Stores the first four integers as trigger indices for the act-1 sequence. The fifth integer is dropped: it is not stored. | H |
| 81 | DFS_Act1_SetTriggerData | int i, bool v, T(bool) update | The act-1 sequence keeps a table of up to 16 (index, value) pairs and a pair count (0 at start-up). With update false: when the count is below 16, the pair (i, v) is appended and the call returns its position (the old count); with 16 pairs nothing is stored and it returns -1. With update true: the first pair whose index is i gets value v and its position is returned, or -1 when no pair has index i. Appending an index that is already present adds a second pair, which reads and updates never reach. | H |
| 82 | DFS_Act1_ResetTriggers | none | Sets the act-1 pair count to 0 (row 81). The pairs are not erased, but no read or update reaches them. | H |
| 83 | ChangeMortogTimes | float hmin, float hmax | Sets the Mortog hold minimum and maximum (seconds; SPEC-0029 Behavior 3). Module 1363's thread counts each rise of NumMortogsOnTarget and adds 5 s to both on odd-numbered rises only (first rise 30 and 35, third rise 35 and 40, and so on; even-numbered rises leave them unchanged), and sets (0, 0) when NumMortogsOnTarget returns 4. With a maximum of 0 the holding Mortogs never leave (SPEC-0029). When a hold ends (maximum not 0), the new move goal is k × unit(P − A), where P is the Mortog's position, A the zone centre and k a whole-number distance drawn uniformly from 4 to 8; no position is added, so the goal lies within 8 units of the world origin. The zone rule usually replaces that goal on the next wander tick. | H |
| 84 | InitMortogBehavior | W, int t, int i, int j, float hmin, float hmax | Needs 0 <= i, j < the world's locator count; otherwise it only shows a debug message. Sets the Mortog zone centre A to the +0x30 point of locator i and the squared zone radius L = (xi - xj)^2 + (zi - zj)^2, a horizontal distance (y is not used); hold minimum = hmin and hold maximum = hmax (the SPEC-0029 Behavior 3 words); zone count 0; and the target index t when t >= 0. Disc: module 1363's Init passes (W, 3, 6, 7, 25.0, 30.0). | H |
| 85 | NumMortogsOnTarget | none | Returns the Mortog zone count (SPEC-0029 Behavior 3). | H |
| 86 | SetAI_HitPoints | int, int | Sets an AI entity's hit points to an absolute value (the entity is looked up by id in the current world; a missing entity does nothing). A value above the maximum sets the hit points to the maximum and runs the heal notification, even when the entity was already at the maximum. A value above the current hit points (and not above the maximum) runs the heal notification and then stores the value. A value below the current hit points runs the damage-applied notification and then stores the value, with no lower clamp and no death step (0 or a negative value is stored as given). An equal value is stored with no notification. The same rule is used by SetHitPoints (row 350) for a hero. | H |
| 87 | GetAI_HitPointsPercentage | int | Returns current hit points divided by maximum hit points, as a float. Returns -1 when the entity is missing. | H |
| 88 | NumMiamiceInCage | none | Returns the Miamouse cage count. | H |
| 89 | SetWhinnySwitches | W, int array inner, int array entrance, int array locators, int n | Sets the Whinny pen list: pen count = n; for each i < n, pen i gets inner-area id inner[i], entrance id entrance[i], pen point = the +0x30 point of locator locators[i], and occupied 0. The pen arrays hold 4 pens, so n above 4 overruns them (the disc passes 4, in module 1398). Despite its name it sets no switch, and it does not use the batch switch form of section 2.1. | H |
| 90 | SetExternalMissileTarget | int id | Arms a one-shot aim override for the hero's shots: it stores the flag and the id (the id is a world locator index, not a target or entity id). The hero shot aim step, run once for each shot of a burst, tests and clears the flag. When the flag was set, the shot's aim point is replaced by the position of locator id (the locator record's +0x30 point, row 130's locator table; an id below 0 or at or above the locator count leaves the normal aim point, and the flag is still cleared). The launch direction is then the unit vector from the shooter's launch point to that aim point. The target id the shot carries is not changed. Only the first shot after the call is affected. The two query calls (rows 359 and 363) are not imported by any module; the shot code reads the same two words directly. Disc: one call site, pack 1428 CB6, id 4 (locator 4, the shot target's centre; SPEC-0041). | H |

**Category 4: Dialogue and messages**

| No. | Name | Arguments | Effect in plain words | Conf. |
| --- | --- | --- | --- | --- |
| 91 | DebugPrintf | string pointer, variable | Prints only when the debug test and the debug word at data address 0x2D7554 are both set. No game-state effect. | H |
| 92 | LevelMsg | int line, W, bool, WAIT | Shows line i of the world's text in the shared dialogue box (SPEC-0022). There is no other display path: no banner, no timer and no auto-dismiss; the box closes only on the player's "Finish" press. When line i is out of range, nothing is shown, no state changes and the call returns at once, whatever the wait mode. Otherwise the speaker and colour rule of SPEC-0022 applies; if the hero is not busy (states 33, 48, 50 to 53 and 55) it is put into the talking state (25) without saving its previous state; the box opens with the line; with WAIT = 1 the call yields each tick while the box is open (only the value 1 waits). The bool is stored and never read. | H |
| 93 | SetConversationTarget | int id, bool b | The hero and the entity both enter the talking state (25), each saving its previous state unless it is already in state 25. The hero's partner becomes the entity; the entity's partner becomes the hero only when b is 1. The dialogue box owner is set to the conversation. In state 25 a character stops and turns its facing toward its partner each tick by a share of the remaining angle: 0.1 for the hero, 0.3 for a townsperson (SPEC-0056; a townsperson in a pose mode does not turn); with no partner it does not turn. The partner is the speaker that LevelMsg uses (SPEC-0022). | H |
| 94 | EndConversation | int id | The hero, if still in the talking state (25), returns to its saved state, and its partner is cleared. The entity is treated the same way. The dialogue box owner is cleared. A hero that already left state 25 (for example when the box closed between two lines and the hero returned to state 1) keeps its current state. | H |
| 95 | IsButtonPressed | BUTTON, bool, T(bool) | Query: whether a controller button is held or was newly pressed. The first argument is a button code, mapped to a pad mask (Confidence High): 0 Square (0x0080), 1 Triangle (0x0010), 2 Circle (0x0020), 3 Cross (0x0040), 4 Start (0x0800), 5 Select (0x0100), 6 L3 (0x0200), 7 debug query (always false in this build, section 8), 8 L1 (0x0004), 9 R3 (0x0400), 10 R2 (0x0002), 11 R1 (0x0008), 12 any button (0xFFFF), 13 D-pad (0xF000), 14 the four face buttons (0x00F0), 15 L1, R1, L2 or R2 (0x000F). The call is bound directly to its body (no wrapper). The second argument selects which of two state words of controller pad slot 0's own record is read (not the combined menu snapshot; SPEC-0033): false reads the held word, true reads the newly-pressed word (the button went down this pass). The button bits are fixed: 0x0004 is L1 and 0x0008 is R1. The third argument: if true, the result is always false, so a call of the form IsButtonPressed(x, false, true) is constant false. Code 7 always gives false. The result is also false while the game-global word at data address 0x35C9C4 (base 0x35C980 + 0x44) is nonzero, and while the word at 0x35C9A0 (base + 0x20) is nonzero, which clears the mask. Script threads run after the pad read and after the hero's update in the same pass (SPEC-0026, main loop step 8.1.12). | H |
| 96 | DisplaySquareButtonContext | context message | Shows label n of design table 13 (SPEC-0024) as the Square caption of the HUD (SPEC-0023, section 33), formatted with the call's extra arguments, and sets the hero's script-prompt flag for the tick (that flag blocks the hero's no-target Square stance entry and is cleared at the end of every hero input step; SPEC-0033). n = 0 hides the caption. The whole call is ignored while the game-global word at data address 0x35C9A0 (base 0x35C980 + 0x20) is nonzero. A caption is shown only on the pass in which it is set, so a script that wants a lasting prompt calls this every tick. Contexts used on the disc: 1 to 6, 13, 26 and 32. | H |
| 97 | DisplayScreenMessage | string pointer, float, float, float | Shows on-screen text at a 3-D position. The third value is raised to at least 1.3 before the point is projected to the screen. Added 2026-11-09 (High): the draw side queues a 0x20-byte record; at draw the point is multiplied by the camera's 4x4 at camera +0x460 (x, y, z times rows 0 to 2 plus row 3), x, y and z are divided by w and (1792, 1824) is subtracted; the text is drawn in text mode 3 only when w > 1.0 and -10 < x < 522 and -10 < y < 458 (strict). The draw returns the number of characters drawn, and the flush call draws every queued record in order and empties the queue. | M-H |
| 98 | FloatToChar | float, int, string pointer | Formats a number as text into a buffer. No game-state effect. The routine that does the formatting is not named (Open question 49). | M |
| 99 | IsEntityTalking | int | Entity lookup in the current world. True when the entity's base state (+0x144) is 25 (0x19), the talking state. False when the entity is missing. | H |
| 100 | LevelPrompt | int i, W, int labels | Shows line i of W's level text table in the shared box, as LevelMsg does but without its wait (SPEC-0022, Showing a level line). It then sets choice mode with two labels and a cleared answer (-1), yields one tick at a time while the box is open, and returns true when the answer is 1 (Circle). labels below 0 gives the built-in labels Yes (Circle) and No (Triangle); labels of 0 or more takes the two labels from record labels of the same table (SPEC-0022, Choices). When line i is out of range (-1 included), nothing is shown and the call returns false at once (confirmed 2026-11-09, High: the choice setup still runs on the closed box before the false return). Every live call on the disc passes -1 for labels, so in play the labels are always Yes and No; no call passes a labels value at or beyond the table's count, which would leave no record to read. | H |
| 101 | IsDialogActive | none | Query: true while the dialogue box has text or a pending stop. | H |
| 102 | IsHeroTalking | none | Query: true when the hero is in the talking state (25). | H |
| 103 | SetConversationTargetWithOrient | int id, bool b, T(bool) t | As row 93, except that the hero's partner is set to the entity only when t is 1. With t = 0 the hero has no partner, so it does not turn, and a line shown by LevelMsg uses the line record's own speaker name. | H |

**Category 5: Entities and NPC AI**

| No. | Name | Arguments | Effect in plain words | Conf. |
| --- | --- | --- | --- | --- |
| 104 | LoopEntityAnim | int entity, int anim, float seconds, CTL | Starts a looping animation on an entity and sets its state to loop (0x37). The seconds value s becomes a tick count. A negative value gives one tick. Otherwise the count is trunc(s x 60): one single-precision multiply by the tick rate (60, written at start-up; SPEC-0007, Rate selection), truncated toward zero (SPEC-0008). A count of 0 is stored as -1, which loops forever, so any time below 1/60 s (0 <= s < 0.0166667) loops forever, not only zero. 27 call sites that play reaches pass 0.01 s, so those loops run until something replaces them. CTL is applied when the loop's tick count runs out (Layout section 2.1). | H |
| 105 | PlayEntityAnim | int entity, int anim, WAIT, CTL | Plays the animation (by table index) on the entity and sets its state to 0x21 (script control). With WAIT equal to 1 (not merely nonzero) the caller yields each tick until the animation is done; the entity then returns to its idle animation and to state 1 (CTL 1) or 0x21 (any other CTL). Without the wait the CTL choice is stored and applied when the animation ends. | H |
| 106 | WalkEntity | int entity, int locator k, WAIT, float stop, CTL | Walks an entity to the position of a locator. Needs k from 0 to the current world's locator count - 1 and the entity in the current world (Note on entity ids); otherwise nothing. In order: (1) the entity's control-flag word is set to 1 when CTL is 1, else to 0; (2) base state 1; (3) the move goal is locator k's point, and the walker's path-replan marker is raised when that goal lies more than 1.0 (horizontal distance) from the previous goal and the walker's cached ground triangle does not contain it; (4) the walk target is that point with its height replaced by the ground height under it, probed from 6.0 above the point; (5) when stop is above 0, the stop radius is set to stop and class 24 is requested, otherwise class 23; the request has priority 2 (SPEC-0056). Class 23 stops when the distance to the target is below 1.0; class 24 stops when it is below the stop radius. Priority 2 is above the entity's default class (priority 1), so the request parks the entity's running AI coroutine. When the walk class ends, the class word and its priority reset to 0 and the parked coroutine resumes where it yielded, so a townsfolk character keeps its mode and wander point across the walk (Medium-High). With WAIT set, the caller yields one tick per loop while the entity's current requested class (row 113) is 23 or 24, with no timeout; it does not poll request code 0x32. CTL does not change the state at the end of the walk: the NPC reads the control-flag word only when a LoopEntityAnim loop ends (state 55 becomes 1 when the word is set, else 33). A walk that is put under script hold follows section 4 (Script hold during a walk). See SPEC-0056 (NPC AI modes and walk routines). Disc: 76 sites; WAIT is set at 5, CTL is 1 at 50, and stop is 1.1 at 14 resolved sites. | H (Medium-High for the ground probe and the coroutine resume) |
| 107 | ReleaseEntityControl | int entity | Sets the entity's base state to 1 (free): from the next NPC update the requests of its AI are applied again (section 4, Script control and the entity's AI). A missing entity: nothing. Disc: 69 sites. | H |
| 108 | WalkEntityAndOrient | int entity, int locator k, WAIT, float stop, CTL | As row 106 (same arguments, index rule, control-flag word, base state 1, ground-probed walk target and priority 2), and the final facing is the locator's direction with its y set to 0 and then normalised. A positive stop selects class 26 and is stored as the stop radius; zero or a negative value selects class 25. Class 25 stops when the distance to the target is below 1.0, then turns the entity until the dot product of its facing and the final facing is at least 0.99. Class 26 stops when the distance is below the stop radius, then turns in the same way. The turn moves the facing by 0.1 of the remaining difference each tick. With WAIT set, the caller yields one tick per loop while the current requested class is 25 or 26. See SPEC-0056 (NPC AI modes and walk routines). Disc: 75 sites; WAIT is set at 19 and CTL is 1 at 56. | H |
| 109 | GrabEntityControl | int entity | Sets the entity's base state to 33 (script hold). From the next NPC update the requests of its AI are stored but not applied, the entity cannot be hit, and each tick in which its current clip's progress is at least 0.99 it requests its default idle animation (row 120) (section 4, Script control and the entity's AI). A walk class applied before the grab freezes, but the clip already playing keeps carrying the entity until its progress reaches 0.99, and the walk resumes on release (section 4, Script hold during a walk). A missing entity: nothing. An entity whose record is not currently bound to a live character is missing too: the call returns at once, stores nothing and is not retried later, so a script that grabs an entity before the streaming pass has bound it (pack 985's King on the first tick) holds nothing; the animation calls of rows 104 and 105 behave the same. It is released by row 107 or by any call that sets state 1 (MoveEntity with CTL 1, WalkEntity, PlayEntityAnim or LoopEntityAnim ending with CTL 1). | H (Medium-High for the drift of an applied walk) |
| 110 | ReturnEntityAnimToIdle | int | Asks the entity to return to its idle animation and sets its base state to 1, so its AI runs again. | H |
| 111 | WalkEntityAlongPath | int entity, int path, WAIT, CTL | Entity lookup only, with no locator check. The control-flag word is set as in row 106 and the base state to 1. The path index is stored in the +0xB8 word of the entity's placement record (a path number of the area's path list, SPEC-0006 Tagged tables, tag 0x0A) only when it is 0 or more; a negative index leaves the placement's own value, so a call with -1 walks the entity along the path its placement names (the pack 1023 cycle zones do this for every villager, SPEC-0045). Class 35 (path walk) is requested with priority 2, which parks the entity's AI as in row 106. Class 35 starts at point 0, reaches each path point when the horizontal distance to it is below 0.5, then moves on to the next point, and ends one tick after the last point is reached (SPEC-0056, Path walk). With WAIT set, the caller yields one tick per loop while the current requested class is 35. See SPEC-0056 (NPC AI modes and walk routines). Disc: 34 sites; WAIT is set at 3. Of these, 26 are an unused path branch of the shared entity-control routine (never selected), 3 are the cycle zone threads (modules 758, 991 and 1023; the 991 one is unreachable) and 5 are scripted walks in module 1300 (entities 8, 14, 15, 16 and 17 with path arguments 1, 6, 7, 9 and 10). The class does not set the movement class; it uses the one the townsfolk routine last set (the move type). The walker needs a navigation mesh (kind 1, containing its position when its townsfolk routine first ran) or it does not move. Speed: SPEC-0056, Walk speed and movement type (5.00 units/s, 1.71 for animation table variant B). | H |
| 112 | MoveEntity | int entity, int locator, bool probe, CTL, bool face | The entity form of MoveHero (row 156): places the entity at the locator's position; probe on: the entity's final height is the probed ground height at the locator's x and z, which replaces the locator's height (the same probe as row 156: ceiling = the record height + 4.0; the highest accepted ground patch at or below the ceiling, else the last rejected sample, which may lie above the ceiling, else -1.0e9 when nothing was sampled, and 0.0 when the point has no ground cell; SPEC-0018, Ground probe); the result is stored without any check, so a miss places the entity at -1.0e9 and an empty cell places it at height 0; probe off: the locator's own height is used; face = also take the locator record's facing, written to the body axes only (not to the stored steering facing): with CTL 1 (state 1) the body returns to the stored heading on the next tick, so the locator heading shows for one tick; with any other CTL (state 0x21) the body keeps the locator heading until the entity is released (Medium; SPEC-0038 section 52); CTL 1 sets state 1, any other value 0x21. A locator outside the table or a missing entity does nothing. Disc: 136 sites. A second host record with a player-record first argument does the same on that record (it does nothing when that record is null); no module calls it. Added 2026-11-09 (High): unlike MoveHero, the entity form updates the NPC move target to the position first, then resets the placement (or transform, when face is clear), and sets the state (1 when CTL is 1, else 0x21) last; MoveHero sets the state before the placement. | H |
| 113 | IsEntityWalkToPointActive | int entity | True when the entity's current requested class is 23, 24, 25, 26 or 35 (the walk classes of rows 106, 108 and 111, including the path walk). The current requested class is the pending request when its priority allows it, else the applied class, so the call is true at once after a walk call, before the next NPC update. It stays true while the walk is frozen under script hold (section 4). A path walk stops counting one tick after its last point is reached (the class reads 0 again). False for a missing entity. The answer is true in the same script tick as the walk call, before the walker's next AI step; the cycle zone threads rely on this, because they ask right after issuing the walks and deactivate a villager whose answer is false. | H |
| 114 | DeactivateEntity | int | Kills the entity permanently: runs its death hook and releases it with the kill bit of its spawn record set (SPEC-0006), so streaming never binds that record again until a forced bind (row 116) clears the bit. | H |
| 115 | SetAI_Agro | int | Turns the entity's aggression on. For a living entity (a dead one is left alone) it sets base state 1 and class-17 state 2, the alerted search (SPEC-0038). The state is 21 instead when the entity's monster row (SPEC-0029) has its Display in Default word (+0x82) equal to 0 and invisibility has not been disabled for the entity (row 121). It sets the search countdown from monster-row words +0x60 and +0x5C, sets the last-seen point to the hero's position, and requests animation 18 (149 for flyers). It does not enter agro (state 19) directly. It does not test the entity's current state: on an enemy waiting hidden in state 8 it replaces the wait with state 2 (21 for a Display-0 row) and animation 18 without the emerge animation, the enemy must then pass its own notice check to reach agro, and otherwise returns to its default state (R-0147; SPEC-0038 Behavior 40). Any mode-change timer drawn with the new mode (60 to 180 ticks) has no known reader, so it looks unused (Medium). | H |
| 116 | ActivateEntity | int | Force-binds the spawn record now (looked up by id), whether or not it is within streaming range, and clears its kill bit (SPEC-0006). The int is the entity id, the position in the entity-id table (Open question 63). When no entity matches, nothing is activated (Edge cases). Added 2026-11-09 (High): the entry resolves to an entry id and a slot. A slot below zero means the world's always-loaded character block: the bind there picks the record with that id (index -1 means the first record whose bound flag is clear); it does nothing unless the block's attached count is below its character cap, returns "bound" at once for an already bound record, and otherwise spawns the character and, when the spawn worked, enters state 1 on it, sets the bound flag, stores the id byte, appends the object to the attached list, bumps the attached count and registers the character's sound references; it reports success even when the spawn failed. Any other slot binds the region slot. The kill bit of the id is cleared by the bind, and the debug message prints only when nothing was bound. | H |
| 117 | SetAI_MovementType | int entity, movement type | Stores the movement type in the entity's navigation block (SPEC-0056: 0 walk, 1 run, 2 fast run). The townsfolk routine and the walk classes 23 to 26 copy it to the movement class every tick, so it also changes the gait of a patrolling villager (id 8 is used only at type 0). Disc: 12 sites in 7 modules: module 1023 (entity 1, type 2), 678 (entities 1, 2 and 7, type 2), 842 (entities 3, 5 and 7, type 2: the Cogham villagers followed by Ixi Spearmen), 909 (entity 31, type 1), 956 (entity 28, type 7), 1163 (entity 9, type 0) and 1335 (entity 36, type 1). | H |
| 118 | DisableVillagerSocialize | int entity | Sets the villager's social gate word to 1, so it is not eligible for villager pairing (SPEC-0040, Behavior 3B). Row 342 clears it; it is 0 at creation. | H |
| 119 | SetAI_Behavior | int, behaviour code, data pointer | Sets an AI entity's behaviour code and data. The AI data block is the entity's placement record itself (SPEC-0006): when the data pointer is not null, bytes 0 to 27 of the given block are copied over record data +0xBC to +0xD7. The behaviour code is the s16 at record data +0x84; on disc every placement sets it. If the new code differs from the stored one, the code is stored and the mode word is set to 0 directly (no mode setter and no timer); with the same code only the block copy happens. On the entity's next behaviour-class-6 (townsfolk) tick, the class's pre-step sees mode 0 and sets the mode to the code before the mode body runs. The new block values act at their next use (for example the next wander point). See SPEC-0056 (NPC AI modes and walk routines). | H |
| 120 | SetDefaultEntityIdleAnim | int entity, int anim | Stores the idle animation id that the NPC requests while it is held in base state 33 (section 4, Script control and the entity's AI). | H |
| 121 | DisableInvisibility | int entity | Sets the entity's invisibility-off word (+0xE60, 0 at creation). For an enemy whose monster row has Display in Default 0 (an invisible kind; SPEC-0038) this has two effects: (a) the hit exclusion of invisible kinds no longer applies (without the word, the enemy is excluded from hits while in its default state, or in state 21 while not playing animation 82, unless the hero's outfit slot holds the Tattered Cloak, item 126); (b) SetAI_Agro (row 115) and the alerted search enter state 2, not 21. Disc: 25 sites. | H |
| 122 | GetClosestNavMeshId | W, int entity | Looks up the entity in W (Note on entity ids). Returns -1 when the entity is missing or W has no nav-mesh list. Otherwise it takes the entity's position (x, y, z) and a mesh kind: kind 2 when the entity's monster row has a nonzero Can Fly value, kind 1 otherwise (SPEC-0018 mesh kinds; SPEC-0029 Can Fly). For each mesh of that kind, in list order, it finds the first triangle in array order that contains (x, z): a point is inside when, for each edge from vertex a to vertex b in vertex order (0 to 1, 1 to 2, 2 to 0), (x - a.x)(b.z - a.z) - (b.x - a.x)(z - a.z) is 0 or less, so edges count as inside. The mesh height h is that triangle's vertex 0 y. The result is the index of the mesh with the smallest abs(h - y); a later mesh replaces an earlier one only when strictly closer. When no mesh of the kind contains the point the result is -1. The mesh count is read from W's list and the meshes from the current world's list (the same list whenever W is the current world). Disc: 5 live sites, each in a module's per-tick update (modules 822, 906, 956, 986 and 1098). | H |
| 123 | SetAI_ConversationTarget | int entity, int t | Sets the entity's talk target: t = -1 the selected hero, t >= 0 entity t, t < -1 none. In base state 25 (talk) the NPC turns toward its talk target each tick with rate 0.3. | H |
| 124 | MoveObjToAnimatedLocator | int object o, int host h, int locator k, bool follow, W | Uses W, or the current world when W is 0. Needs o and h in the world's animated-object list, an inline locator list on h (h's type-11 header, SPEC-0028) and 0 <= k < that list's count; otherwise nothing happens. follow true: o's locator link becomes (h's locator record k, host h) and nothing else happens at the call: o is not moved by the call itself. From then on every pose evaluation of o (the per-tick update of states 0, 2, 5 and 6, and the absolute evaluations of InitObjOn, ActivateObj and SetObjToTime) replaces o's base position by the locator's current point, base = L + r + (D d - d), where L is the record's +0x30 point, r the host's root offset (+0x80), d = L - the host's rest translation (x, y, z), and D the host's rotation from its rest orientation to its current one (identity for a host that does not turn, which is every host on the disc; then base = L + r). o's position is its own root offset plus this base (plus its +0x90 offset when its mode byte is 1 or 3). Only position is taken: o keeps its own rotation, scale and own clip, the locator's facing is not used, and o's rest translation is ignored while the link is set. There is no per-frame copy: an o in state 8 or 10 that nobody re-poses stays where it was last posed even while h moves, so scripts re-pose followers themselves (SetObjToTime in a thread, or PlayAnim on o). The link reads h's pose as stored at that moment; a host that stops, is deactivated or changes clip simply leaves its last root offset in effect, and a looping host's root resets to zero at each wrap (R-0270; SPEC-0042, module 990, has the worked case). follow false: o's rest translation (+0xF0) is set once to the locator's current world position (formula A, Note on animated locators), and o's last-update change (+0x150) is set to (0, 0, 0, 1). Disc: 10 sites, in modules 990, 1077, 1089 and 1091, all with follow true (no site uses false). They run 44 effective calls: 990 object 0 hosts objects 39 to 50 at locators 0 to 11 (the lever lift, SPEC-0042); 1077 and its copy 1129: host 24 carries objects 32 and 33 at locators 1 and 2; 1089: host 0 carries objects 12 to 17, 18 to 23, 24 to 29 and 30 to 35 at locators 1 to 6, 9 to 14, 17 to 22 and 25 to 30; 1091: hosts 75, 100, 111, 116, 130 and 134 carry objects 135, 51, 52, 53, 136 and 54, locator 0 each (four more table rows pass host -1 and do nothing). Every host animates (lift or descent clips with no rotation), so on the disc base = locator point + host root offset. Test vector (module 990): object 39 follows locator 0 of object 0, point (-1.082, 0.560, 68.602), host y root 0 to 6.997 over 1.933 s; with object 39's own offset 0 it is posed at y 0.560 at load and y 7.557 when the lever animation has ended (R-0270). | H |
| 125 | SetEntityUndamageable | int entity, bool b, W | Looks the entity up in W; nothing when W is 0 or the entity is missing. Sets the undamageable flag (+0x7F4) to 1 or 0. While it is set the entity still takes hits: the hit reaction, the switch to agro (class-17 state 19) and the hit count all run; only the hit-point loss is skipped. Disc: 22 sites in 4 modules (784, 1056, 1327 and 1398). At these sites W is not 0: the module loads it from its own data word that holds the module's stored world, so the call takes effect (High for module 1056; Medium-High for the other three). Module 1056 sets it on entities 4, 7, 8, 9, 14, 20 and 21; which characters these are is not mapped. Module 1398 sets it every tick on entities 0 to 4 (the two Drackonacks and three Giant Drackonacks of region 1399) and 16 to 21 (the six Evil Harrises of region 1403) and never clears it, so they cannot be killed there (SPEC-0043, 1398; R-0146). | H |
| 126 | SetTargetActive | int target, bool b | For a valid target of the current world’s aim-target list (0x60-byte records at world +0x5824, count at world +0x5820), writes its active word at +0x58. Each target record is built at world load from one n2 entry of the type-11 chunk (SPEC-0028 section 6; count at header +0x20, 0x0C-byte entries). Its marker source at +0x50 is the registry model selected by entry +0x00 when inline mesh +0x04 is zero; otherwise it is the inline mesh. The initial target point at +0x40 is the xyz position in the locator-like record at entry +0x08 (its fourth lane is zero). The active word starts at 1. While item 36 is held, the hero’s scan considers active targets at a three-dimensional distance below 110.0 units. It scores the normalized vector from the camera eye to each target against camera forward; only a best score above 0.998 can be selected. The line-of-sight sweep runs from 1.5 units below the camera eye to the target and samples cells at growing offsets along the normalized camera-to-target segment: a step counter t starts at 0 and, after each probe, grows by 5.0, and the probe point moves from where it was by the direction times the new counter, so the probes lie 0, 5, 15, 30, 50, 75 ... units from the start (not 0, 5, 10, 15; R-0273, High). The loop runs while the counter (not the distance travelled) is below the segment length, so a segment of 5 or less probes only the start cell. For each sampled cell, its eligible frame records (no exclusion of type 6) and its class-3 volume records are tested against the full segment from the lowered camera point to the target, not just the interval since the previous sample. A class-3 volume is eligible only when its kind is 3, its class bit 3 is clear and its radius is above 4.0; it is then tested as a flat-ended cylinder of its stored length with radius plus 2.0 (SPEC-0018, Other cell-entry readers). The segment is not shortened before the target. A blocked target cannot become the best candidate. Obstructions in cells between sample positions can be missed; the collision geometry in each sampled cell is tested against the whole segment. Each scan clears the aimed word at +0x54 for every target, then sets it for the chosen target. The scan returns 0 for no selection, 1 for a character aim candidate, or 2 for a Rope Gun anchor; a successful anchor pass takes precedence. It writes the hero's world-space shot aim point separately: the selected character's aim point plus half its motion height when a character candidate passes, or camera position plus 150 times camera-forward otherwise. Anchor coordinates remain in the Rope Gun manager. | High for compared vectors, range, sightline, return values and stored point; Medium-High for remaining scan rules |
| 127 | GetDistanceToGoalPos | int entity | Returns the 3-D distance from the entity's position to its stored move goal (set by the walk and move calls). Returns 0.0 when the entity is missing or its goal-reached flag is set (the navigation step sets it within 0.25 units of the goal). | H |
| 128 | GetAnimatedLocPos | int object h, int locator k, float outputs x, y, z | For a valid h, writes the current position of h's locator k (formula A, Note on animated locators). When h is invalid the outputs are left unchanged; an invalid k writes (0, 0, 0). | H |
| 129 | IsEntityReactingToHit | int | Entity lookup in the current world. True when the entity's AI mode word (+0xA34) is 19 (the alert mode, row 59) and its attack sub-state word (+0xAF8) is 10, the hit reaction. False otherwise and when the entity is missing. | M-H (the tests High; that sub-state 10 is the hit reaction Medium) |
| 130 | MoveObjToLocator | int object o, int locator k, bool link | Needs o in the animated-object list and k in the world locator table (section 3). link true: o's locator link becomes (record k, no host), so its base is the locator's +0x30 point on every update. link false: o's rest translation is set once to the locator's +0x30 point, and its last-update change (+0x150) is set to (0, 0, 0, 1). Disc: 1 site, in module 1023 (object 4, locator 498, false); in a world with 498 or fewer locators the call does nothing. | H |
| 131 | MoveTargetToAnimatedLocator | int target t, int object h, int locator k | For t in the world's aim-target list (row 126), if k is valid on h, sets the target's point at +0x40 to the animated locator's current world position (formula B). The target remains active or inactive as before; this call does not apply the locator's direction. | M-H |
| 132 | SetEntityInvulnerable | int entity, bool b, W | Looks the entity up in W; nothing when W is 0 or the entity is missing. Sets the invulnerable flag (+0x7F0) to 1 or 0. While it is set the entity is never a hit candidate (section 4, Script control and the entity's AI; SPEC-0040 12A). The NPC take-hit also sets this flag when the entity's hit count reaches its row limit (when the limit is not -1). The Darkest Faerie's three controllers (SPEC-0054) set this flag to 1 when they start, and nothing clears it, so she is never a hit candidate (High). Disc: 2 sites, in module 1056 (entities 61 and 76). | H |
| 133 | GetEntityPos | int, float outputs | Writes an entity's position into float outputs. | L |
| 134 | GetEntityCurrentNavmeshID | W, int | Query: the nav-mesh id the entity stands on. | L |
| 135 | MoveEntity | int entity, float x, float y, float z, CTL | Same host record as row 332. Sets the entity's move goal and its position to the point (x, y, z), with no ground probe; the facing is kept. CTL 1 sets base state 1, any other value 33. A missing entity: nothing. Disc: only module 1398 (script trigger 34) calls it, twice: it places entity 23 (CTL 1) and entity 22 (CTL 0) at a point read just before with GetEntityPos. | H |

**Category 6: Fades, letterbox and movies**

| No. | Name | Arguments | Effect in plain words | Conf. |
| --- | --- | --- | --- | --- |
| 136 | FadeScreenOut | RGB c, float s, WAIT | The fade lasts n = max(1, trunc(s x 60)) frames, formed by one single-precision multiply by the tick rate (section 4, Seconds to ticks). c is passed by value: R is bits 0 to 7, G bits 8 to 15 and B bits 16 to 23; the top byte is ignored. Sets the fade colour to c, alpha 0, target 128 and step 128 / n (SPEC-0052, Screen effect: requests, Fade out). With WAIT nonzero the caller yields one tick per loop while alpha differs from the target. There is no timeout, and a new fade started by another thread restarts the wait. Disc: 264 call sites in 54 modules (section 7, Disc call-site counts). Resolved fixed durations: 1.0 s (85 sites), 0.5 s (82), 0.7 s (51; 41 frames), 0 s (24; 1 frame), 2.0 s (4), 0.1 s (3), 0.2, 0.25, 0.3, 0.4 and 1.5 s (2 each), and 0.6 and 1.2 s (1 each), and 0.8 s (2 sites). One helper call site has a caller-selected duration of 0.05 s (3 frames) or 0.03333334 s (2 frames). No fixed-duration FadeScreenOut site remains unresolved. WAIT is set at 236 of the 264 sites. | H |
| 137 | FadeScreenIn | RGB c, float s, WAIT | As row 136, with alpha 128, target 0 and step -128 / n (SPEC-0052, Screen effect: requests, Fade in). Disc: 246 call sites in 51 modules (section 7, Disc call-site counts). WAIT is set at 51 of them. Durations (every site is a fixed value): 0.5 s (98 sites; 30 frames), 1.0 s (72; 60 frames), 0.7 s (49; 41 frames), 0.4 s (8; 24 frames), 2.0 s (5), 1.5 s (4), 0.3 s (3), 1.8 s (2; 107 frames), 0.6 s (2), 0.8 s (1; 48 frames), and 0.2 s and 0 s (1 each). No FadeScreenIn site takes a computed time. | H |
| 138 | PlayFMV | int movie, bool | Queues the movie in append mode; ids 92 and above are ignored. The bool is not used (SPEC-0026). | H |
| 139 | LetterBoxOff | float s, WAIT | Clears the held flag, then requests bars down over n = max(1, trunc(s x 60)) frames (SPEC-0052, Screen effect: requests, Bars down). With WAIT nonzero, the caller yields one tick per loop until the bar done flag is set. Disc: 54 call sites in 22 modules; WAIT at 8; durations 0 s (27; 1 frame), 1.0 s (23), 0.5 s (2), 0.7 s (1) and 0.2 s (1; 12 frames); every site is a fixed value. | H |
| 140 | LetterBoxOn | float s, WAIT | Requests bars up over n = max(1, trunc(s x 60)) frames (ignored while the bars are held), then sets the held flag. With WAIT nonzero, the caller yields one tick per loop until the bar done flag is set. When the request was ignored, the done flag keeps its old value. Disc: 52 call sites in 21 modules; WAIT at 7; durations 0 s (43), 1.0 s (5), 0.5 s (2), 0.2 s (1), 1.5 s (1). | H |
| 141 | SetReturnWorldForFMV | eW slot s | Stores the pack of slot s (slot table A, SPEC-0006) as the movie manager's return world. The last call wins, and the value stays set until the next movie hand-off, which uses and then clears it (SPEC-0026, Movies), so a return world set without a movie applies to the next movie played. There is no range check. Every site passes a fixed slot (13 sites): 31, 74 (twice), 78, 109, 150, 151 (four times), 64, 178 and 191. | H |
| 142 | HasMoviePlayed | int id | True when id is below 92 and that movie's played word is set; false for 92 and above. | H |
| 143 | OpenLetterBox | float seconds, WAIT, colour | Converts the time to n = trunc(seconds x 60) ticks (the tick-rate word 0x368FA0), at least 1, and starts the letterbox opening over n ticks; when a colour is given, its r, g, b and alpha bytes are taken for the bars. Nothing starts while a guard word of the bar object (+0x30) is nonzero; that word is the bars-held word (1 after LetterBoxOn, 0 after LetterBoxOff; SPEC-0052), so OpenLetterBox and CloseLetterBox do nothing while the held bars are on (Medium-High). With the wait option set, the caller yields one tick at a time until the bar object reports done. Disc: 2 call sites, both in module 990 (one script-trigger routine, which opens and then closes twice). | H |
| 144 | CloseLetterBox | float seconds, WAIT, colour | As row 143 (same tick count, colour bytes and guard word), with the closing change; with the wait option, the caller yields until the bar object reports done. Disc: 2 call sites, both in module 990. | H |
| 145 | FrontendHUDReset | none | Resets the heads-up display state. | L |
| 146 | SetReturnTransitionForFMV | eW slot s | Stores the pack of slot s as the return transition; it stays set until the next movie hand-off, as in row 141. At the hand-off, when no return world is set and the exterior is current, the transition is copied into the return world, and the transition buffer is reloaded with it. Because the copy makes the return world set, that reload lands on entry record 0 (plane -1) and is made current. Disc: the only site is in pack 1184 (script trigger 7: slot 131, which is pack 1220), just before PlayFMV(40). | H |

**Category 7: Game-state and blackboard flags**

No host call in any category opens the save screen, saves the game, or sets a save-point flag. Save points are region checkpoint records handled by the world update (SPEC-0025, SPEC-0006); a level affects them only through its script threads (section 4 of Behavior, Save prompt while a sequence runs). High (no such call among the host names, and none in the calls made by the 163 distinct modules).

| No. | Name | Arguments | Effect in plain words | Conf. |
| --- | --- | --- | --- | --- |
| 147 | QueryGameState | int | Returns the game flag with that id. There are 743 game flags (ids 0 to 742) in a bit set; ids above 742 read 0. | H |
| 148 | SetGameState | int, bool | Sets (true) or clears (false) the game flag with that id (0 to 742; larger ids are not stored). Setting or clearing notifies the flag's callback (row 383) when the current world has a script manager. When the flag rises from 0 to 1, the stinger mapped to that flag (if any) is played, and a per-flag hook runs when that flag's hook entry is set. Confirmed 2026-11-09 (High): the notice carries the bit's value before the change and the new value 1 and is sent only when the world has a script manager; the stinger (played at volume 1.0) and, when the flag's journal row is nonzero, the journal-pending icon happen only when the bit was off; the bit is set in every case, even with no world loaded. ClearGameState (false) has no null test of the world or manager (a missing world crashes), sends the notice with the old value and new value 0, and plays no stinger and raises no journal icon. A second argument some callers pass is ignored. | H |
| 149 | AddBlackboardMessage | message type t, int v, eW slot s | The blackboard has 193 rows, one per world identifier (slot), and each row holds 8 entries of (type, value); type 0 marks a free entry. The call takes row s, uses the entry whose type is t, else the first free entry, and writes (t, v). When the row already holds 8 other types, nothing is stored. A read with no matching entry returns -1 (row 150), so scripts write -1 to mean "nothing" (54 sites); the entry stays in use. Types on disc: 1 (7 sites, modules 1023, 1176, 1350 and 1371), 2 (191 sites, arrival tags) and 3 (9 sites, modules 1184, 1217 and 1220). The board is not saved. It is cleared (every type and value 0) at start-up, when the title screen's Start press is accepted, when a new game is confirmed, when the world is torn down on a quit to the title or a game-over exit, and when a saved game is applied, so it survives every area change, door, movie return and swap. | H |
| 150 | GetBlackboardMessageValue | message type t, eW slot s | Returns the value of the entry of type t in row s, or -1 when no entry matches. The read does not clear the entry. Typical use: an activate-world callback reads type 2 of its own slot to choose its arrival locator (60 sites). | H |
| 151 | IsDebugMode | none | Returns 0 (false) on every path in this build, so debug-gated branches never run (section 8 of Behavior). | H |
| 152 | SetLevelState | int (0 to 63), bool, W | Writes one of the 64 level-state flags of the world: true sets it, false clears it. World 0 means the current world. An out-of-range index or a missing world does nothing. The flags are cleared when the world's buffer is released before each load (and at start-up), so they last for the whole residency of the world: they survive world swaps and interior visits, they are not saved, and a flag set in Init stays set (SPEC-0028, Persistence). | H |
| 153 | QueryLevelState | int (0 to 63), W | Returns the level-state flag at the index. World 0 means the current world. Returns 0 when the index is out of range or no world is loaded. | H |
| 154 | SetGameOver | int | Sets flag E of the game-globals record (data address 0x35C98C) to 1. The argument is not used. On the next pass the game-state code is 2, unless a higher-priority code applies, and the in-play state hands over to the game-over page (SPEC-0026, Game-state code). | H |
| 155 | SetDebugScriptLogState | debug log mode | Does nothing in this build: the debug check is a constant zero, so the log mode is never written. Never runs in play (section 8). | H |

**Category 8: Hero control and state**

| No. | Name | Arguments | Effect in plain words | Conf. |
| --- | --- | --- | --- | --- |
| 156 | MoveHero | int locator i, bool probe, CTL control, T(bool) face | Moves the active hero at once to entry (locator) record i of the current world's entry list (80-byte records; SPEC-0006). From the code; not run in the original (High unless marked). The four arguments are the locator, the probe switch, the control option and the face switch; there is no speed argument and no fade argument (a fade before or after is a separate call). **Range.** An index outside 0 to count - 1 does nothing at all (no state change either); there is no check that the world has an entry list at all. **Position.** The record's position (+0x30) is used with its x and z. Probe nonzero: the ground probe (SPEC-0018) runs at that x and z with the ceiling = the record's y + 4.0 (never from y itself), and the result replaces only the y; the result is stored unchecked, whatever it is: 0.0 for no grid, a point outside the grid or an empty cell; the last rejected candidate (possibly above the ceiling) when nothing is at or under the ceiling; -1.0e9 when nothing is sampled; a negative result is not rejected. Probe zero: the record's y is used as it is. **Control.** Control 1 gives the hero state 1 (free control); any other value gives state 33 (0x21), which ignores pad routes. The state is set on the active hero before the move; entering 33 from another state also zeroes the hero's velocity and root step, entering 1 does neither. **Face.** Face nonzero moves the hero with the record's position and facing (the whole facing vector; the facing basis is rebuilt from it); face zero moves the position only and the facing basis stays as it was (SPEC-0040, Behavior 1C). **Always.** In both forms the position is written with w = 1, the hero's velocity and force are set to zero (whatever the arguments), the locomotion sub-object's stored position follows, the ground attach is redone (grid cell, ground entry and gap, with no position change) and the previous-position word is set to the new position. Nothing else happens: no camera change, no fade, no partner or petpet move, no start-pose change and no input lock beyond the state number. **After a probe-off placement.** The hero starts at the locator height with zero velocity and falls under the shared gravity (SPEC-0040, Behavior 14A): with control 1 the stance state's walk-off check (ground gap above 3.0 and vertical velocity below -3.0) sends it to the air state on about the 4th update after the placement (Medium for the tick), and the air landing rules of SPEC-0033 follow; with control 0 (state 33) there is no walk-off check, it falls until the ground constraint stops it (about 25 ticks from a gap of 10.7; Medium) and stays in state 33 with no landing animation. **Survey of the level modules (from the code and disc data; High for the arguments, Medium-High for the ground heights).** 284 call sites in 89 packs, 308 rows when expanded by locator value. Combinations (probe, control, face): (1, 1, 1) 202 rows; (1, 0, 1) 87; (0, 1, 1) 19; (0, 0, 1) 2. Face is 1 at every site, so no level module uses the position-only form. ActivateHero (row 187) makes the same placement internally with (locator, 1, 1, 1). Probe off (the locator is 10.7 to 32.8 above the ground below it): pack 732 locators 4 and 5; 735 locator 4; 956 and 970 locator 2 (control 0); 1077 and 1129 locator 1; 1217 locator 1; 1220 locator 25; 1253 locators 0, 1 and 2; 1382 locator 0; 1392 locator 0; two more probe-off calls take their locator from data. With probe on, 277 of the 289 rows find an accepted candidate: the hero ends within 0.5 of the locator y in 100 rows, 0.5 to 2 below in 105, 2 to 4 below in 41, 4 to 10 below in 23, more than 10 below in 8 (largest: pack 822 locator 6, 166.5 below); the locator height is authoring slack above the ground, not a fall. Exceptions to the probe-on rule, where the probe result is stored as it comes: result 0.0 (hero y = 0) for pack 732 locator 0, 735 locator 3, 1184 locator 44 and 1220 locator 29 (the point is outside the grid or in an empty cell; Medium-High); last rejected candidate above the ceiling for pack 1098 locators 79 and 7 and pack 1254 locator 0; an object-owned patch (the result depends on the object's pose when the call runs) for packs 956 and 970 locators 43 and 44, 1244 locator 22, 1324 locators 3 to 5 and 1371 locator 14; a masked entry in the cell for pack 1184 locator 50. Call sites whose locator is not a constant: 956 and 970 (a script record field), 1324 and 1392 (an argument); pack 1397 passes -1 (a no-op by the range rule). | H |
| 157 | PlayHeroAnim | int anim, WAIT, CTL | Plays the one-shot animation on the active hero (state 0x33) and stores CTL, then always yields one tick (even with WAIT 0), then, with WAIT nonzero, yields each tick while the hero stays in state 0x33. The one-shot ends when its clip progress reaches 0.99; the hero then goes to state 1 when the stored CTL is 1, and otherwise to state 0x21 (script hold). Of the 106 hero animation calls (rows 157 and 168) on disc, 69 pass CTL 1 and 28 pass CTL 0 when the value is set in the same code path; the other nine pass it through a shared helper that was not resolved (High for the exit rule; Medium-High for the counts). | H |
| 158 | WalkHeroAndOrient | int locator, WAIT, CTL | Walks the active hero to the locator record's position and turns it to the record's facing. With WAIT set, the caller yields while the hero is in state 0x34 (walk and orient). CTL as in Layout section 2.1. | H |
| 159 | WalkHero | int locator, WAIT, CTL | Walks the active hero to the locator record's position. With WAIT set, the caller yields while the hero is in state 0x32 (walk). CTL as in Layout section 2.1. | H |
| 160 | BlockPlayerControl | hero selector | Selects the hero (-1 active, 0 Tor, 1 Roberta). If that hero is present, the ACTIVE hero plays one-shot animation 0 with CTL 1 (as row 157, so the call yields one tick), then the selected hero is put in state 0x21. | H |
| 161 | SetHeroEquippedState | bool b | Writes the party's weapon-ready word (shared inventory) and stores the same value in a second word of the inventory record, whose reader is not identified (Medium). When b differs from the stored value and the sound system is up, the selected hero's sound bank plays cue 0 (b true) or cue 1 (b false); no cue plays when the value does not change. The word is also set to 1 by the hero's attack routines, and to 0 by the talk state (25), state 28, SwitchHero and several hero routines. Hero routines read it to choose armed variants: for example, one route enters state 7 when it is set and state 2 when it is clear. Disc: 23 sites pass false, 14 pass true. | H (store and cue); M-H (meaning) |
| 162 | WhichHeroIsActive | none | Returns true when Roberta is the selected hero (hero index 1). It does not count objects. See SPEC-0053, sections 2.2 and 3.3. | H |
| 163 | ReturnHeroAnimToIdle | none | Asks the hero to return to idle. | H |
| 164 | GrabHeroControl | none | Sets the selected hero's state to 33 (script hold). On entry to 33 from another state, the hero's motion-record velocity and the velocity of its locomotion sub-object are set to zero. In state 33 the hero only requests the animation code in its input word (SPEC-0040 15.3) and ignores pad routes until a call frees it (RestorePlayerControl, MoveHero with CTL 1, or a walk or play call ending with CTL 1). The hold state has no exit of its own: it only requests its current animation each tick, and no other route code leaves it; a scene end does not. When a control thread ends, the step that frees its table entry only lowers the thread count, releases the thread's heap block and empties the handle; it writes nothing to the hero, so the hold lasts until RestorePlayerControl or a CTL 1 call (High). | H |
| 165 | BossFightTweakHero | boss-fight instance selector, boss-fight stage selector | Refills the heroes at the start of a boss fight. Only stage 0 has an effect; any other stage does nothing. For instances 0 to 3 it refills the health and the second meter (probably magic, Medium) of hero slot 0 (Tor's record) only, whichever hero is selected: each current value is set to its maximum and the stored-value notification runs. For any other instance it first revives the hero that is not selected if that hero is in the death state and the swap mode is exactly 2 (state 1, health raised to maximum; with any other swap mode nothing is revived), then refills both heroes the same way. A hero in the death state who is not revived gets the full health value but stays dead (SPEC-0055, health gain in the death state). The refill changes play: both heroes start the fight at full health and full second meter. The instance number has no other effect. In pack 1246 the fight-start call is (14, 0), a full refill of both heroes, and the death-scene call (14, 1) does nothing (SPEC-0039, section 1246). Instances used on disc: 0 to 3 and 5 to 14 (4 is never passed). Pack 1428's boss-room return calls it with instance 12, stage 0, after the entity and hero moves and before the fade-in (SPEC-0041). | H |
| 166 | RestorePlayerControl | hero selector | Selects the hero as in row 160 (the argument is used). If that hero is present and in state 0x19, 0x21, 0x33 or 0x37, sets it to state 1. It does not otherwise touch player input. | H |
| 167 | SetHeroSwapMode | swap mode | Writes the swap mode, the lock on switching: 0 locks to Tor, 1 locks to Roberta, 2 is free (the default), 3 and 5 refuse every switch, 4 is free and places the incoming hero at its own stored spot. Values on disc: 0, 1, 2, 3 and 5. See SPEC-0053, section 3.1. | H (guard), Medium-High (labels) |
| 168 | LoopHeroAnim | int, float, CTL | Loops a hero animation for the given seconds, converted to ticks as in row 104: trunc(s x 60) in single precision at the tick rate (60 per second, written at start-up; SPEC-0007, Rate selection), and one tick for a negative time. Unlike row 104, a count of 0 stays 0; it is not turned into the forever value. The call stores whether CTL is 1 in the hero's control-flag word, requests the animation on the selected hero, sets the hero's request code to 0x37 (loop) and stores the tick count in the hero's loop counter. It does not wait. While the hero is in the loop state (0x37), the counter drops by one each tick while it is above zero; at zero the loop ends: the hero goes to state 1 when the stored CTL is 1, and otherwise to state 0x21 (High). | H |
| 169 | DamageHero | int | Applies the given amount of damage to the current hero through the hero's full hit routine, with attacker id -1. The hit routine's rules apply: a hero in the death state takes no damage, and a hit that takes health to zero runs the death handler with the gate allowed. See SPEC-0055 (hero death and respawn). | High |
| 170 | IsHeroOccupied | none | True when the selected hero is in state 7 to 19, 21, 23 to 30, 33, 35, 37, 38, 41 or 43 to 60; or has an active stun (status 0 with its timer at 1 or more); or the screen effect is moving (fade alpha differs from its target, bar alpha differs from its target, or bar progress differs from its target; SPEC-0052). Otherwise false. It is the hero's own busy test of SPEC-0040 15.2, the same routine and not a wider copy: the 41 state tests, the script-held states 33, 48, 50, 51, 52, 53 and 55, the stun test and the screen-effect test are its four parts. The same test makes the world interaction chooser use its special-state branch (SPEC-0022). | H |
| 171 | IsHeroDead | hero selector | Query: true when the hero is in the death state (state 16). Always false for an index other than 0 or 1. Refined 2026-11-09 (High for the index rule): index 0 and 1 select the first and second hero objects themselves, whichever hero is selected, and the answer is the object's own "is dead" method returned unchanged; that the method is the state-16 test is not separately verified (Medium). | H |
| 172 | KillCurrentHero | none | Puts the active hero through the default death directly. The death callback gate is bypassed, so no callback runs, and the hero's health is left unchanged. See SPEC-0055 (hero death and respawn). | High |
| 173 | SwitchHero | hero selector, bool, T(bool) | Requests a switch to the given hero. Makes no request when that hero is already selected. The request is refused by the swap mode (row 167) or when the target hero is dead. The two booleans select the fade and flash sequence (first) and the finish cue (second); with both false the swap is silent. A request made while a swap is running overwrites the shared request record. On every call, whether or not a request was made (also for -1 or the already selected hero), the weapon-ready word of row 161 is then set to 0, with the cue-1 sound when it was set (High). See SPEC-0053, sections 3.2 to 3.5. | Medium-High |
| 174 | GetDistFromHeroToAnimatedLocator | int object h, int locator k | Returns the 3-D distance from the selected hero's position to the current position of h's locator k (formula A, Note on animated locators). Returns 0.0 when there is no current world, no object list, or h is out of range. An invalid k gives the distance to the world origin. | H |
| 175 | GetDistFromHeroToLocator | int locator k | Returns the 3-D distance from the selected hero's position to the +0x30 point of locator k. Returns 0.0 when k is out of range. | H |
| 176 | GetHeroAttackAlignment | none | Returns the active hero's equipped weapon Mote id from equipment word 2; returns -1 when that slot is empty (SPEC-0021, Equipment slots). | H |
| 177 | GetHeroHeightPatchType | none | Query: the type of ground patch under the hero. | L |
| 178 | NeedHeroAlive | hero selector | Revive check, not a requires-alive test. Only the selectors 0 (Tor) and 1 (Roberta) are accepted; -1 and any other value do nothing. When the named hero is in the death state, the revive helper runs. Its revive gate is the swap mode word: the helper acts only when the swap mode equals 2 (free both ways). It then revives the partner of the active hero, if that partner is dead, by setting the partner to idle at full health. Returns 1 when a revive happens, otherwise 0. The revive of a dead partner with a switch is a separate route in the hero action routine (SPEC-0053, section 3.8, route b). Modules 1244 (its entry scene) and 1394 (at area entry) call it for each dead hero. See SPEC-0036 (Revive by script) and SPEC-0055 (hero death and respawn). | High (gate); Medium-High (return value) |
| 179 | AlignAxisToVectorToHero | int, axis selector, float | Turns one column of the object's current block toward the hero. The columns are X, Y and Z (+0x100, +0x110, +0x120; SPEC-0028); selector 0 picks X, 1 picks Y and any other value Z (the disc passes only 2). Let u be the picked column normalised, d the unit vector from the object's translation (+0x130) to the hero's position (full 3-D, height included), and t the float clamped to 0 to 1. The call returns u . d, taken before the turn. When u . d < 1 - 0.001 t^2, the new column is normalise(t d + (1 - t) c), with c the stored column as it is; otherwise the new column is d itself. The other two columns are then rebuilt by cross products so the three stay orthonormal, keeping the old Y column's plane when X or Z is picked and the old Z column's plane when Y is picked. Returns 0 and changes nothing when the object is missing. Added 2026-11-09 (High): after the rebuild the three rows are scaled by the chosen column's original length and their w is set to 0; the blend uses the raw (not normalised) chosen column. | H |
| 180 | IsHeroSwapInProgress | none | Returns 1 from acceptance of a switch request until the swap task ends, else 0. It is not a raw swap value. See SPEC-0053, sections 2.3 and 3.2. | High |
| 181 | GetHeroShieldAlignment | none | Returns the active hero's equipped shield Mote id from equipment word 1; returns -1 when that slot is empty (SPEC-0021, Equipment slots). | H |
| 182 | FillBothHerosMagicPoints | none | Sets the magic points of both heroes to their maximum and refreshes the magic display. | H |
| 183 | GetHeroYPos | none | Query: the hero's height. | L |
| 184 | SetHeroStatus | int code, float s | Sends the status record (code, s, -1, -1) to the current hero's status-apply method (SPEC-0036, Status apply): status code, duration in seconds converted to a whole number toward zero (-1 = the status default), and -1 for the two extra values (poison or fire amount and period, so the defaults). Because it goes through the hero's own apply, the hero rules hold, immunities included: with status 8, 9 or 10 active, curse, poison or slow is refused and the block effect plays instead. Example: module 1091 calls SetHeroStatus(2, -1), poison for the default 120 s. | H |
| 185 | RunHero | int locator, WAIT, CTL | WalkHero (row 159) with the run flag set: the active hero runs to the locator record's position. With WAIT set, the caller yields each tick while the hero is in state 0x32 (50). CTL as in Layout section 2.1. | H |
| 186 | GetHeroSwapMode | none | Returns the current swap mode (0 to 5, as in row 167). See SPEC-0053, section 3.1. | High |
| 187 | ActivateHero | hero selector h, int locator, bool block, bool fade, bool cue | In this order, within the one call: (1) MoveHero(locator, probe on, CTL 1, face on) (row 156). MoveHero has no hero argument, so it acts on the hero that is selected when ActivateHero is called (not on h): that hero is placed at once on the locator's probed ground position with the locator's facing, and its state is set to 1. (2) SwitchHero(h, fade, cue) (row 173): a request only; none is made when h is -1 or already selected, and it can be refused (swap mode, dead target). If the swap runs, its placement step copies the outgoing hero's placement to the incoming hero (SPEC-0053, section 3.4 step 7), so h ends up on the locator (in swap mode 4, at its own stored spot instead). (3) When block is nonzero, BlockPlayerControl(h) (row 160) runs at once, before the swap task has run. Disc: 2 sites, both in module 1244: (1, 16, false, false, false) and (0, 12, false, false, false); the second follows SetHeroSwapMode(0). The return value is not used. | H |
| 188 | GetAxisDotVectorToHero | int, axis selector | Returns the dot product of the chosen stored direction vector of animated object i with the direction from the object's world position to the selected hero's position. Both vectors are normalised first. Selector 0 picks the object's world "left" vector, 1 its world "up" vector and any other value (2 on disc) its world "forward" vector (the same vectors as rows 298, 299 and 301). Returns 0 when the object is missing. Zero-length vectors are not guarded. | H |

**Category 9: Items, money and inventory**

| No. | Name | Arguments | Effect in plain words | Conf. |
| --- | --- | --- | --- | --- |
| 189 | DoesHeroHaveItem | int id | Returns the held count of id in the shared party inventory: the count of the first matching slot in the id's category list (SPEC-0021, Item slot lists, category-list query), or 0 when there is none. Scripts use it as a truth value. Both heroes share one inventory, so the selected hero does not matter. | H |
| 190 | GiveItemToHero | int id, int n, bool popup | With popup set, the item pickup popup for id is shown first, in its plain form: no first-find voice and no world item hook (SPEC-0021, Auto-collect step 6; Medium for the popup's look). Then the party add of SPEC-0021 runs with quantity n:<br>- n below 1 or an unknown id: nothing.<br>- Carry limit: a limit Q above 0 caps the held count at Q.<br>- Once-per-area items 19, 20, 151, 154 and 174: refused when already found in this world.<br>- Stat-up items: raise both heroes, play cue 0x9F and a voice, and are not stored.<br>- Coins: 135 adds n Neopoints; 136, 137, 138 and 139 add 5, 10, 25 and 100 whatever n is; cue 0xA5.<br>- 175: n is the map number (SPEC-0021, Treasure maps).<br>- Equipment (categories 1, 2 and 5): replaces the held piece only when the new id is higher.<br>- Category 9 fills both slots.<br>Disc: 57 sites; popup is set at 7 (Mote 160 with n 20 twice and 100 twice, Mote 170 with n 100, items 36 and 62). | M-H (popup), H (call path) |
| 191 | RemoveItemFromHero | int id, int n | Finds id with the slot query of SPEC-0021 (Item slot lists; only the first two slots for categories 1, 2, 5 and 9). When found: if n is below the held count, the count becomes count - n; otherwise the slot is emptied. Nothing when not found. A negative n would raise the count; the disc passes 1 (27 sites) and 5 (1 site). | H |
| 192 | GetNeopoints | none | Query: returns the shared Neopoints value (signed integer), or 0 when no wallet exists. Both heroes use one wallet (SPEC-0021, Wallet). | H |
| 193 | RemoveNeopoints | int | Subtracts the amount from the shared wallet when the amount is above 0, and floors the result at 0. It does not require the wallet to hold the full amount. Amounts of 0 or less do nothing. | H |
| 194 | RemoveAllOfItemFromHero | int id | Finds id with the slot query of SPEC-0021 (only the first two slots for categories 1, 2, 5 and 9) and empties that slot. The equipment words are not cleared. Nothing when not found. Disc: 43 sites. | H |
| 195 | GiveNeopoints | int | Adds the amount to the shared wallet when the amount is above 0. No upper limit is applied; a result below zero (a sum past 2,147,483,647) is stored as 0. Amounts of 0 or less do nothing. | H |
| 196 | DoesHeroHaveTreasureMap | int id | k = id - 200 when id is above 200, else id. True when 0 <= k < the map table's count and entry k's owned word is nonzero. A debug override word that would make it always true is 0 on disc and never written. | H |
| 197 | ItemSetPriceMultiplier | float f | Scales the buy price of items in the current world's spawned-item list. If the list is set, each record whose flag byte is 1 (a buy candidate) has its buy price p (signed 16-bit) replaced by trunc(f x p), capped at 32767; there is no lower cap. One pass over the list at the time of the call: records added later keep their price, and repeated calls compound. With no list, nothing happens. Disc: five level modules (1005, 1008, 1011, 1014 and 1020) each call it once with f = 4.0, from a control thread, while story flag 298 is off. See SPEC-0050 (section 5) and SPEC-0021. An earlier reading of the same field is Open question 70. | H (arithmetic and callers); M (field meaning) |
| 198 | ItemActivate | W, int slot, int ordinal, bool snap, T(bool) placed, VEC position, T(VEC) facing | World form. The first integer is a slot number of W (a SlotCreate result, Behavior section 5), not a region number. It must be 0 to the created-slot count - 1 and the slot's result word must be nonzero (a completed region read); otherwise the call returns 0 and spawns nothing, so a negative number spawns nothing. The placement with that ordinal among the slot's item placements (kind 8) is added by the SPEC-0021 rules, with snap and placed as in the note after this table, the default lifetime (-1), and the two vectors as position and facing overrides (none when 0). Returns the add's result. | H |
| 199 | GetPlayerPetPetType | none | Returns the kind of the live petpet (SPEC-0047, Script query): -1 when there is no petpet, when its remaining time T is below 1, or when its type id is not in the primary table; otherwise 0, 1, 3, 6, 7 or 8. | H |
| 200 | IsThatATatteredCloakYourWearing | none | True when the active hero's body entry (category-5 slot) holds item 126; see SPEC-0021 "Tattered Cloak". | H |
| 201 | AddItemToWorld | int placement, bool snap, bool placed, float lifetime | If the world's item table is set, adds the current region's placement with that ordinal as a scripted add, passing on the two bools (snap to ground, placed) and the float (the item's lifetime in seconds), with no position override and no facing override. Returns the add's result, or 0. The meaning of the bools and the float is in the note after the Category 21 table (High). This is a host record of its own, distinct from row 364; no module imports it. Its scripted-add reading (luck gate, vetoes, negative-key item roll) is in the note after the Category 21 table. | H |
| 202 | ItemActivate | int id, bool snap, T(bool) placed, VEC position, T(VEC) facing | Placement-id form, current world. The id's entry in the world's placement-id table gives a placement ordinal and a slot. A slot code of 0xFF means the world's own item table (the current region); any other slot must be a created slot of the world with a nonzero result word. An unknown id, an entry the lookup marks as invalid, or an unloaded slot returns 0 and spawns nothing. Otherwise the placement is added as in row 198 (default lifetime, the two vectors as overrides). See the note after the Category 21 table. | M-H |
| 203 | LockChest | W, int, int, int | Locks a chest. Stores the object id, the key item id and the message id in the first empty entry of the world's two-entry lock table (Behavior section 2.3). If both entries are full, nothing is stored. | M |
| 204 | ItemDeactivate | W, unsigned integer | Deactivates a world item. | L |

**Pet query (row 199, GetPlayerPetPetType).** Row 199 returns the kind of the live petpet: 0, 1, 3, 6, 7 or 8. It returns -1 when there is no live petpet, when its remaining time T is below 1, or when its type id is not in the primary table. Its inputs and rules are in SPEC-0047 (Petpets, Script query). The kind is not the pet type id, which is stored in the save slot (SPEC-0025) and maps to a kind through SPEC-0047's table.

**Category 10: Slots and asset groups**

| No. | Name | Arguments | Effect in plain words | Conf. |
| --- | --- | --- | --- | --- |
| 205 | SlotLoad | W, int slot, int j, bool keep, T(bool) activate | Loads character region pack + 1 + j into the slot. The region is the archive resource numbered pack + 1 + j; when the pack number is negative it is j + 1. A slot number outside 0 to n - 1, where n is the count given to SlotAllocate, does nothing: nothing is loaded or spawned, and the call returns 0 after one yield step. The slot number is passed to the call without change; no value is substituted for it. A load reads the slot number from a module data word. A word whose file value is -1 and that is never stored in reached code makes the load do nothing, and the call returns 0 after one yield step. That -1 is in the module's own data bytes, copied with the module image; no runtime value supplies it. Where the module stores the SlotCreate result into the word first (the usual pattern), the load uses that slot. Otherwise: (1) the slot's previous content is released first, on every load whatever keep is: a read still pending for the slot is cancelled; the result word is set to 0; the slot's six region lists are emptied (character records, item placements, checkpoints, animation tables, effect records and the fixup data); each of the slot's 15 character objects is released; three 15-entry lists on the selected hero are emptied (Open question 85); the counts are cleared; the character cap is set to 6 and the keep flag to 1. World items spawned by an earlier load are not removed. (2) keep is applied: a nonzero keep sets the keep flag, so the per-tick update runs the full streaming pass for the slot (it releases dead and far characters and binds the nearest eligible records up to the cap, SPEC-0006), and leaves the cap of 6 set by the release; a zero keep clears the keep flag, so the per-tick update only releases dead characters whose death countdown ended and binds nothing, and sets the character cap to 15. (3) The region number is recorded for the slot, the load starts and the slot becomes busy. (4) The call yields once per step while the slot is busy, with no timeout. (5) The call yields once more after that, even when the slot was not busy. (6) The call returns the slot's result: 1 when the read completes and the data starts with the IDM signature, and 0 otherwise (so 0 after a failed or missing read). The region's characters (kind-3 records, up to 40 read) are set up when the read completes, whatever activate is: one setup per record, using character object (record index mod 15), and the active count is the number of successful setups, at most 15. This is only the load-time warm-up: which record lives in which character object during play, and so the order the characters think in, comes from the streaming bind of SPEC-0006, which gives the nearest eligible records to the hero the first free objects (the Faerie Sisters' order in SPEC-0011 is an example). (7) When the result is nonzero, activate is nonzero and the slot number is below the number of slots created by SlotCreate, each of the region's item placements (kind-8 records, up to 120) goes through the region-load route of SPEC-0021 (Spawn routes): the luck gate, the vetoes for keys of 0 or more, then the flag test; flags 0 and 1 spawn, at the placement point, moved to the ground plus 0.5 when the placement's snap byte is set. Activate does not spawn characters. SlotLoad is the only call that reads a character region. The region is read into the buffer that SlotCreate reserved for that slot number, starting at the buffer's current position rounded up to a multiple of 128 bytes, with the rest of the buffer (end minus that position) as the available length. The region need not be one of those named in that slot's SlotCreate array; several level scripts load such a region, and every case on disc fits its buffer. A slot number below the capacity that never received a SlotCreate buffer has no buffer (start, position and end all zero), so its read has no target (Open question 28). Disc: activate is 1 at 380 of 384 sites; keep is 1 at 352. | High for the read target and the only-reader rule, the slot-number rule, the unchanged argument, the no-op for an out-of-range number, the wait and the extra yield, the keep flag and its effect on the per-tick update, the release, the result value, the character setup at read completion and the item-placement spawn; Medium-High for the data-word pattern (confirmed for the SPEC-0032 modules and by a count over all modules); Medium for the outcome of a read with no buffer and for the meaning of the selected hero's three lists |
| 206 | SlotAllocate | W, int n | Sets up n empty character slots for the world and sets the created-slot count to 0. Each slot starts with no region (region number -1). Returns 1 in every case. | H |
| 207 | SlotCreate | W, int n, int array | Loads nothing, and nothing loads for it later. Reserves one buffer from the world heap for the next slot number, raises the created count by one and returns that number. The array (n entries) is used only to size the buffer: for each entry j the call takes the archive size of region pack + 1 + j (its sector count times 2048, from the archive directory already in memory, without reading the region) and the number c_j of the world's entity-id entries whose region byte is j. With R the largest size and c the largest c_j, the buffer has R + 6388 x min(c, 15) + 20612 x c + 153 bytes. The per-c term 20612 is 9780 + 10064 + 2F with F = 384, the sum of two fixed pool sizes, 16 x 8 + 16 and 16 x 14 + 16. Test vectors: R = 300,000 and c = 4 give 408,153; c = 0 gives R + 153; R = 100,000 and c = 20 give 608,213. The region numbers in the array are not kept: no later step (world start, the per-tick pass, activation, or a following frame) loads them; only SlotLoad (row 205) reads a region into the buffer. The slot number is the number of SlotCreate calls already made in the world; the call returns -1 and reserves nothing when all slots given by SlotAllocate already have buffers. With an empty array (n = 0) the call still reserves a small base buffer and uses a slot number. With the count given to SlotAllocate equal to 1, the first SlotCreate in a world returns 0. | H |
| 208 | SlotReset | W, int slot | For a slot number 0 to n - 1: the release of row 205 step (1) (pending read cancelled, result 0, lists emptied, characters released, counts cleared, character cap 6, keep flag 1), then the region number is set to -1. The buffer is kept. A slot number outside 0 to n - 1, including -1, does nothing. Returns nothing. Because the release clears the result, SlotCharacterCount then reads 0 and the per-tick pass skips the slot (it updates only slots with a nonzero result). | H |
| 209 | SetWorldCollisMask | W, unsigned integer | Sets the world's collision layer mask: a plain store of the mask into the world record. | H |
| 210 | CharacterNavmeshAssign | int entity, int mesh | Entity lookup in the current world (Note on entity ids). The mesh set: when mesh equals the stored mesh id, nothing. Otherwise the id is stored; when it is 0 or more, the character's current triangle on that mesh is located again and its move goal is set to its current position. If no triangle is found, only a debug message follows (the character then counts as off-mesh). A negative id is stored and gives off-mesh steering (SPEC-0056). Nothing happens when the entity is missing. Disc: 59 sites. | H |
| 211 | CharacterActivate | W, int slot, int n | Activates character n (0 to 14) of the slot. The slot number must be 0 to W's slot capacity - 1; a negative number (so -1 as well) or a number at or above the capacity returns 0 at once, with no activation and no state change, as CharacterDeactivate does (row 213). Test vector: a slot word holding -1: CharacterActivate(slot, 0) returns 0 and no character becomes active. (High) Activating a character starts its default-class coroutine at once (SPEC-0029, Behavior 1); a boss controller therefore runs from the activation, not from a later release. | H |
| 212 | SetWorldDisplayMask | W, unsigned m | Stores 2 x m in W's display-mask word (0 at world setup). The word is the node-mask filter of the level display tree (together with the live story-layer bits, SPEC-0046 World layer mask) and, always (also when it is 0), of the world's placed-object mesh draw, without the story-layer bits: a node with nonzero layer flags is drawn only when it shares a bit with the filter, so with word 0 only nodes with flags 0 are drawn (SPEC-0028, Run-time object changes, Looks). The pack 1299 private tree ignores it. Disc values 1 and 2 give filters 2 and 4. | H |
| 213 | CharacterDeactivate | W, int slot, int n | The slot number must be 0 to W's slot capacity - 1; otherwise the call returns 0. It finds the slot's active character bound to record n and releases it by the release rule of SPEC-0006 (Release and rebind): active count lowered, position and facing written back unless its AI mode is 7, base state 42 unless dead, object freed. The record's kill bit is not set, so the slot's streaming may bind the record again. Returns 1 when a character was released and 0 when none is bound to n. | H |
| 214 | IsWorldOrCharacterLoadActive | none | Query: a world or character load is in progress, read from the loader state of the global loader record. | H |
| 215 | GetAssetGroupCharacterRange | int group, int output first, int output last | Writes the first and last index of the run of entries in the world's entity-id table (tag 0x12, SPEC-0006) whose slot byte equals group, compared as a signed byte (so -1 selects slot 0xFF). Its body is the run search, variant A (SPEC-0022, Record search by run). With no matching entry only last is written (count - 1), and first keeps the caller's value. Reached in 8 level modules, with keys 0, 1, 4, 6, 17, 21, -1 and computed values. The second run-search variant (on the placement-id table, tag 0x13) has no caller. Outputs only. | High |
| 216 | SlotCharacterCount | W, int | Query: the slot's active character count, at most 15: the number of successful character setups when the slot's last read completed (row 205, step 6). The release at the start of every SlotLoad and in SlotReset clears it, so it reads 0 after a failed read and after SlotReset. A slot number outside 0 to n - 1, including -1, returns 0. | H |
| 217 | SetWorldLoadOverRide | int slot | Sets the current world's pending slot (SPEC-0006, Pending slot writer) when the argument, taken as unsigned, is below 193, so a negative argument is ignored as well as one of 193 or more. On its next run the pending step uses the pending slot in place of the cell exit value under the hero, and then sets the pending word back to -1, whatever it decided (SPEC-0006). Fixed values at live sites: 131 (module 1184, three sites in one trigger), 165 (module 1391) and 861 (module 822). 861 is above 192, so that call has no effect. Pack 1427 has one more live site: a shared helper that each of its eight script-trigger callbacks calls with its own slot (187 to 191), which sets the pending slot only while a module once-word is clear and then sets that word; the module's Update clears it every tick (SPEC-0037, 1427). Init takes the eight callback addresses from a table in module data when it registers the triggers, so the helper is reached only through those registrations, never by a direct call; it is live in play whenever pack 1427 runs. | H |
| 218 | CharacterNavmeshAssign | W, int slot, int id, int mesh | Looks up character id in created slot `slot` of W. The slot must be below the created count and have a nonzero result; the match is the descriptor-id match of that slot's characters. When found, the mesh set of row 210 applies; otherwise nothing. Disc: 20 sites (slot-word forms, ids 0 to 7, meshes 0 to 13). | H |
| 219 | ActivateTransitionAssets | none | Despite its name this acts on the interior buffer I: when I holds a loaded world, it makes I the current world (the same make-current as a world change, SPEC-0006, called without the extra argument that the enter-interior job passes), and then, when the selected hero exists and has a movement controller, resets that controller. With no interior loaded it does nothing. The make-current runs even when no hero exists; only the controller reset needs the hero. Only pack 1122 calls it (SPEC-0043). | H |
| 220 | CharacterDeactivate | player character pointer | For a character that belongs to a slot (its slot pointer, character word +0xBC8, is set): its record ordinal is taken from its spawn record (+0xA44) within the slot's character records, the record's kill bit is set (SPEC-0006, Kill memory), so streaming does not bind it again, and the character is released as in row 213. Returns 0 when the pointer is 0 or the character has no slot. | H |
| 221 | CharacterSetAvoidPoint | W, entity id, table index | Looks up the character bound to the entity id; with none bound it does nothing. Otherwise it reads record n of the world's 0x50-byte record table (the type-11 table of locators; the index must be 0 to count - 1, else nothing) and stores its fourth float (offset 0x3C) as the character's avoid scalar r and its first three floats (offset 0x30) as the avoid point Q; the scalar is the fourth lane of the stored point as well. A class-17 enemy in the wander state uses the pair for the proximity retarget (SPEC-0038 Behavior 25); r stays 0 on every character otherwise, which turns the retarget off. One caller exists on disc: the Park Shrine pack applies index 33 (Q = (0.4049, 10.1515, 1043.5057), r = 9.0) to entities 16 to 21, but only before their region has loaded, so every call finds no character and has no effect in play. | H |
| 222 | CharacterIndex | player character pointer | Query: the entity id of a character, from its counter (character word +0xA40). Every character receives a counter at spawn, and the counter is its record ordinal in the list it was spawned from (the world pack's own character records, or the records of the region held in the pool slot), 0 to 49; no pool base is added at spawn (R-0223; High: the spawn routines pass the ordinal straight to the counter store, and the same ordinal is stored as the slot's record byte that entity lookups compare). Its body is the grouped lookup (SPEC-0022, Grouped record lookup): g = floor(counter / 50) - 1 and o = counter mod 50; g = -1 gives o; g at or beyond the number of pool slots in use gives -1; otherwise the result is the index of the first entity-id table entry (tag 0x12, SPEC-0006) whose slot byte equals pool key g and whose ordinal byte is o, or -1. Because every spawned counter is below 50, g is always -1 and the result is the ordinal itself; the table branch is never reached. Returns -1 at once when the character's id (+0x14C) is 0, 1, 100 or 101 (Open question 56). Test vector: the Evil Harris at ordinal 3 of region 1403 returns 3, and the pack 1398 trigger-34 routine uses entity 3 + 16 = 19. One level module calls it: pack 1398, in the routine of trigger 34 (SPEC-0043), which passes the triggering character, adds 16 to the result to form the entity id it then uses to read that character's position, and later deactivates the character. | High (Medium-High for the reason the ids 0, 1, 100 and 101 are excluded) |

**Category 11: Sound and music**

| No. | Name | Arguments | Effect in plain words | Conf. |
| --- | --- | --- | --- | --- |
| 223 | PlaySound | int cue, VEC emitter, bool noRestart | Plays cue on the area sound slot (SPEC-0031, Cue playback) with R = r0 = 0 (so radius 200), the default volume and one-shot mode. The emitter is a pointer kept by the voice and read on each update, so a voice started with an object's position pointer (row 26) follows the object. A null pointer means no emitter: no attenuation and the default pan. With noRestart set, nothing starts when a voice with the same slot, cue and emitter (any voice of that cue, when the emitter is null) is playing. Disc: 148 sites; noRestart is set at about 41; about 22 pass a pointer (15 animated-object pointers, 6 locator pointers) and the rest pass null. | H |
| 224 | AddAnimObjSound | W, int object, int cue, float a, float b | Each animated placed object of the world has 3 sound slots, all empty (cue -1) at world setup. When the world's sound slot is set and the object index is below the object count, the first empty slot of that object gets the cue, a start time max(a, 0.00001) x D and an end time min(b, 0.99999) x D, where D = (F - 1) / 15 s is the object's duration in seconds (F frames). With all 3 slots in use, nothing happens. Each tick an object advances in its states 0, 2, 5 or 6, the slots are checked against the interval from the old time to the new time (lo = the smaller, hi = the larger, after the end clamp and loop wrap): the first slot with lo <= start < hi plays its cue once, do-not-restart, on the area slot at the object's reference centre; the first slot with lo <= end <= hi stops that cue. At most one start and one stop per object per tick; a set-time call runs the same check. A start fraction of 1.0 never plays. Disc: 152 live sites in 18 modules; most pass (0, 1), which plays the sound for as long as the motion runs (SPEC-0031). | H |
| 225 | KillSound | int cue | Stops the first voice (in pool order) of the area slot that is active with that cue, whatever its emitter (SPEC-0031, same-voice test and stop). Only one voice is stopped per call. Disc: 39 sites. | H |
| 226 | RegisterMovableBlockSoundIndex | W, int | Registers the movable-block sound index: a plain store of the int at offset 0x2CCC of the world record. | H |
| 227 | PlayStingStream | stinger id, float volume | Stinger -1 plays nothing. Ids 0 to 18 play at volume x 1024, scaled by the music volume, only when the music system is running and no earlier stinger is still playing. Stinger 4 also plays only when at least 40 seconds of game time have passed since the last stinger 4 that played. Disc sites: (4, 1.0) x7, (4, 1.2) and (18, 1.0). | Medium-High |
| 228 | LoopSound | int cue, int locator k, bool noRestart | k must be in the current world's locator table; otherwise nothing. Starts cue on the area slot as a keep-alive voice, with locator k's point as its emitter. A keep-alive voice starts with a counter of 4, loses 1 each update and stops when the counter is below 1; each VisitLoopSound adds 1 (SPEC-0031). So the sound lasts about 4 updates unless it is refreshed every tick. noRestart works as in row 223. Disc: 18 sites, all with noRestart 0; module 1217 pairs each start with a VisitLoopSound loop. | H |
| 229 | KillSound | int cue, int locator k | k must be in the current world's locator table; otherwise nothing. Stops the first area-slot voice with that cue whose emitter is locator k's point. That is the pointer LoopSound uses and the one GetLocatorPositionPointer (row 17) returns. Disc: 18 sites. | H |
| 230 | SetMidiExciteLevel | int n | Stores n as the interactive music's excite level E when 0 <= n < 100 (the excite maximum); other values are ignored (SPEC-0009, Intensity inputs). The hero also writes E every tick, so a script value lasts until the next hero write. | H |
| 231 | OverRideMidiSettings | int bank, int level, int reverb type, int reverb volume | Stores the four values unchanged, with no check, in the music controller's override words: the bank offset override, which lasts until a call passes -1, and the level, reverb-type and reverb-volume overrides, which apply to the next controller tick only (SPEC-0009, Overrides). Disc: 6 live sites, two in one script trigger of each of modules 758, 1023 and 1358: (64, -1, -1, -1) on one branch, which gives bank 74, and (-1, -1, -1, -1) on the other. | H |
| 232 | VisitLoopSound | int cue, int locator k, bool | k must be in the current world's locator table; otherwise nothing. Finds the area-slot voice with that cue and locator k's point as its emitter and adds 1 to its keep-alive counter (row 228). Starts nothing, so the bool has no visible effect. | H |
| 233 | PlayMainSound | int cue, VEC emitter, bool noRestart | As row 223, on the main sound slot. | H |
| 234 | SetMidiAdventureLevel | float | Sets the adventure music level. | L |
| 235 | PlayHeroSound | hero selector, int cue, VEC emitter, bool noRestart | Plays only when the selector is -1 or equals the active hero's index; otherwise nothing is played. The cue is played on the active hero's own sound slot (not the area slot of row 223) through the same start routine as PlaySound, and the bool is passed in the same argument as PlaySound's noRestart (do not start when the same cue and emitter are already playing). With a null emitter the hero's position pointer is used. | M-H |

**Category 12: Timers, threads, time and random numbers**

| No. | Name | Arguments | Effect in plain words | Conf. |
| --- | --- | --- | --- | --- |
| 236 | ControlThreadsAllocate | W, int n | W must be non-null and have a script manager. Allocates from W's heap a table of n 16-byte entries (callback, argument, thread handle and a constructed sub-object) and stores n as the control-thread count. Every entry starts free. The entries are stepped in table order in step 4 of the per-tick script step (Behavior section 1). A second call replaces the table; threads started from the old table are no longer stepped (no disc module relies on this). Disc n: 1 (52 sites), 2 (31), 3 (13), 4 (9), 5 (4), 6 (3), 7 (2), 11, 12, and four computed values. | H |
| 237 | ControlThreadStart | W, callback taking a raw pointer, raw pointer | W must be non-null (otherwise only a debug message follows) and have a script manager. Takes the first entry of W's control table whose thread handle is empty, stores the callback and the argument, and creates a thread that runs callback(argument) and then ends. The thread is first stepped in step 4 of the per-tick script step (section 1). When no entry is free, or ControlThreadsAllocate was never called, nothing starts (only a debug message follows). The call returns no value: the slot index is not returned, and none of the 281 sites reads a result. An entry becomes free on the tick its thread ends: step 4 frees an ended thread's handle as it steps it. | H |
| 238 | Sleep | int | Yields n ticks. n of zero or less does nothing. | H |
| 239 | Wait | float s | Yields n = trunc(s x 60) ticks, computed in single precision (section 4, Seconds to ticks, with disc examples). n <= 0 returns at once without yielding. | H |
| 240 | Random | int a, int b | Returns a uniformly drawn integer in [min(a, b), max(a, b)], both ends included: low + (r mod (high - low + 1)), with r an unsigned 32-bit Mersenne-Twister (MT19937) output. Equal bounds return the bound without drawing. The generator seeds itself from the console clock on first use, so the sequence is not reproducible; a separate reseed helper has no direct caller. | H |
| 241 | GetNumberOfFramesPerSecond | none | Returns the tick-rate word (60.0, written at start-up; SPEC-0007, Rate selection), the rate that Sleep and Wait use. | H |
| 242 | memset | (C library) | Fills a block of memory. It is the first host record (record 1, after the placeholder) and has no game-state effect. | H |
| 243 | GetSystemTime | none | Returns the game clock in seconds: the sum of the tick step (1/60 s) over all ticks since start-up. This is a runtime value, not a constant on the disc. It is the same game clock that the hurting-surface rule of SPEC-0057 uses (one hit every 2.5 s of this clock). | H |
| 244 | ActivateTimer | float start, float limit, int direction, RGBA colour pointer | As SPEC-0023 section 31, Activate: the timer becomes visible and running, with value = start. A null colour uses the default colour word. | H |
| 245 | DeactivateTimer | none | Turns the timer's visible flag off (SPEC-0023 section 31). | H |
| 246 | GetTimer | none | Returns the countdown timer's value in seconds, a float (SPEC-0023 section 31). | H |
| 247 | SetTimerColor | RGBA colour pointer | Sets the timer colour to the RGBA word at the pointer; a null pointer gives the default colour word at 0x2DD040 (128, 128, 128, 128). The colour is pushed to the timer element on each update while the timer is visible. | H |
| 248 | StopTimer | none | Running off: the value is neither stepped nor clamped; visibility and the value are kept (SPEC-0023 section 31). | H |
| 249 | StartTimer | none | Running on; visibility and the value are kept (SPEC-0023 section 31). | H |

**Category 13: Triggers and callbacks**

| No. | Name | Arguments | Effect in plain words | Conf. |
| --- | --- | --- | --- | --- |
| 250 | RegisterScriptTrigger | W, callback taking the collision record pointer, int | Stores the callback under the trigger index in the world's trigger table; the callback runs when a volume carrying that index fires. The table has n entries, n = the world's trigger capacity (Behavior section 3); an index outside 0 to n - 1 is ignored. Every resolved index on the disc is in range. Registering an index again replaces the stored callback, so the later routine is kept (Layout section 3; Medium-High). The enter test is a per-tick overlap test, and the fire step writes the state word (section 3; Layout section 3). | H (registration and capacity), M (enter test, partly) |
| 251 | RegisterEntityProximityCallback | W, int entity, callback taking the AI-proximity record pointer, bool | Registers the callback in the world's proximity-callback table at the given index; an index outside the table is ignored. The table has one entry per character descriptor of the world, and the index is the descriptor id that IsAI_Dead and the entity calls use. Each entry holds the callback, the descriptor index and a range flag. This table is the talk table of SPEC-0022 (Talk entry): it is read by the hero's world interaction chooser, not by the 400-unit proximity test of section 3. The approach radius is two-staged: the entity must be the hero's talk candidate (talker list, within 20.0 units and facing cosine at least 0.5, unless a nearer lock-on leader takes the place), and with the range flag set the "Talk" caption (label 8) shows while the 3-D distance is below 5.8 units; a Square press in that tick starts the callback as a script thread with the record (descriptor index, distance, flag). With the range flag clear the callback starts with no press and no range test; every registration on the disc sets the flag. While the thread runs no new start happens. A registered candidate blocks the chooser's later idle interactions (door confirm, book, Feed petpet and others) even out of range. Any item given on approach comes from the callback's own body; some level callbacks give items behind flag tests or a prompt answer. | H |
| 252 | RegisterActivateWorldCallback | callback taking a world record pointer, W | One callback per world; a later call replaces it. Each time W becomes current, the callback is started as a thread, unless it is still running, with W's slot number as its argument. It therefore runs after the arrival placement of that load (section 6, Init order). | H |
| 253 | RegisterTeleportCallback | callback taking the collision record pointer, int, context message, W | Binds the callback and its prompt message to door number n of that world (world 0 means the current world). The callback replaces the default door rule for that door: when the hero confirms the prompt at the door, the callback runs as a script thread and the default rule is skipped (SPEC-0006, doors). | H |
| 254 | WaitTillLeaveTrigger | collision record pointer | Yields each tick while the state word is not 0 and not 3 (section 3). A record last fired on frame s reads 3 from the decay of frame s + 2, so a callback waiting on its own record returns on the resume of frame s + 3. Against an object-0 firer that stays inside, it returns after the first decay to 3, although the firer has not left. | H |
| 255 | RegisterOnEnterCallback | callback taking the AI-proximity record pointer, W, int v | One callback per world; a later call replaces it. Stores the callback with the record (v, 0, 0). Each time W becomes current, the callback is started as a thread with a pointer to that record, unless it is still running, right after the activate-world callback (row 252). With W 0 it only shows a debug message. | H |
| 256 | GetScriptGroupInstance | int i, W | Uses W, or the current world when W is 0. Returns a pointer to entry i of the world's script-group list (the type-40 chunk, 0x44-byte entries; SPEC-0028 section 11). Returns 0 when there is no list or i is outside 0 to count - 1. Scripts read its locator, trigger, object and camera lists. | H |
| 257 | RegisterItemAskAboutCallback | W, int item, callback taking the item-proximity record pointer, bool prompt | Writes (callback, item, prompt flag) into W's item ask-about table (10 records): into the record that already holds that item id, else into the first free record (a free record holds id -1). The search for an existing record accepts only item ids below the table size, 10. Nothing is stored when the table is full or W is 0. The firing rule is in Behavior section 3 (Item ask-about). It never fires on disc: the search at fire time has the same below-10 rule, and all 27 registrations name ids 57 to 174, so no callback, caption or collect block ever happens (an original quirk; Edge cases). | H |
| 258 | GetCollisInfo | int trigger | Returns a pointer to the collision block (Layout section 3) of the current world's trigger record for that index, or 0 when the index is out of range. Scripts poll its state word. | H |
| 259 | RegisterAIPlayerActivateCallback | W, callback taking an integer and returning a bool | With W null: a debug print and nothing else. Otherwise the callback becomes W's activation filter. It is stored in W and passed on to W's embedded character block and to every slot created so far. Each block keeps it as its filter, and the streaming eligibility test calls it with the record ordinal before it binds a record (SPEC-0006, activation filter). A slot created after the call has no filter. Registered from Init in modules 735, 985, 1056, 1096, 1217, 1428 and 1432: 10 live sites, because 1217 and 1428 each choose one of two callbacks by a module test. Example: module 985's callback admits ordinals 5, 7 and 8 and refuses 0 to 4 and 6 when story flag 281 is clear, and the reverse set when it is set; it also assigns a walk mesh to entity 7 or to entities 0 to 4 as a side effect of being asked (SPEC-0041, 985). | H |
| 260 | RegisterHeroBuyItemCallback | W, callback taking the buy-item record pointer, bool | Stores the callback and the flag in W's script record; one buy callback per world, and a later call replaces it. With W 0 it only shows a debug message. When the hero buys an item in the current world and a callback is stored, the buy is not done at once: the callback starts as a thread (unless the previous one is still running) with a record that holds the item id at +0x00 and a flag word at +0x08, copied from the stored flag. When the callback returns with the flag word 0, the buy runs then, without the callback; with a nonzero word it is skipped. A callback can change the word to decide. | M-H |
| 261 | RegisterScriptInstanceTrigger | W, callback taking the collision record pointer, int t, int i | i is an entry of W's script-group list (row 256); nothing happens when there is no entry i. When the entry's kind word is 7, every member trigger index of the entry is registered and t is ignored; each member record gets owner value i and its word set to 1. Otherwise trigger index t is registered when it is 0 to the world's trigger capacity - 1 (else nothing), with owner value i and word 0. Registering writes the callback, the owner value, the word and the index into the trigger record (Layout section 3), so a later registration of the same index replaces it. With W 0 it only shows a debug message. | H (registration); M (use of the owner value) |
| 262 | GetScriptGroupMainTrigger | int i, W | For entry i (row 256): when its trigger count is above 0, returns GetCollisInfo of its first trigger index, that is a pointer to that trigger's collision block in the current world (row 258), even when W names another world. Returns 0 when there is no entry or no trigger. It is not an id. | H |
| 263 | RegisterTransitionMap | int | Stores a map-display identifier that the area map and location display use when the level's own map identifier is -1. It loads nothing and moves no one; the map page has no travel (SPEC-0051). Values on disc run from 5 to 73; their mapping to map pages is not given here. | H (no load); M (use); L (values) |
| 264 | RegisterHeroDeathCallback | W, callback taking the hero-death record pointer, bool | Stores a death callback and its flag on the area object. The callback body starts once per death, not once per area object: the stored task handle is cleared when the callback task finishes, so the next death starts it again (only while the area's world is the current world). A death while the task still runs starts nothing; with flag 0 the default death still runs. The flag is read on every death: 1 skips the default death for every later death in that area object, and 0 lets the default death run. Registering again does not start a second callback while the first is still running. See SPEC-0055 (hero death and respawn). | Medium-High |
| 265 | RegisterHeroPickupItemCallback | W, callback taking the pickup-item record pointer, bool | Stores the callback and the flag in W's script record; one pickup callback per world, and a later call replaces it. With W 0 it only shows a debug message. When the hero picks up an item in the current world and a callback is stored, the pickup is not applied at once: the callback starts as a thread (unless the previous one is still running) with a record that holds the item id at +0x00 and a flag word at +0x08, copied from the stored flag. When the callback returns with the flag word 0, the pickup is applied then, without the callback; with a nonzero word it is skipped. A callback can change the word to decide. Five packs register one on disc, all with flag 0: 816 (the callback tests item 76), 893 (item 77), 906 (item 24), 1096 (item 117) and 1238 (item 83); each sets a game or level state and never changes the word. | M-H |
| 266 | RegisterScriptAnimatedTrigger | W, callback taking the collision record pointer, int g, int k | Registers record k of W's animated trigger group g. The world keeps a list of such groups, each with a count and its own 0x48-byte trigger records. g and k must be in range; otherwise nothing. The record gets the callback, owner value -1, word 0 and index k, as in row 250 but in the group's own table. With W 0 it only shows a debug message. | H (registration); M (which object each group belongs to) |
| 267 | RegisterScriptBreakable | W, callback taking the collision record pointer, int b, int id | Registers record b of W's breakable trigger table, a second table of 0x48-byte trigger records kept beside the plain trigger table. b must be 0 to its count - 1; otherwise nothing. The record gets the callback, owner value id (the group id), word 0 and index b. With W 0 it only shows a debug message. | H (registration); M (use of the owner value) |
| 268 | RegisterScriptWorldSwap | W, callback taking a raw pointer | Stores the world-swap scene callback that StartTransitionScene (row 283) runs. Unused in play, because no live StartTransitionScene call exists. | H (store); M-H (use) |
| 269 | RegisterHeroUseItemCallback | W, callback taking the use-item record pointer, bool | Stores the callback and the flag in W's script record, in the same way as rows 260 and 265 (one per world; a later call replaces it; W 0 only shows a debug message). When the hero's use step asks for it and a callback is stored, the default use is skipped and the callback starts as a thread with the flag copied into its record (+0x08); when it returns with that word 0, the hero's use step runs then. | M (the use path) |
| 270 | DeactivateScriptGroup | int i, W | For entry i (row 256): each member trigger index is registered with no callback in the entry's world, so it no longer starts a script. Each member placed object is deactivated (the row 4 rule). Each member breakable's active word is cleared. Locators, cameras and effects are not touched. | H |
| 271 | ReActivateScriptGroup | int i, W | For entry i (row 256) with its world reference set: re-runs the built-in set-up of its kind: 1 lock chest, 2 sealed chest, 3 proximity fuse, 4 chain, 5 reaction object, 6 dig site (SPEC-0048, SPEC-0051). Kinds 0, 7 and 12 do nothing and return 0. Kinds 1–6 return 0 or 1 under their own checks: kind 1 requires a positive configured list count; kind 2 requires both linked object lookups to succeed; kind 3 requires at least one eligible entry; kinds 4 and 5 require a nonempty configured list; kind 6 requires a configured count greater than two. Missing shared world/group state returns 0. These are per-kind conditions, not one common setup-success result. | H |
| 272 | HasHitEntity | int | Entity lookup in the current world; the id used is the entity's target id (+0x14C), or -1 when the entity is missing. True when the selected hero's melee hit list (ten ids at +0x400, SPEC-0040) holds that id; false when there is no hero. Quirk: a missing entity gives -1, which matches an empty slot, so the call is then true whenever the hero's list has an empty slot. | H |
| 273 | RegisterScriptWorldSwapActivated | W, callback taking a raw pointer | Stores the "activated" callback that runs when the exterior becomes current at the end of a StartTransitionScene sequence (row 283). Unused in play. | H (store); M-H (use) |
| 274 | RegisterScriptGroupType | W, int type, callback taking the collision record pointer, and a callback taking the script-group init record pointer | Sets the two callbacks (run and initialize) for a script group type. Added 2026-11-11 (High): the world loader itself registers script group type 3 whenever the pack has a type-40 chunk, with two fixed built-in callbacks (the second, the init callback, is the group start at a marker); no level module has to call this for type 3. | M; H for the loader's own type 3 registration |
| 275 | ForceTriggerScriptIfNotRunning | int | Runs the trigger script with that index unless it is already running. The fire passes object 0 (Medium). The only call on disc is in pack 1246's Update, gated on level state 1, which nothing in that world or in any creature of it ever sets, so the call never runs in play and a build may omit it (SPEC-0039, section 1246; Medium-High). | L |
| 276 | CollideEntityWithTriggers | int entity | When the entity has a motion record, runs the trigger collision step of its cell list at its current position (flag 1). The per-volume rule is in section 3. Nothing happens when there is no current world, the entity is missing, or it has no motion record. | H |

**Category 14: World and area transitions**

| No. | Name | Arguments | Effect in plain words | Conf. |
| --- | --- | --- | --- | --- |
| 277 | StartExteriorReplacementLoad | eW, int plane index p, RGB colour pointer | Starts loading an exterior world in place of the current one, with a fade colour. p is stored unchanged in the exterior slot's load request and decides where the heroes land after the load (section 6): p >= 0 lands on teleport plane p of the loaded world; p = -1 puts the heroes at entry record 0; p <= -2 keeps the hero's current position and facing. The call works only while the exterior world is current; otherwise it shows an error message and loads nothing. Every script call sets the "unload all" option, so all world buffers are unloaded on the next frame before the pack is loaded into the exterior buffer (SPEC-0006). Disc: of the 98 live exterior loads in level modules (rows 277 and 284), all from an exterior to an exterior, 86 pass -1, 11 pass 0 and 1 passes 2; two more calls, in unreachable self-reload helpers of packs 1176 and 1371, pass 0. | H (placement, exterior-only rule, counts); M (load) |
| 278 | StartLeaveInterior | eW, int plane a, int entry b, RGB colour pointer | Starts leaving an interior and returning to the exterior world. The call works only while the interior world is current and an exterior world exists; otherwise it shows an error message and does nothing. It is also ignored, with no message, while an earlier enter-interior or leave-interior request still has its transition thread running on the exterior world's script manager (High). Its world slot names the transition pack to reload into the transition buffer after the exterior becomes current (or nothing when -1), not the destination: the destination is always the resident exterior. All 21 live calls on disc run from interiors and name a transition. After the exterior is current the hero lands on exterior teleport plane a when 0 <= a < the exterior's teleport plane count; otherwise at exterior entry record b, where b is replaced by 0 when b < 0 or b >= the entry count of the INTERIOR slot (quirk: the bound comes from the interior, the record from the exterior's list; reproduce). Plane landings are probed; entry landings are not (section 6). The 19 calls whose plane and entry arguments are resolved pass (-1, -1), so they land on exterior entry 0. | H (placement, interior-only rule, the slot meaning); M (load) |
| 279 | StartEnterInterior | eW, int plane a, int entry b, RGB colour pointer | Starts entering the given interior world. The call works only while the exterior world is current; otherwise it shows an error message and loads nothing. While an earlier enter-interior or leave-interior request still has its transition thread running on the exterior world's script manager, the call is ignored with no message (High). (On disc, pack 706's door callback calls it from a transition world, where it is refused). Entering unloads the transition buffer, loads the interior and remembers the transition pack that was resident (SPEC-0006). Once it is loaded and current the hero lands on interior teleport plane a when a is valid, otherwise at interior entry record b (0 when b is out of range of the interior's own count); the landing height is always probed from y + 6.0 (section 6). The 36 live call sites (44 destinations, all interiors, called from exteriors) all pass plane -1; the 28 whose entry argument is resolved pass entry -1, so they land on interior entry 0. | H (placement, exterior-only rule); M (load) |
| 280 | StartHeroLongFall | none | Sets the active hero to state 48 (the long-fall state), used for a drop into a sinkhole. It is not a death: health is not changed and no death rules run. See SPEC-0055 (hero death and respawn). | High (state); Medium (animation) |
| 281 | SetHeroStartPos | int | Copies the position (record offset 0x30) and facing (offset 0x40) of entry record i of the current world's entry list (0x50-byte records) into the start pose of both heroes (SPEC-0040, Behavior 1C). It never moves anyone, at once or at a later load. An index out of range does nothing. Every path that reads the start pose writes it first (SPEC-0055, section 8), so the call has no visible effect on position in this build (Medium-High). The four routines that respawn a hero (the new-game start, the world-load placement, the saved-game apply and the deadly-ground timer) each store the start pose immediately before the respawn; the timer does so only for an in-range entry index, and all 79 disc indices are in range. Calls in modules 777 (entry 8, in Init) and 1246 (entry 11, after the move to locator 11) leave position unchanged either way, so a rewrite may omit the call or store the pose without changing any play. | High (store); Medium-High (no visible effect) |
| 282 | GetLastActiveWorld | none | Returns the previous active area-slot number, the value retained when the game last made a world current. It reads the previous-slot field; see SPEC-0006, Prior and active world slots. | H |
| 283 | StartTransitionScene | eW | Stores the named pack on the transition buffer's loader. When that loader has a world-swap scene callback (row 268; registered by packs 797, 908, 1055 and 1254), the transition world becomes current, the pack streams into the exterior buffer, the scene runs, then the exterior becomes current and its activated callback (row 273) runs. The only two calls on disc (pack 758) are in code that nothing reaches, so the call is unused in play. | H (store, dead callers); M-H (scene sequence) |
| 284 | StartExteriorReplacementLoadWithoutFade | eW, int plane index p | Same as 277 without the fade; p has the same meaning. | H (placement); L (rest) |
| 285 | IsTransitionLoading | none | Returns the busy word of the transition world slot: 1 from the start of that slot's pack read until the read completes (before the pack is parsed), else 0. Returns 0 when there is no transition slot. | H |

**Category 15: Host names outside the first group (rows 286 to 294)**

| No. | Name | Arguments | Effect in plain words | Conf. |
| --- | --- | --- | --- | --- |
| 286 | Draik_GetLeftLegIdx (and eight sibling queries: Draik_GetRightLegIdx, Draik_GetTailIdx, Draik_GetChestIdx, Draik_GetLeftArmIdx, Draik_GetRightArmIdx, Draik_GetLeftWingIdx, Draik_GetRightWingIdx, Draik_GetHeadIdx) | none | Each returns one of the nine stored breakable ids set by row 78. | H |
| 287 | CharacterDistance | W, int slot, int id | Returns the 3-D distance from the character to the camera point (the camera object's +0x30 point, the same point as SPEC-0038 Behavior 33). It does not measure to the hero. Returns positive infinity when the character is missing. The world argument is read: it supplies the slot list that the slot number indexes (a slot outside the list or without a live record gives the missing result), whereas row 288 uses the current world. No module calls it. | H |
| 288 | CharacterDistance | int entity | As row 287, with the entity lookup (Note on entity ids): the 3-D distance from the character to the camera point, positive infinity when the entity is missing. No module calls it. | H |
| 289 | KnockbackHero | float (not used) | Calls the current hero's knockback method, which ignores the declared float (the method does not read it). The method requests hero animation 64 and makes one sound-service call on the hero's sound word with zero parameters, whose effect is not identified (Medium). Its body has no velocity, position or damage call. No module calls it. | H (float unused); Medium (the rest) |
| 290 | DoAIConversation | int entity A, int animation A, int entity B, int animation B | For each pair, the entity is looked up in the current world (a missing entity counts as finished at once); its script-flag word is set to 1, the animation is requested and the entity's base state is set to 33 (script hold). Then, each tick (the first check comes after one tick), each entity whose stored current animation id no longer equals its requested animation (the one-shot ended and the animation player moved on; row 305) is returned to base state 1 (free); entities finish independently. The call returns once both have been released. | M-H |
| 291 | DFS_Act1_IsTriggerHit | none | Always false in this build. No level module imports it: only pack 1174 uses the act-1 routines, and it imports the set-indices, set-trigger-data and reset-triggers calls only. | H |
| 292 | GetFrameCount | none | Returns the frame counter, which rises by 1 at the end of every main-loop iteration. This is a runtime value, not a constant on the disc. | H |
| 293 | GetTimeStep | none | Returns the engine's time step per tick: 1/60 s, written once at start-up from the boot rate table (SPEC-0007, Rate selection); it is not a constant on the disc. | M |
| 294 | IsAnimatedCameraActive | int track | True when a scripted camera play is active and its track record is the record at index track of the world's camera track list; false for an index outside the list, an empty list entry, or when no play is active. | M-H |

**Category 16: Animated object pose, bounds and playback**

| No. | Name | Arguments | Effect in plain words | Conf. |
| --- | --- | --- | --- | --- |
| 295 | GetEntityPosAndDir | int, float outputs x6 | Writes the entity's position (three floats) and its direction (three floats) into the six outputs. Writes nothing when the entity is missing. Returns nothing; outputs only. | M |
| 296 | GetAnimObjBoundingBoxCorner | int, float outputs x3 | Writes one corner of the bounding box of the animated object with the given index into the three outputs. The int selects the object (Edge cases); no corner index is used. Outputs only. | M |
| 297 | GetAnimObjBoundingBoxSize | int, float outputs x3 | Writes the bounding-box size vector of the animated object with the given index into the outputs. The int selects the object as in row 296. Outputs only. | M |
| 298 | GetAnimObjWorldForward | int, float outputs x3 | Writes the world-space forward vector of the animated object with the given index into the outputs. The int selects the object as in row 296. Outputs only. | M |
| 299 | GetAnimObjWorldLeft | int, float outputs x3 | Writes the world-space left vector of the animated object with the given index into the outputs. The int selects the object as in row 296. Outputs only. | M |
| 300 | GetAnimObjWorldStartPos | int, float outputs x3 | Writes the world-space start position of the animated object with the given index into the outputs. The int selects the object as in row 296. Outputs only. | M |
| 301 | GetAnimObjWorldUp | int, float outputs x3 | Writes the world-space up vector of the animated object with the given index into the outputs. The int selects the object as in row 296. Outputs only. | M |
| 302 | GetVisualCenterPos | int, float outputs x3 | Writes the visual centre point of the animated object with the given index into the outputs. The int selects the object as in row 296. Outputs only. | M |
| 303 | SetAnimObjAdditionRotXZ | int object, float a, float b | Sets the object's additional rotation to a rotation about a horizontal axis chosen by the first float: the axis is (cos a, 0, sin a) with a taken as a plain radian value (no degree conversion), and the rotation angle is the second float b, in degrees as in row 23 (unit quaternion (axis x sin(b/2), cos(b/2)), replacing any earlier additional rotation). It is not one rotation per axis. No module calls it, so play never reaches it (Open question 53). | M-H |
| 304 | RestartAnim | int animated-object index | Takes an animated object of the current world (not an entity), by object index; an out-of-range index or an absent object does nothing. It only sets the object's playback state to "playing once"; it does not change the object's time or speed. An object that is stopped or looping therefore continues from its current time and ends at the clip end; an object already held at its end (finished) is returned to finished on the next update, so no visible replay occurs. Use PlayAnim (which also sets the speed and clears the extra vector) to start a clip. | M-H |
| 305 | IsEntityPlayingAnim | int entity, int animation id | Query: true when the entity (looked up by id in the current world) has the given animation id as its current requested animation id; false when the entity is missing. | H |
| 306 | SetAnimObjAdditionRotX | int, float | Same as row 23 with the axis (1, 0, 0): the angle is in degrees, the rotation is the half-angle unit quaternion about the X axis, and it replaces any earlier additional rotation. No module calls it, so play never reaches it (Open question 53). | M-H |
| 307 | SetAnimObjAdditionRotObjX | int, float | Same as row 23, but the axis is the first row of the animated object's own current transform (its local X direction at the time of the call), with the angle in degrees; nothing happens when the object does not exist. No module calls it, so play never reaches it (Open question 53). | M-H |
| 308 | SetAnimObjAdditionRotObjY | int, float | As row 307 with the second row of the object's transform (its local Y direction). No module calls it (Open question 53). | M-H |
| 309 | SetAnimObjAdditionRotObjZ | int, float | As row 307 with the third row of the object's transform (its local Z direction). No module calls it (Open question 53). | M-H |
| 310 | IsEntityCurrentAnimDone | int | Query: true when the progress of the entity's current animation is at least 0.99 (the clip with the entity's current animation id among the first four clip slots of its animation layer). False when the entity is missing, when the clip is not found in those slots, or when its skeleton has no animation table (then a debug line is also printed). | M-H |
| 311 | GetObjNumFrames | int | Returns the frame count of the animated object with the given index as a 16-bit value, or 0xFFFF (the 16-bit form of -1) when there is no object. The int selects the object (Edge cases). | H |
| 312 | GetAnimID | animated-object pointer | Returns the animation id stored on an animated object, or -1 when the pointer is null. | H |

**Category 17: Camera control**

| No. | Name | Arguments | Effect in plain words | Conf. |
| --- | --- | --- | --- | --- |
| 313 | SetAiHotspotDistCamera | int entity, int slot, float distance, float angle (radians), float (ignored) | Sets the camera for an entity from a hotspot. When the second int is below nine it selects one of nine stored per-entity offsets; otherwise a default offset is used. The float argument gives a direction from an angle in radians. The entity id is the first integer argument and the slot selector is the second integer; the float arguments arrive in their written order: the first float is the distance by which the direction is scaled, the second float is the angle in radians (a plain radian value, no degree conversion) whose sine and cosine build the horizontal direction, and the third float is never used (Medium-High; no module calls this row, so play never reaches it; Open question 59). Established 2026-11-09 (High): the character is looked up (nothing if none). The position and forward are, for a slot below 9, taken from rows 2 and 3 of that slot's attach matrix in the character's model object, and otherwise the character's position plus up x 3.0 with the body forward. The forward is flattened (y = 0) and normalised, side = unit(up x forward), and the direction = -(forward x cos(angle) + side x sin(angle)). The scripted pose rows are (up x forward unnormalised, w 0), up, direction (w 0), and the eye = position - direction x distance (w 1); the pose starts with a field of view of 1.5707964. | H |
| 314 | SetCameraToTrackHeroMode | int | Puts the camera into hero-track mode, using a camera track record chosen by the int. The camera is placed at the record's point (the first key of the track when it has keys), aimed at the selected hero, and set to camera mode 12 (SPEC-0027). Does nothing when the index is negative, out of range or the record is absent. No module calls it. | H |
| 315 | SetAI_DistCamera | int entity id, float, float, float | Enters the fallback camera pose on the entity: the first float is the horizontal reach R, the second the yaw offset o and the third the height h; the distance is sqrt(R^2 + h^2) and the elevation asin(h / R) (SPEC-0027, Fallback pose). Does nothing when the entity is missing or its region is not loaded. No module calls it (Open question 51). | M-H |
| 316 | OverRideCameraAttributes | float distance, float elevation, float minimum height | Turns on the camera attribute override and stores the three floats as the follow distance, the default elevation (radians) and the minimum eye height above the ground of the script profile (profile 4 of SPEC-0027, Camera profile record). While the override is on, the follow camera uses that profile instead of the ground-kind profile; the profile fields are reloaded only when the selected profile index changes (Open question 51). No module calls it. | H |
| 317 | EndCameraTrackHeroMode | none | Sets camera mode 0 (normal follow) only when the camera is in mode 12; does nothing otherwise. | H |
| 318 | PlayTrackAnimObjCamera | int track, int object, WAIT, CAMW end mode, float speed | As row 38, but with the object's track anchor (object offset 0x180) instead of its look-at anchor. No module imports it. | H |
| 319 | SetHeroDistCamera | float, float, float | Enters the fallback camera pose on the selected hero. The three floats are passed on unchanged as the horizontal reach R, the height h and the yaw offset o (SPEC-0027, Fallback pose); this order differs from row 315, which takes reach, yaw offset, height. No module calls it (Open question 51). | M-H |
| 320 | RestoreCameraAttributes | none | Turns off the camera attribute override that OverRideCameraAttributes set. On the next follow tick with a ground patch under the focus, the camera selects the ground-kind profile again and reloads it. The numbers written by the override stay in the profile table. | H |

**Category 18: Effects, sound and vibration**

| No. | Name | Arguments | Effect in plain words | Conf. |
| --- | --- | --- | --- | --- |
| 321 | MoveEffectToCharPosition | int, int, float, float, float | Moves an effect to a character's position plus an offset given by the three floats. Upgraded 2026-11-09 (High): the first int is an entity slot (missing entity: nothing) and the second the script effect index; the floats are height, forward distance and side distance. The new position = the character's position + body-forward x forward distance + (0, height, 0) + unit(forward x world up) x side distance; the effect is moved there with the forward vector as its direction (as MoveEffect, row 324). | H |
| 322 | PlayHeroShieldSound | hero selector, cue id, VEC, do-not-restart bool | When the selector is -1 or matches the active hero, plays the cue id on the active hero's shield sound slot. A null VEC uses the active hero's position. The bool is the cue call's do-not-restart flag (SPEC-0031, Cue playback). | H |
| 323 | PlayHeroWeaponSound | hero selector, cue id, VEC, do-not-restart bool | Same as row 322, using the active hero's weapon sound slot. | H |
| 324 | MoveEffect | int, float x6 | Moves the effect given by the int to a new position. The six floats form two three-vectors that the body builds into the effect's start and end points. The pairing is partly established. | L |
| 325 | VibrateController | int preset, float duration, float magnitude | Sends a rumble request with the given preset (0 to 4; 5 or more does nothing), duration in seconds and magnitude, by the same rule as every other rumble request (SPEC-0033, Rumble). The host call exists, but no level module calls it. | H |
| 326 | SetMusic | int | Starts the music track with the given id, with the loop flag set. | M |
| 327 | UnloadSoundBanks | int | Unloads the sound bank with the given id from the sound system. | M |
| 328 | RemoveAnimObjSound | W, int object, int cue | When the world's sound slot is set, empties the first slot of the object with that cue (row 224). A voice already playing is not stopped. No level module imports this row. | H |
| 329 | MoveEffectEndSegmentToAnimatedLocator | int, int, int | Does nothing. The body is empty; the name suggests it would move an effect's end segment to an animated locator, but it does not. | H |
| 330 | MoveEffectEndSegmentToLocator | int, int | Does nothing. The body is empty; the name suggests it would move an effect's end segment to a locator, but it does not. | H |

Test vector: selector -1, cue 7, null VEC and do-not-restart set uses the active hero’s weapon slot and position (row 323); it starts nothing if a matching slot/cue/emitter voice exists. The same inputs through row 322 use the shield slot.

**Category 19: Entities, characters and movement**

| No. | Name | Arguments | Effect in plain words | Conf. |
| --- | --- | --- | --- | --- |
| 331 | MoveEntity | int entity, float x6, CTL | As row 135, and also sets the facing from the direction given by the last three floats (w = 0), by the same setter as row 112's face option. A notification is sent for the active hero when the debug flag is on. No module calls it. | H |
| 332 | MoveEntity | int entity, float x3, CTL | Same host record as row 135; the effect is stated there (move goal and position set to the point, no probe, facing kept; CTL 1 gives base state 1, any other value 33). | H |
| 333 | IsAI_Flying | int | Calls a per-class query and returns its raw value. The flying-state meaning is not confirmed (Open question 49). | L |
| 334 | MoveTargetToLocator | int, int | Moves the target marker with the given index to a locator. Does nothing when the index is out of range. | M |
| 335 | GetAI_MaxHitPoints | int | Returns the maximum hit points of an AI entity (looked up by id in the current world), or -1 when the entity is missing. | H |
| 336 | GetBreakableAlignment | int | Returns the alignment byte of the breakable object with the given index, or 0 when the index is out of range. | H |
| 337 | IsAI_Hit | int | Query: true when the AI entity (by id, current world) is in AI mode 19 (agro) and its agro outer sub-state word is 10, the marker that the hit-reaction entry sets (SPEC-0038). False when the entity is missing or in any other mode or sub-state. | H |
| 338 | SetTargetColor | int, float x4 | For a valid target index, copies the four float argument values into the first 16 bytes of that target's marker record. The copy preserves their bit patterns; their channel meanings and whether this record region is rendered as colour remain unknown. An invalid index does nothing. | M-H |
| 339 | CharacterPathAssign | int, int | Looks the entity up as the walk calls do (first int) and stores the second int in the +0xB8 word of its placement record, whatever its value (a negative value included). It starts no walk and sets up no route; a route already set up is not changed (SPEC-0056, Path walk). Does nothing when the entity is missing. No level module calls it. | H |
| 340 | CharacterOnScreen | int | Query: true when the entity is on screen (its visible flag is set). False when the entity is missing. | M |
| 341 | GetEntityPositionPointer | WORLD, int | Looks up the entity with the given id (the world argument is passed through) and returns a pointer to its position data, or 0 when the entity is missing. | M |
| 342 | EnableVillagerSocialize | int | Turns on a villager's social behaviour. The counterpart of row 118. | M |
| 343 | CharacterOnScreen | WORLD, int, int | Query: true when the current character is on screen. The body uses the current character and does not use its arguments. | L |
| 344 | GetAssetGroupItemRange | int, int outputs x2 | Writes the first and last item indices of an asset group into the two int outputs. Outputs only. | M |
| 345 | DFS_Act1_IsValidTrigger | int i | Returns the value of the first act-1 pair whose index is i (row 81), or 0 when no pair has index i or the table is empty. | H |

**Category 20: Hero, hit points and missile target**

| No. | Name | Arguments | Effect in plain words | Conf. |
| --- | --- | --- | --- | --- |
| 346 | KillBothHeroes | none | Puts both hero objects (the primary hero and the second hero slot) into their death state by the default death path. The death callback gate is bypassed, and health is left unchanged. See SPEC-0055 (hero death and respawn). | High |
| 347 | CloneHero | hero selector (not visible in the body), int | Calls the hero-copy routine with the int argument and a value taken from the current hero record. The hero enum is not visible in the body (Open question 59). | L |
| 348 | AddHitPoints | hero selector, int | The selector -1 means the selected hero; any other value is a hero index (0 Tor, 1 Roberta). Adds the amount to the hero's hit points, caps the result at the hero's maximum, then runs the heal notification (always, also when the amount is 0 or negative). Rules of the accessors: SPEC-0040, Behaviors 4 to 6. | H |
| 349 | DeductHitPoints | hero selector, int | Selector as in row 348. If the amount is below the current hit points, subtracts it. Otherwise sets the hit points to 0 and runs the death hook with both flags set (so the death gate and the default death run). The damage-applied notification then runs in both cases, also for an amount of 0. A hero already at 0 runs the death hook again. | H |
| 350 | SetHitPoints | hero selector, int | Selector as in row 348. The value rule of row 86: above the maximum gives the maximum and the heal notification; above the current value stores it and runs the heal notification; below stores it and runs the damage-applied notification (no death step, no lower clamp); equal stores it silently. | H |
| 351 | GetDispIDofLastItemPickedUp | int | Returns the display id of the last item the entity picked up. | H |
| 352 | IsHeroMovingBlock | none | Query: true when the selected hero's state is 18 or 19 (the block grab and push states, SPEC-0033). | H |
| 353 | UseItemFromPlayerInventory | int | Uses the inventory item with the given id on the hero. | M |
| 354 | GetHeroArmorAlignment | none | Returns the active hero's equipped armour Mote id from equipment word 0; returns -1 when that slot is empty (SPEC-0021, Equipment slots). | H |
| 355 | GetHeroHeightAboveGround | none | Returns the height-above-ground value stored in the active hero's placement record, or 0 when the record's floor flag is set. The floor flag is cleared at the start of each hero record tick and set to 1 when a contact response lands on a near-horizontal surface (contact normal with |y| above 0.9) or when the hero is being carried by a moving block, so the call reports 0 while the hero stands on such a contact and the stored ground gap otherwise (Medium). | Medium-High |
| 356 | GetHeroPositionPointer | none | Returns a pointer to the hero's position data. | M |
| 357 | GetMaxHitPoints | hero selector | Returns the maximum hit points of the given hero (selector -1 is the selected hero, otherwise the hero index). | H |
| 358 | GetHeroHeightPatchMaterial | none | Returns the material id of the ground patch under the hero, or -1 when the hero has no patch. | H |
| 359 | HasExternalMissileTarget | none | Query: true once if an external missile target was set. The query clears that flag, so a second call returns false. | H |
| 360 | GetPlayerID | player record pointer | Returns the player id stored on a player record, or -1 when the pointer is null. | H |
| 361 | GetDispIDofLastItemPickedUp | none | Returns the display id of the last item the hero picked up. | H |
| 362 | SetDefaultHeroIdleAnim | int | Sets the hero's default idle animation id. | H |
| 363 | GetExternalMissileTarget | none | Returns the id of the external missile target. | H |

**Category 21: Items, dialogue and counters**

| No. | Name | Arguments | Effect in plain words | Conf. |
| --- | --- | --- | --- | --- |
| 364 | AddItemToWorld | int locator, int item, bool snap, bool placed, float lifetime | Adds an item to the world at a locator. The first integer is the locator index and the second is the item id. The index must be zero or more and below the world's locator count (the type-11 locator count, Behavior section 3); an index out of range spawns nothing. The first bool is "snap to ground", the second is "placed" and the float is the item's lifetime in seconds (negative: the default lifetime); all three are passed on to the item spawn (SPEC-0021, spawning). The item spawns at full scale (start and target scale 1.0) with no delay, at the locator's position, with the locator's facing vector as its facing. Every call on the disc makes a placed item that never expires (see the note after this table). This is a host record of its own, distinct from row 201. | H |
| 365 | DisplayDialogText | dialogue record pointer, WAIT | Shows the dialogue text described by the dialogue record. When the wait mode is non-zero, the call yields each tick until the dialogue box has closed. | M |
| 366 | DisplayDialogText | char pointer | Shows the given text as a dialogue box with no speaker and no options. | M |
| 367 | SetItemBlockMode | bool | Sets the item system's block flag to the given value. | M |
| 368 | RegisterItemProximityCallback | WORLD, callback taking the item-proximity record | The body passes the callback and the world's item-proximity record to a shared-record lookup. When a matching record exists, that helper increments its +0x30 word; it does not write the callback pointer. The source of the matching record remains unknown (Open question 54). | M |
| 369 | ActivateCounter | int, float, RGBA pointer | Shows the on-screen counter. The body forwards its three arguments unchanged to the counter routine, which stores the id and the colour pointer (default when null), and stores the float when it is positive. | M |
| 370 | DeactivateCounter | none | Hides the on-screen counter. | M |
| 371 | GetCounter | none | Returns the current counter value (the callee's result, unchanged). | H |
| 372 | SetCounterColor | RGBA pointer | Sets the counter colour. The body forwards the colour pointer unchanged; the routine stores it (default when null). | M |
| 373 | MakeMovableItemUnMovable | int | Does nothing. The body is empty. | H |

**Note on ItemActivate and AddItemToWorld (rows 198, 201, 202 and 364; Confidence: Medium).** The placement rules named here (luck gate, vetoes, item roll, money roll, coins and placement ids) are specified in SPEC-0021 (Drop and money flow, and the placement sections). This note gives only how each host call uses its arguments.

- **ItemActivate, placement-id form (row 202).** The integer is a placement id. The id selects an entry of the world's placement-id table, which gives a slot and a placement ordinal. A slot code of 0xFF means the world's own item table (the current region); any other slot must be a created, loaded slot. The placement is then added by the SPEC-0021 rules. Both ItemActivate forms pass the default lifetime (-1) and treat their two bools as snap and placed (see below); the two vectors override the position and the facing when they are not 0.
- **ItemActivate, world form (row 198).** The first integer is a slot number of W (a SlotCreate result), and the second is a placement ordinal among the item placements of the region loaded in that slot. A number outside 0 to the created-slot count - 1, or a slot whose result word is 0, returns 0 and spawns nothing. Modules 1056 and 1096 pass a module data word that holds the result of an earlier SlotCreate (file value -1, 0 in play), so in play these calls use slot 0 (1056: the lesson slot, ordinal 0, snap false, placed true; 1096: the boss slot, the dispenser ordinals 0 to 12, snap and placed true) (High for the rule; Medium-High for the argument values). The items therefore come from the placements of the character region loaded into that slot. Module 1251 also passes a module data word with file value -1 that holds its SlotCreate result, slot 0 in play (SPEC-0037 module 1251; High).
- **AddItemToWorld, shared arguments (rows 201 and 364; High).** Both forms end in the item spawn of SPEC-0021 and pass it the same three values:
  - *snap* (first bool): when true, the spawn point's height is replaced by the ground height under it (the ground probe of SPEC-0018), so the item starts on the ground.
  - *placed* (second bool): when true, the item is a placed item and never expires; a non-charm placed item is also static (no spin, no bob, no flight) and never ages. A placed charm (item type 17) still hops on its spawn height, but never expires. When false, the item is an ordinary loose item with the hover, hop or coin flight of SPEC-0021.
  - *lifetime* (the float), in seconds. A value of 0 or more is the item's lifetime. A negative value means the default: no expiry when the item is placed; otherwise 15.2 s for a charm and 11.2 s for any other item. An item past its lifetime fades out and is removed (SPEC-0021).
- **AddItemToWorld, scripted-add form (row 201).** This form adds a placement by its ordinal in the current region, as a scripted add, and forwards snap, placed and the lifetime. The luck gate runs first. The vetoes of SPEC-0021 also run first, but only for keys of 0 or more. A negative key gets an item roll and no money (SPEC-0021, step 4). A key of 0 or more is spawned as the item itself. Two distinct host records carry the name: row 201 (int, bool, bool, float) and row 364 (int, int, bool, bool, float); no module imports row 201.
- **AddItemToWorld, five-argument form (row 364).** The first integer is the locator index and the second is the item id. The index must be zero or more and below the world's locator count (the type-11 locator count, Behavior section 3), and an index out of range spawns nothing. Snap, placed and the lifetime are passed on. The item spawns with start and target scale 1.0 and no delay, at the locator's position, with the locator's facing vector as its facing.
  - In one level module the calls pass first integers 5 to 10 and second integers 160 to 170. Another module makes 117 calls with first integers 24 to 71 and second integer 139, the 100-coin item of SPEC-0021, one coin per call. The locator count is on the disc, and every resolved locator argument of the 33 AddItemToWorld sites lies inside its pack's count.
  - All 118 AddItemToWorld call sites on the disc (the 117 coin calls and the one call site of the other module) pass snap true, placed true and lifetime -1 (High). Every scripted item therefore sits on the ground and never expires: the 100-coin items are static (they do not fly, bounce or spin), and the items 160 to 170 of the other module, which are charms, hop in place.
  - The host description gives a return of the new item handle, or 0 when the index is out of range. The returned value is the new item record's identity word, which later item lookups match by value (Low). No call site on the disc reads the returned value (all 34 live sites, 33 in one module and 1 in the other, discard it), so the handle has no effect on play (High). A rewrite may return the new item's identity for a spawned item and 0 for an out-of-range index.

**Category 22: Reanimation timers and timed waits**

| No. | Name | Arguments | Effect in plain words | Conf. |
| --- | --- | --- | --- | --- |
| 374 | GetReanimateTime | player record pointer | Returns a character's reanimate timer in seconds: the stored tick count (the dead-state counter, SPEC-0040) times the tick step (1/60 s, written at start-up; SPEC-0007, Rate selection). The record must be non-null, must not be one of the two hero records, must be in AI mode 16 (dead) and must have a counter above 0; otherwise the result is -1. The word "player" in the name does not mean a hero: a hero always gives -1. | H |
| 375 | SetReanimateTime | player record pointer, float | Stores the reanimate timer of a character: seconds times the tick rate (60, written at start-up; SPEC-0007, Rate selection), truncated to a whole tick count, into the dead-state counter. Does nothing unless the record passes the same test as row 374 (non-null, not a hero, AI mode 16, counter above 0 before the store). | H |
| 376 | WaitTillLeaveTriggerWithDelay | collision record pointer, int | Yields each tick until the trigger record reports the hero or entity outside (state 0 or 3) for the given number of consecutive ticks. Any other state resets the count. | H |
| 377 | SetReanimateTime | int, float | Looks up the entity with the given id in the current world and sets its reanimate timer, with the same conditions as row 375 (a missing entity does nothing). | H |
| 378 | GetReanimateTime | int | Looks up the entity with the given id in the current world and returns its reanimate timer, with the same conditions as row 374. Returns -1 when the entity is missing or does not pass them. | H |

**Category 23: World zones, loading and flag callbacks**

| No. | Name | Arguments | Effect in plain words | Conf. |
| --- | --- | --- | --- | --- |
| 379 | ActivateExteriorAssets | none | Despite its name this acts on the transition buffer T: when T holds a loaded world it makes T the current world (as in row 219) and then, when the selected hero has a movement controller, resets it. Otherwise does nothing. No module calls it. | H |
| 380 | ActivateInteriorAssets | none | Despite its name this acts on the exterior buffer E, in the same way as row 379 (make E current, then reset the selected hero's movement controller). No module calls it. | H |
| 381 | SetZoneVis | zone visibility mode | Turns zone visibility on when the mode value is 1, and off for any other value. | M |
| 382 | RegisterLevelFlagCallback | callback taking a raw pointer, int index | Stores the callback in record number index of the world script record's level-flag callback table (64 records of 24 bytes, one per level-state flag; row 152). An index outside 0 to 63 does nothing. The record also keeps the index in its level-flag field and -1 in its game-flag field. No module on disc calls it; delivery and re-arm are as row 383. | H (store, use); no module uses it |
| 383 | RegisterQuestFlagCallBack | callback taking a raw pointer, int index | As row 382 for the game-flag callback table (743 records of 24 bytes, one per game flag; rows 147 and 148); the index is stored in the game-flag field and -1 in the level-flag field. An index outside 0 to 742 does nothing. No module on disc calls it. The tables are read: each flag set or clear posts to the record, and the world's per-frame step frees the record once its callback thread has ended, so the record fires again on a later notification (SPEC-0022, Progress flags, item 4). The callback receives the pointer to its own record, which holds the new value at +8 and the old value at +12. | H (store, use); no module uses it |
| 384 | GetPriorExteriorFile | none | Returns the area-slot number last released from E, or -1 before one has been stored. | H |
| 385 | GetPriorInteriorFile | none | Returns the area-slot number last released from I, or -1 before one has been stored. | H |
| 386 | GetPriorTransitionFile | none | Returns the area-slot number last released from T, or -1 before one has been stored. | H |
| 387 | IsExteriorLoading | none | Returns the loading state of the exterior load slot, or 0 when no load is registered. | M |
| 388 | IsInteriorLoading | none | Returns the loading state of the interior load slot, or 0 when no load is registered. | M |

**Note on the hero class (rows 52 to 195).** The HERO class is part of the name encoding of rows 52, 53, 54, 171, 178, 182, 192, 193 and 195, but it is not a call argument. The only hero argument in these rows is hero selector, where 0 selects Tor, 1 selects Roberta and -1 selects the active hero (SPEC-0036; Medium for the -1 rule). Row 182 acts on both heroes. Rows 192, 193 and 195 act on the shared wallet, not on a hero. Rows 348, 349, 350, 357 and 361 are the other hero-class methods (section 2 of Behavior, Note below).

**Note on rows 295 to 388 (module use).** Of the 94 names in categories 16 to 23, only AddItemToWorld (row 364) is imported by a level module (2 modules). A scan of every import of the 163 distinct modules confirms that no module imports any of the other 93 names (High). Three rows have empty bodies (329, 330 and 373) and do nothing.

### 2.1 Generic switch setter (Confidence: High for the key arithmetic; Medium for the effect and the callers)

This is the setter that a switch id goes through. It has no catalogue row of its own. Its inputs are a switch id k (an integer) and a value (zero or nonzero).

1. **Key.** If k is below 201, the key is k + 200. Otherwise the key is k. Ids 0 to 200 therefore map to keys 200 to 400, and keys 201 to 247 are reached from two ids (k and k + 200 for k from 1 to 47). The key is looked up in the current world's switch list. If it is not found, the next world's list is searched.
2. **Slot.** The value is stored in a slot of the switch table. The slot is k when k is below 201, and k − 200 otherwise.
3. **Effect.** When the key is found, the entry's placed effects are switched. A nonzero value starts them (the effect's active flag is set). A zero value stops them (the flag is cleared and the effect's runtime slots are cleared). The placed effects are the placed effect records of the world (SPEC-0028, sections 4 and 9). Only the first effect index named by the entry is applied.

Rules that follow from this:
- Keys below 200 are never reached through this setter, and they need no other switch path. A type-40 entry's key is a row of the shared collected table (SPEC-0028, section 11; SPEC-0051): lock chests and element-sealed chests use rows 1 to 199 as their persistent "opened" mark, tested when the entry is set up and set to 1 when it opens (SPEC-0048, Lock chests). Dig sites use rows 201 to 247, which this setter reaches (one dig site has key 165). High.
- Key 0 is never sought by this setter.
- Calling the setter with an id whose key is not in either world's list changes the slot and starts or stops nothing. Calling it cannot reach pack 951's lock-chest key 43: the setter searches keys 200 to 400, and that entry has no placed-effect index.

A batch form takes a bit set and an id count (up to 255) and calls the setter once per id, with the value taken from the bit of that id. Row 89 (SetWhinnySwitches) does not use it: it is the Whinny pen-list setter and sets no switch (High).

Synthetic check: id 3 maps to key 203 and slot 3. Id 203 maps to key 203 and slot 3, so the two ids drive the same entry and slot. Id 201 maps to key 201 and slot 1.

### 2.2 Breakable objects (rows 46, 47, 48, 55 and 77) (Confidence: High for states and transitions; Medium for the break sub-steps)

Each breakable object has a state word. Its states:

| State | Name | Intact (row 48) | Invulnerable (row 77) | Drawn |
| --- | --- | --- | --- | --- |
| 0 | disabled | no | yes | no |
| 1 | intact, breakable | yes | no | yes |
| 2 | broken | no | yes | no |
| 3 | intact, not breakable | yes | yes | yes |

Transitions:
- SetBreakableActive (row 47): true moves the object to state 1. False moves it to state 0. From state 2, true returns the object to state 1, but the broken pieces are not restored (reproducible quirk, Edge cases).
- MakeBreakable (row 55): false moves state 1 to state 3. True moves state 3 to state 1. No other state changes.
- Break (row 46): acts only in state 1. In order: the state becomes 2; the object's drop, if it has one, is spawned at the break position, and a second drop follows if it has one; one effect object is spawned for each break piece, at the piece's transform, using the break source values; when the object's break value is above zero, its timed effect runs; its break sound plays. In states 0, 2 and 3 Break does nothing: no pieces, no drop and no state change.
- Hits and switch groups also run this break effect. A switch group breaks its first member in state 1 or 3, and only a member in state 1 is broken (the group and hit rules belong to the interactable and boss specs).

Reset: each world load rebuilds the breakable objects of the world from their start mode, and broken pieces are not stored, so a broken object is intact after a reload. Start mode 1 gives state 0. Start mode 2 gives state 1 with no drop. Any other start mode gives state 1. The start mode is applied only to objects whose sound lookup succeeds (Medium).

Break source values: before a break, three world words are copied to three globals that the effect spawns and the cue use as their source (Low for meaning; High for the copy).

### 2.3 Locked chests (row 203) (Confidence: Medium-High)

- The world holds a lock table of two entries, 12 bytes each, at world offset 0x5914. Each entry holds a key id (-1 when empty), a key item id and a message id.
- LockChest stores its three arguments (object id, key item id, message id) in the first empty entry. When both entries are full, the call stores nothing.
- A lookup by object id returns the key item for that id, or -1 when the id is not in the table, and gives the message id.
- A press-activated chest with a key item set: when the hero does not hold that item, the lock message is shown and nothing opens. When the hero holds it, all of that item is removed from the hero and the chest opens. Opening runs the object's open animation and effect. The routine that clears a lock entry has no caller in the boot program, so the entry stays after the chest opens (reproducible quirk, Edge cases).

### 3. Triggers, callbacks and proximity (Confidence: Medium, except where marked)

- **Registration.** A trigger is registered once, usually from Init, under an integer index. Registering writes the callback into the world's trigger table at that index. An index outside the table is ignored. A null world is refused and nothing is registered (High). Registering the same index twice keeps the later routine (pack 758, index 5; Medium-High).
- **Locator table (High).** Every locator argument of every host call (MoveHero, MoveEntity, WalkHero, WalkHeroAndOrient, RunHero, WalkEntity, WalkEntityAndOrient, SetHeroStartPos, the ReturnToNormalCamera preset, the LoopSound, KillSound and VisitLoopSound emitter, MoveEffect, AddItemToWorld, MoveMovableBlockToLocator, MoveObjToLocator, SetAnimObjAdditionRotLocatorForward and GetDistFromHeroToLocator) indexes one table: the 80-byte records of the world pack's type-11 chunk. The locator count is the u32 at payload +0x24, and the records start at the payload offset stored at +0x4C. A record holds its position at +0x30 and its facing (direction) at +0x40. Counts on disc run from 0 to 503 (pack 678: 50). Every resolved locator argument of every module lies inside its pack's count (for example MoveHero 272 of 272 sites, ReturnToNormalCamera 281 of 281 plus the -1 sites, WalkEntity 75 of 75, MoveEntity 125 of 125, AddItemToWorld 33 of 33). Added 2026-11-09 (High): a second lookup flavour exists for locators, by the 16-bit id at record +0x1E (the first record with that id); LaunchFireMote uses it, so its two ints are locator ids, not record indices. Its debug lines print only when debug mode is on and the debug level word (game-global +0x70) is at least 2.
- **Trigger capacity (High).** The world's trigger table has n entries, where n is the first u32 of the type-11 chunk payload; n is 0 when the pack has no type-11 chunk. Of the 192 packs, 189 carry a type-11 chunk; n runs from 0 to 228, and 69 packs have n = 0 (pack 678: 8). Indices 0 to n - 1 register; others are ignored. Every resolved RegisterScriptTrigger index on the disc is in range: 99 packs register triggers, with no index outside the capacity, and 26 of them register exactly index n - 1.
- **Other per-world table sizes (High).** Fixed when the world is set up: teleport callbacks, one per teleport plane; breakable triggers, one per breakable object; proximity callbacks, one per character descriptor of the world (indexed by descriptor id); item ask-about callbacks, 10; game-flag callbacks, 743; level-flag callbacks, 64; control threads, the n given to ControlThreadsAllocate.
- **The world's object pools are these callback tables (High, raised 2026-11-11 after the manager's construction was checked: the counts are A = the first u32 of the type-11 payload, B = the third word of the placed-objects chunk, C = the teleport plane count, D = the entity proximity count; the loader also registers script group type 3 itself when a type-40 chunk exists, see call 274).** The seven pools A to G and the scene list that SPEC-0028 section 15 lists are the script manager's callback tables: A the script triggers (count = the first u32 of the type-11 payload, the trigger capacity above); B the breakable triggers; C the teleport callbacks (one per teleport plane); D the entity proximity callbacks (one per tag-0x12 entity entry); E the 743 game-flag callbacks; F the 64 level-flag callbacks; G the 10 item ask-about callbacks. The scene list holds the per-placed-object instance triggers, one sub-list per placed object, sized by the first word of that object's inline type-11 header. Resolved 2026-11-09 (High): the table of 50 entries with 0x400-byte stacks built beside them is the script manager's own thread pool (50 threads, 1 KB stacks), set up in the same constructor from the same memory arena. Pool order in the manager: triggers, breakables, teleports, proximity entries, game-flag callbacks, level-flag callbacks, ask-about records. All record blocks start idle (zeros, with -1 in index fields): trigger and breakable records 0x48 bytes, teleport records 0x44, proximity and callback records 0x18. A release rule frees finished callback threads.
- **Enter test (High).** The trigger pass (SPEC-0028, Behavior, Trigger pass) tests each volume once per tick for each actor that runs it, with no enter or leave edge. A hit runs the trigger's fire step only when the volume's flags pass the fire gate of its kind and path (see Fire paths below), and the fire step writes the callback state word under the update rule (Layout, section 3): 0 idle, 1 first fire, 2 repeated fire, 3 decaying toward 0. For the selected hero's body contact, state 1 means the hero has just entered, 2 that it stays inside, and 3 then 0 that it has left (3 two frames after the last contact frame, 0 one frame later). Order in a tick: world step (movable blocks, region characters), selected hero (shots, then body contact), petpet, then the script step (threads resumed, then decay).
- **Per-tick firing (example, Low).** A script-group trigger of the simple kind fires only when all of these hold: the group has at least one member; the member's animation has finished; the hero is on the ground (height above the ground below 0.1 units); and the hero is not occupied by another action. When it fires it plays the group's animation and camera and may activate an object.
- **Proximity callbacks.** Entity proximity callbacks receive the proximity record for an entity id. The bool of RegisterEntityProximityCallback is the range flag of the talk entry (row 251; set in every call on the disc; High). The on-enter registration takes no bool; its callback receives the record (v, 0, 0) holding the int given at registration (row 255; High). The talk frame that every entity proximity callback follows (at most one line, bracketed by SetConversationTarget and EndConversation, nothing at all when no line is chosen) and the line rotation kept in module data words are specified in SPEC-0022 (Proximity talk frame and line rotation) (High).
- **Teleport callbacks** are registered with a trigger index and a prompt. They run when the hero steps on the trigger and the prompt is shown while the hero is inside.
- **World callback.** Each time the world becomes current, its activate-world callback is started as a thread with the world's slot number (unless it is still running), and then its on-enter callback (rows 252 and 255). Both start after the arrival placement of the load (section 6, Init order; High).
- **Hero event callbacks** (death, use item, pick up item, buy item; rows 264, 269, 265 and 260). For buy, pick up and use, a registered callback holds the event back: the event's own effect does not happen at once, the callback starts as a thread with a record holding the item id and the registered flag word, and the event's own effect runs after the callback returns, only if the flag word is still 0. If the previous callback thread is still running, no new thread starts and the event does nothing at all (it is not queued; Medium-High). With no callback registered the event runs as usual. The death callback has its own rule (row 264).
- **Item ask-about (High).** Each world has a table of 10 ask-about records (callback, item id, prompt flag, asked flag). A free record has id -1.
  - Registration (row 257) reuses the record whose id equals the item, or else takes the first free one. The reuse search accepts only ids below 10, the table size.
  - The tick test: each tick, when the selected hero's nearest ready world item is within 8.0 units, the hook searches for the record by that item's id. The same below-10 rule applies here.
  - A found record that is not yet asked, with its prompt flag set: the Square caption label 9 (design table 13) shows, and the item is not collected that tick. When Square is pressed that tick, the record is marked asked and its callback starts as a thread with (item id, distance). It therefore runs at most once per world load. The test runs after the world interaction chooser, so such a record could act on the same press as a door confirm.
  - Otherwise the world's item-proximity record (row 368) is used instead. Its callback, if one is set, starts each tick its thread is not running.
  - On disc all 27 registrations name ids 57 to 174, and no module registers an item-proximity callback. So no ask-about callback, caption or collect block ever happens.
  - Clean build: reproduce this (nothing fires), or record the choice to make it work (Edge cases).
- **Proximity test (Medium).** A trigger or proximity record takes part in the proximity test only when the pointer at its offset 0x74 is not null. The test uses the squared distance between the record's position (the position vector at offset 0x40) and a reference point that the caller passes in. When that squared distance is below 160000 (400 units squared), the record's proximity action runs with the scale 1.0, the data block at offset 0x10 and the value at offset 0x64. Otherwise, or when the pointer is null, the test returns 0 and nothing runs. The 400-unit radius is a code constant. Other count scans in the effect code use the same test (not described here). The proximity action itself is not described here.
- **Movable block pass (High for the dispatch and timing; Medium for the tests).** A movable block walks the volume records of the selected hero's cell list A (not the block's own cell) only while it is moving and on the tick its moving flag clears; an idle block does not walk them but fires the volume it last hit again on every tick (SPEC-0048, Trigger volumes). The block's shape is a vertical cylinder of radius S and height H at its position. For each non-null record, the kind byte at offset 0x10 selects the handler:
  - kind 1: needs flag bit 3, then a volume test that uses the block's position, the radii in its block at 0x68 and the record's centre at 0x30. On a hit the fire step runs.
  - kind 2: no action.
  - kind 3: needs flag bit 3, then a test that uses the record's axis at 0x40 and its length at 0x24 with the block's radius. On a hit the fire step runs.
  - kind 4: when the float at record offset 0x2C is 0 or less, needs flag bit 8 and a volume test on the record. When it is above 0, needs flag bit 5 and a test of five faces of the plate box (all but the top) with a test point of radius 0.6 S (S the block's size; fixed constant 0.6; SPEC-0048, The block's moving-rectangle test). On a hit the fire step runs.
  - other kinds, kind 7 included: no action.
  The fire step runs only when the block's manager pointer at offset 0x70 is not null. It fires with no actor, object 0 and the block's number (offset 0x98) as the last argument (+0x28). The block's word at offset 0x60 is then set to the record. The numbers 3, 5 and 8 that the tests name are flag bit numbers (0x0008, 0x0020 and 0x0100), not table numbers.
- **Fire paths, by volume kind and flag bit (High; SPEC-0028, section 13 has the full table).** The fire step takes the trigger index from the volume's u16 at +0x1C and the actor, object and block words from the path that calls it.
  - Body contact (each actor's trigger step: cell list A, cell list B and the inline volumes of animated placed objects): kinds 1 and 2 fire with flag bit 3 (0x0008) and bit 0 clear; kind 3 with bit 3 and bits 1 and 2 clear; a static rectangle (kind 4 or 7, switch zero or below) with bit 8 (0x0100) and bits 0, 1, 4 and 6 clear; a moving rectangle (switch above zero) with bit 5 (0x0020) and bits 1, 7 and 8 clear; kind 8 never fires. A hit goes to the owner's trigger-hit method: the selected hero's fires with object 1, every other character's with object 0, and the base method does nothing. A petpet's fire is refused by the fire step. NPCs and enemies test only cell list B (trigger objects) and the placed-object inline volumes; the host CollideEntityWithTriggers (row 276) runs a cell list A walk for an entity, with the fire bit required. The plain type 11 volumes (spheres, cylinders, rectangles) sit in cell list A, so a script trigger whose volume is a plain type 11 record can be fired by a townsperson or enemy only through that host call (R-0282, High). Example: pack 1335 triggers 5, 8 and 9 are plain type 11 volumes and the module never calls CollideEntityWithTriggers, so the seated Seradar at the centre of trigger 8 and the palace minions fire nothing; only Roberta's body and her shots do.
  - The hero shot pass: each live shot of the selected hero, each tick: kinds 1, 2 and 3 with bit 3, a static rectangle with bit 8, a moving rectangle with bit 5, no bits-clear conditions and no gate mask; fires with the hero as actor, object 0 and shot word +0x20 = 1.
  - The movable block pass (above): fires with no actor, object 0 and +0x28 = the block number.
  - The world projectile step: the same bits as the shot pass (a moving rectangle needs bits 5 and 8), and only for indices that the act-1 trigger map holds (pack 1174); no actor, object 0.
  - A volume with flags 0x0001 (bit 0 with bits 3, 6 and 8 clear) never fires a script trigger. It gives the contact push and stops a shot only.
  - Example: in the final-fight world (pack 1432) the 52 cylinders with flags 0x0001 do not fire; trigger index 0 is one static rectangle (flags 0x0100) and index 1 is cylinder 8 (flags 0x0008). The selected hero's body fires either with object 1; a hero shot fires either with object 0 and +0x20 = 1 (SPEC-0054).
- **Group types.** A script group has a type from 1 to 7. Group type 7 registers all of its members. The group's behavior depends on its type (Medium).
- **Pending book word (Medium-High).** No host call writes the pending book word. Only kind-3 book-station volumes set it (SPEC-0028).

### 4. Threads and waits (Confidence: High for Sleep, Wait, control threads and script control, Medium otherwise)

- **Script threads** are cooperative. Each is a routine that runs until it yields. A yield returns control to the frame scheduler, and the thread resumes on a later tick. One yield equals one game tick (1/60 s, SPEC-0007). The order in which threads are stepped each tick is in section 1 (Per-tick script step).
- **Thread limit (High).** At most 50 script threads can be alive in one world at once, each with a 1 KB stack. Every callback run, control thread, Update run and sequence is one thread. The original has no safe failure when a 51st is needed (Edge cases). Confirmed 2026-11-09 (High): the pool is built for 50 threads of 0x400 bytes, and the allocator, when the in-use count is not below the maximum, prints a debug message that it ran out of threads and returns null; a thread header of 0x24 bytes sits before each stack.
- **No automatic stepping (High).** Threads started by script calls are stepped only by the per-tick script step (section 1). They never auto-step: the thread-pool sweeps of the main loop step only threads created with the auto-step word, and none of the script's threads has it.
- **Control threads (High).**
  - ControlThreadsAllocate(W, n) gives W a table of n entries. Each entry holds a callback, its argument and a thread handle, and starts free.
  - ControlThreadStart(W, f, p) takes the first free entry and creates a thread that runs f(p) and then ends. It returns nothing to the script (the host routine returns the entry index, or -1 when the table is missing, has a count below 1 or is full; the script call discards it; confirmed 2026-11-09). With no free entry it does nothing.
  - Step 4 of the per-tick script step steps each entry's thread in table order. An entry whose thread has ended is freed in that same pass, so it can be reused from that point on, including by a later step-4 thread in the same tick.
  - A control thread counts toward the 50-thread limit.
- **Sleep(n)** yields n ticks. n of zero or less does nothing.
- **Scheduling rule of script threads (High).** Script threads are cooperative and have no counter, flag or ready list of their own. Each tick the per-tick script step (above) runs every live thread once, in the fixed order of its five steps (special threads, trigger callbacks, other callbacks, control threads in slot order, Update creation), and a thread runs until it yields or ends. A thread yields only at Sleep, Wait or a wait loop of a waiting wrapper; it resumes in the next tick at the same place in that order. A thread that never yields would stall the whole tick (no rule frees it); a wait loop that never ends stalls only its own thread and keeps its slot (and its place in the 50-thread limit). No thread can keep another from finishing.
- **Caught-exception record (High).** A 0x30-byte record with a callback, a flag, a link and a 64-bit counter exists in the boot program, but it is not a script thread: it is the compiler runtime's record of a thrown C++ exception. Its two 16-bit fields at offsets 0x04 and 0x06 hold 4 and 1 (the language tag and version), the word at 0x08 is the thrown object, 0x0C its type, 0x10 the cleanup routine, 0x14 the caught flag (zero while the exception is still being thrown, set when a handler takes it), 0x18 the link to the next older record (the newest record is the head of the chain), 0x20 the handler count and 0x28 the original object. The end-of-handler step subtracts one from the count; a nonzero count ends the step. At zero, a record that is the newest in the chain and has the caught flag clear is kept (the exception is still in flight; its count is then -1 and later subtractions never bring it back to zero, so it is never finished). Confirmed 2026-11-09 (High): the creation routine builds this record and makes it the new head of the chain, and the end-of-handler routine behaves as described here; the endless routine that creates one such record and yields in a loop is the runtime's own throw loop and never returns. Any other record is unlinked, its cleanup routine is called with the thrown object and the value 2 (destroy the thrown object), and its storage is released. Nothing in the game proper throws: the throw and record-creation routines are called only from the runtime's own unwinder code, and none of the 276 distinct import names of the 173 level modules is a throw, catch or exception name. So no such record is ever created in play and a rewrite implements nothing for it.
- **Wait(f)** yields the whole part of f x 60 ticks, by the seconds-to-ticks rule below; a count of 0 or less returns at once without yielding. The multiplier is the game tick rate, 60, written once at start-up from the boot rate table (SPEC-0007, Rate selection); it is not in the file.
- **Waiting wrappers.** When a call's wait mode is nonzero (exactly 1 for PlayEntityAnim; Layout section 2.1), the caller yields each tick until the operation finishes. Examples: PlayAnim waits while the object is in the play-once state (state code 2); PlayHeroAnim always yields one tick and then, with the wait set, waits while the hero stays in the one-shot state; WalkHero waits while the hero is walking. There is no timeout in these loops (Low; a wait that never ends stalls its thread).
- **Ticks for seconds.** Animation time is in seconds. Sampling uses frame = floor(time x 15) and fraction = time x 15 - frame (Medium; 15 frames per second is a code constant).
- **Seconds to ticks (High).** A script time of s seconds becomes trunc(s x 60) ticks: one single-precision multiply by the tick rate (60, written at start-up; SPEC-0007, Rate selection), truncated toward zero, with no double-precision or rounding step (SPEC-0008). Wait, LoopEntityAnim, LoopHeroAnim, the fade calls and the letterbox calls all use it. Fades and letterbox bars use at least 1 frame. A decimal time whose single-precision value lies just below the decimal loses one tick: 0.7 s gives 41, 1.3 s gives 77 and 1.8 s gives 107, while 0.5 s gives 30 and 0.6 s gives 36. LoopEntityAnim stores a count of 0 as -1 (forever), so any time below 1/60 s loops forever (row 104); LoopHeroAnim has no such rule (row 168).
- **Wait-loop request codes (Medium).** The hero waits poll four request codes. 0x32 (50, walking): WalkHero yields while the hero holds it. 0x34 (52, walking and orienting): WalkHeroAndOrient yields while the hero holds it. 0x33 (51, one-shot): PlayHeroAnim sets it, always yields one tick, and then, with WAIT set, yields while the hero holds it. 0x37 (55, loop): LoopHeroAnim sets it and does not wait. Each wait yields once per tick and has no timeout.
- **Script control and the entity's AI (High).** Every character has one base state word (SPEC-0040, Runtime fields); the state setter keeps the previous value when the state changes. State 1 is free and state 33 (0x21) is script hold. In base states 16 (dead), 25 (conversation), 33 (script hold), 36, 42, 53 and 55 (loop) the NPC update does not run the state-request apply step (SPEC-0040, 15A); it runs in every other state. So while an NPC is in one of these states, the requests of its AI (mode machine, behaviour classes, walks, greeting) are stored but not applied, and they take effect again once the state returns to a default one. Each tick in state 33, when the current clip's progress is at least 0.99, the NPC requests its default idle animation (row 120). A character in base state 16, 33, 36, 53 or 54 is never a hit candidate; together with the invulnerable flag (row 132) this is the base rule of the hittable test. GrabEntityControl sets state 33; ReleaseEntityControl and ReturnEntityAnimToIdle set state 1; the walk calls also set state 1. A script walk (priority 2) parks the AI coroutine as described in row 106. A walk class that was already applied before the hold follows the next bullet.
- **Script hold during a walk (High; Medium-High for the drift).** An entity whose walk class (23 to 26 or 35) is applied, and that is then put in base state 33, works like this:
  - The walk freezes. The walk stays its applied class, but its walk routine is not stepped, so there is no steering, no arrival test and no hold test.
  - The current clip keeps moving it. The clip already playing keeps playing at its start rate, and its root motion keeps moving the entity along its facing. The facing is not updated in state 33.
  - It stops at clip end. On the first tick the clip's progress is at least 0.99, the entity requests its default idle animation (row 120). The idle has no root travel, so the entity stands. With a looping walk clip the drift lasts at most one clip cycle.
  - The walk resumes on release. When the state returns to 1, the walk routine resumes where it yielded, requests the movement clip again and goes on to the same target.
  - Waits and queries carry on. IsEntityWalkToPointActive stays true throughout, and a WAIT loop of the walk call keeps waiting.
  - The drift follows from the motion tick and the animation pass, which state 33 does not gate; it is not confirmed on the running game (Open question 87).
- **Hero script hold (High).** When the hero enters state 33 from another state, its motion-record velocity and the velocity of its locomotion sub-object are set to zero. In state 33 the hero only requests the animation code in its input word (SPEC-0040 15.3) (rows 164 and 166).
- **Save prompt while a sequence runs (High).** While the enter-interior or leave-interior sequence thread of the current or the second world is alive, the game-state code is 6 (SPEC-0026). The save-point step shows the "Save Game" prompt and accepts the Square press only when that code is 0 (SPEC-0025, Save points), so a running interior sequence hides the save prompt and blocks opening the save screen. Other codes above 0 do the same (for example code 4 while a dialogue box is up, code 5 during a hero swap). This is the only way a level affects saving; no host call does it directly.
- **Switch-group prompt thread (High; SPEC-0048, Prompt thread).** This routine runs as the trigger-0 callback for a kind-4 switch group after a repeated trigger fire, while the main hero remains in the volume and breakable 0 is intact. It checks whether the target element is nonzero and differs from the active Mote element. While mismatched, it shows Square context prompt 29 until Square has been held once; otherwise it shows prompt 30 until Square has been held once. Both are context prompts, not host events or sound cues. Prompt 29 can show again after the element matches.

### 5. Slot loading and characters (Confidence: High for region numbers, slot numbering and the keep flag; Medium for the other slot flags)

- A **slot** holds one set of character data for a world. SlotAllocate(W, n) creates n empty slots and sets the created count to 0. The slot record and its fields are in Layout, section 4.
- **Slot numbers.** A slot number runs from 0 to n - 1, where n is the count SlotAllocate gave the world. Every slot call passes its number through unchanged. A number outside that range, including -1, is ignored: SlotLoad returns 0 after one yield step, SlotReset does nothing and SlotCharacterCount returns 0. No call maps -1 to another slot (High). A level module reads the number from a data word of its own. A load reads that module data word. A word whose file value is -1 and that is never stored in reached code makes the load do nothing. The SPEC-0032 modules that create a slot store the SlotCreate result into their word during init, so their loads go to slot 0 (High for the code; Medium for the other modules).
- **Region number.** A character region is the archive resource numbered pack + 1 + j, where pack is the world's pack number and j is the index the level passes in. Example: for pack 678, SlotLoad with j = 2 loads resource 681. When the pack number is negative the region is j + 1.
- **SlotCreate(W, n, js).** Loads nothing. Each call reserves a buffer for the next free slot number, in creation order, sized for the regions named by the array (row 207 gives the size), and returns that slot number, or -1 when all slots given by SlotAllocate already have buffers. A call with an empty array still uses a slot number. The array's region numbers are not stored, and nothing loads them later (High).
- **Which characters an area shows (High).** A character region appears in play only when a SlotLoad that runs names it, through a slot word that holds a created slot number. Being listed in a SlotCreate array is not enough. A SlotLoad call can be reached from several branches that each set a different j; every branch counts. Examples: regions 848, 915, 929, 949, 952, 954, 1048, 1060, 1109, 1113, 1116, 1117, 1119, 1310 and 1383 are listed in a SlotCreate array and named by no SlotLoad, so they never appear; 1304 and 1309 are listed in a reached level's array but loaded only by a level that is not reached.
- **SlotLoad(W, slot, j, keep, activate).** Row 205 applies. Steps in order: range check (a slot outside 0 to n - 1 returns 0 after one yield step and does nothing else); release of the slot's previous content (pending read cancelled, result 0, region lists emptied, characters released, counts cleared, cap 6, keep flag 1), so a load replaces the set; keep applied (keep false clears the keep flag and sets the character cap to 15; keep true sets the keep flag and leaves the cap of 6); the region number is recorded, the load starts and the slot becomes busy; the call yields while the slot is busy, with no timeout, then yields once more; the region's characters are set up when the read completes, whatever activate is; the call returns the slot's result (1 for a completed read whose data starts with the IDM signature, else 0); and when the result is nonzero, activate is nonzero and the slot number is below the created count, the region's item placements are spawned by the region-load route of SPEC-0021. SlotLoad is the only reader of a character region; it reads into the slot's SlotCreate buffer from the buffer's position rounded up to 128 bytes. A slot number below the capacity that has no SlotCreate buffer has an all-zero handle, so its read has no target (Open question 28).
- **Activation limit.** The activate step spawns the region's item placements (kind-8 records), not its characters, and runs only for slot numbers below the created count (High).
- **Keep and the per-tick pass.** The fourth argument of SlotLoad is a keep flag, passed to the slot code unchanged. With keep set (1), the per-tick update of the slot runs the full streaming pass of SPEC-0006: it releases characters that are dead with their death countdown ended or farther than 280 units from the active hero, and binds the nearest eligible records within 280 units up to the cap. With keep clear (0), the per-tick update only releases dead characters whose death countdown ended; it binds nothing new, so the slot shows the characters set up when its read completed (up to 15) and loses only its dead. Keep is not a reset: the slot's previous content is released on every load (High).
- **Embedded region cap.** The pack's own embedded character region streams like a keep-set slot, with a cap of 6 by default. SetMaxActiveAI (row 62) sets that cap, clamped to 15; packs 1176 and 1246 set 12 (High).
- **Permanent kills and forced binds.** DeactivateEntity (row 114) kills a character permanently by setting its record's kill bit; ActivateEntity (row 116) binds a record at once and clears its kill bit (SPEC-0006 kill memory; High).
- **Activation filter.** RegisterAIPlayerActivateCallback (row 259) sets the filter that decides, per record ordinal, whether streaming may bind a record. It reaches the embedded block and the slots created before the call, not later ones (High).
- **Waves.** The engine has no wave timer. Waves are driven by the level script: it tests the previous set with IsAI_Dead (53 call sites in 21 modules), and then loads another region into a slot with SlotLoad, which releases the previous set first. Example: the Coliseum (pack 1371) loads regions 1378, 1379 and 1380 by story stage (High for the absence of an engine timer; Medium for the example's stage order).
- **World reset.** A world-level reset (not named here) empties every created slot and sets the created count to 0, so SlotCreate numbering restarts at 0 (Medium). There is no separate free call (Open question 29).
- **Characters.** A slot holds up to 15 characters (indices 0 to 14). CharacterActivate(W, slot, n) activates character n. SlotCharacterCount returns how many are active. The character records are the region's own chunk data (SPEC-0004, SPEC-0005).
- **Asset groups.** GetAssetGroupCharacterRange maps an asset group, which is a slot key of the world's entity-id table, to the first and last index of that key's run of entries (row 215). Which group a given key names in a module is level data (Open question 9).

### 6. Transitions and area loads (Confidence: Medium; High for the placement after a load, the landing rules and the Init order)

Sequence when a world is entered:
1. The world is chosen. Its pack number (the key for region numbers in section 5) and its world identifier are set.
2. The pack's module is attached to the world and its import names are bound to host functions.
3. The family is chosen by the buffer (section 1.1): E Exterior, I Interior, T Transition.
4. The family's Init is called with the world, once, as a plain call inside the load-completion step (Init order, below). Init registers triggers and callbacks, allocates slots and threads, and starts the first threads.
5. Each tick while the world is current, the world's script step runs (section 1, Per-tick script step); it keeps one Update thread running where the module has an Update entry.
6. When the world's script manager is torn down, Exit is called (where present), just before the module is unloaded.

**Init order (High).** A world's pack is read by its buffer's loader. When the read completes, the loader's completion step runs in this order: clear the buffer's busy word; check the IDM header; walk the chunks (SPEC-0006), which also binds the type-37 module; resolve the family's Init export and call Init(world). All of this happens before the arrival placement of that load and before the world is made current.
- For a request-driven load (exterior replacement, return after a movie, pending slot), the driver pumps the loader until no load is active, then places the hero (placement after an exterior load, below), then makes the world current when the request's make-current word is set.
- For the enter-interior job, the job waits for the interior buffer, makes the interior current, then lands the hero.
- During Init, the current-world pointer is the world that was current before the load; nothing in the load makes the loaded world current before Init has run. It equals the loaded exterior only when E is released and read again in place while it is current: a script exterior replacement load, a save load or Continue made while E is current, a movie hand-off with a return world while E is current. It is T when an exterior is streamed in from a transition (the load starts only while T is current, and the swap needs E's ground, which exists only after the load) and when a game starts or a save is loaded from the title (T is read and made current, then E is read). It is T or I when the movie hand-off reloads E with T or I current. It is E while an interior or a transition loads (enter interior, leave interior, streaming of a transition). An interior visit does not reload E, so E's Init does not run on return (SPEC-0028, Run-time object changes). Host calls in Init that act on the current world (MoveHero, SetHeroStartPos, ActivateObj, entity lookups) act on that world; the calls that take W act on the loaded world.
- A start pose stored by Init (module 777's SetHeroStartPos(8)) is overwritten by the placement that follows. Arrival logic belongs in the activate-world and on-enter callbacks (rows 252 and 255), which start when the world becomes current, so after the placement.
- Because Init is a plain call inside the loader's completion step, a yield inside Init would yield the loader (Open question 80).

Transition requests:
- StartEnterInterior, StartLeaveInterior, StartExteriorReplacementLoad (with or without fade), and StartTransitionScene each record a world and the fade colour, and start a load. Their parameters are described in the catalogue (rows 277 to 284). StartEnterInterior and both exterior-replacement loads work only while the exterior world is current, and StartLeaveInterior only while the interior is current; otherwise each shows an error message and does nothing. StartLeaveInterior's world names the transition to reload, not the destination. StartTransitionScene has no live caller. RegisterTransitionMap (row 263) sets a map-display id and is not a load (High).
- **World-taking call sites (High).** Over every host-call import site in all modules, 194 call sites in 66 modules take a world (slot) argument. Every slot value is a fixed value in the calling code (several branches can share one call), except one helper in pack 1427, whose eight trigger callbacks pass slots 187 to 191 through it to SetWorldLoadOverRide; that helper sets the pending slot at most once per tick. Live calls by kind: enter interior 44, leave interior 21, exterior replacement 98, return world after a movie 12, return transition after a movie 1, pending setter 6 (the three 1184 sites, 1391, 822 and the 1427 helper); StartTransitionScene none. Dead sites: the two StartTransitionScene calls of pack 758, pack 1098's entry to 1170, the self-reload helpers of 1176 and 1371, one site in 1184 and a second entry to 1409 in 1350.
- Placement after an exterior load (High). Each world slot's load request carries a plane index p (reset value -1; scripts pass it unchanged). When the load completes, the request is cleared and then: if p >= 0, the active hero teleport-lands on teleport plane p of the loaded world (SPEC-0040, Behavior 1C), and placement ends if that succeeds. Otherwise (p = -1, or a refused plane index) the start pose becomes entry record 0 of the loaded world; if p <= -2 it becomes the hero's current position and forward instead. The hero's current-exit link is cleared, the start pose is stored and the hero respawns at it without the heal (SPEC-0040, Behavior 1C). A loaded world with no entries gives the zero vector for both position and facing (edge case). Then, when the request's seventh word (make current after placement) is set, the loaded world becomes current. Script loads set that word to 1; the movie hand-off sets it only when it uses a return world or a copied return transition (rows 141 and 146; SPEC-0026), so otherwise the buffer it reloads is not made current by the hand-off (High).
- Interior entry and exit (rows 278 and 279) run as jobs with their own landing rules (rows 278 and 279). Every enter call passes plane -1; the enter and leave calls whose entry argument is resolved (28 enter, 19 leave) pass entry -1, so they land on entry record 0 of the destination.
- While a load is pending, the transition driver runs each tick. It loads the files for the current and the next world, and then retires the previous transition thread (Medium).
- Query functions give prior buffer slots: GetPriorExteriorFile, GetPriorInteriorFile and GetPriorTransitionFile return the most recently released E/I/T area slot, and GetLastActiveWorld returns the previous active area slot (SPEC-0006, Prior and active world slots; High).
- IsTransitionLoading and IsWorldOrCharacterLoadActive report whether a load is still running.
- For the Transition family, the host registers a transition map with id -1 when the world is set up (Medium).

### 7. How level behavior will be specified (Confidence: High for the rule; the schema is a plan)

The clean side will not execute the original machine code of any level module. There is no emulation, no MIPS code loading and no code translation. Each level's behavior will be written in a per-level spec as data plus rules, using the names in the catalogue (section 2) as its vocabulary.

A per-level spec lists, in order:
- **init steps**: slot allocation and slot tables (SlotAllocate, SlotCreate), thread allocation and starts (ControlThreadsAllocate, ControlThreadStart), object setup (InitObjOn, ActivateObj), and registrations (RegisterScriptTrigger, RegisterEntityProximityCallback, RegisterTeleportCallback, RegisterActivateWorldCallback, RegisterHeroDeathCallback, and so on);
- **triggers and proximity**: for each index or entity id, which handler runs and when;
- **handlers**: each handler is a list of actions, possibly under conditions on flags (QueryGameState), or a line cycler (a stored dialogue set and a cursor of two or three steps, played with LevelMsg), or a small state machine with named variables and transitions, or a poll (wait until a predicate);
- **thread bodies**: the steps a control thread runs, with Sleep, Wait and the waiting wrappers;
- **update and exit**: the per-tick routine, if any.

The action vocabulary is the 23 catalogue categories (section 2): animated objects, locators and effects; cameras; combat and boss hooks; dialogue and messages; entities and NPC AI; fades, letterbox and movies; game-state and blackboard flags; hero control and state; items, money and inventory; slots and asset groups; sound and music; timers, threads, time and random numbers; triggers and callbacks; and world and area transitions. Categories 16 to 23 add animated-object pose and playback; camera control; effects, sound and vibration; entities, characters and movement; hero hit points and missile target; items, dialogue and counters; reanimation timers and timed waits; and world zones, loading and flag callbacks.

Numeric and identifier arguments (flag ids, object ids, locator ids, entity ids, line ids, sound ids, item ids) are given as data in the per-level spec, because most of them are read from the level's own data and not written as fixed values in the code. Until they are decoded, the per-level spec must mark each one as "from level data, id not yet mapped".

**Module data words are live state (rule for the rewrite; High).** A level module's data words are read-write state that lives while the module is loaded. Scripts keep counters, phase numbers and "done" or "door open" words there, and branch on them in triggers, threads and the update routine. The rewrite must treat every module data word that the code writes as a live variable whose initial value is its value in the file, and must keep every branch that depends on it: a per-level spec names such a word as a state variable, and any tool that derives level behavior from a module must evaluate the module code against live module memory. Treating a module data word as a constant equal to its file value gives wrong results: it hides every call that runs only after the word changes (most puzzle completions and several story flags). When a module is loaded, its words hold their file values, and then its load-time constructors run (Which level routines run in play). Init therefore sees the file values, apart from the words a constructor writes, and every later routine (triggers, threads, callbacks, Update) sees the current values written since the load. The words last until the buffer that holds the pack is next released (section 1.1): an exterior keeps them across interior visits and swaps with its transition. Some scripts dispatch through jump tables (a table of code addresses in module data, indexed by a computed value, for example a trigger record's index field, or an entry-message value minus 10, divided by 10). Every table entry is an ordinary branch target of the same routine and must be followed. Reachability of a host call is therefore a run-time property: of the 720 SetGameState call sites in the 173 modules, 136 lie in code that a static walk with constant module data does not reach although the code is called from Init, Update or a registered callback.

**Which level routines run in play (reachability rule; High).** A per-level spec lists a routine as a game effect only when it passes this rule:
- **Roots.** Init, Update, Exit and the module's load and unload hooks are the roots.
- **Load-time constructors (High).** The load hook runs the module's constructor list: a list in module data of a count followed by routine addresses and a zero word, run from the last entry to the first, once per load, after linking and before Init. The unload hook runs a destructor list (a pointer to a zero-terminated list of routine addresses, run first to last); it holds one entry in packs 1056, 1163 and 1382 and none elsewhere, and no destructor makes a host call. Each constructor entry calls a routine that receives (1, 65535) and then sets module data to its start values. 36 of the 163 distinct modules have a constructor list: in 26 of them it resets the shared entity-control table (SPEC-0043) and in the others it sets other module records and words. Only pack 1324's constructor makes host calls: QueryGameState(239), stored in a module word, and GetSystemTime(), stored in a word no routine reads. These routines are reached although no call or registration names them, so they count as live.
- **Routine tables in module data (High).** A routine whose address sits in a table in module data is reached when a reached routine reads that table and calls or registers the address. Pack 1427 is the only module where this adds host calls: its Init registers eight script triggers from such a table, which reaches its pending-slot helper (row 217). No other level module holds a routine address in its data apart from the constructor lists and the jump tables below.
- **Links.** A routine is reached when a chain of links leads to it from a root. A link is a direct call, or a registration that stores the routine as a callback: script trigger, entity proximity, teleport, on-enter, item ask-about, activate-world, hero death, item pickup, use item, buy item, AI activate, world swap, script-group type, or a control-thread start. A routine whose address is stored this way is reached even though nothing calls it directly; the host calls it later through the registered pointer.
- **Gates.** A link counts only when no link of the chain sits under a debug-mode test (section 8). Every other branch on the way is an ordinary gate that can pass in play: tests of game and level flags, of module data words (live state, above), of record fields such as the trigger index or the fire state, and floating-point compares (timers, distances, hit-point sums).
- **Jump tables.** Each target of a jump table inside a routine is part of that routine. A trigger routine registered under several indices that switches on the record's index runs every case whose index is registered.
- **Not reached.** A routine that no call, registration, constructor list or table names never runs. A per-level spec may call such a routine dormant. A routine reached only through a debug-gated link is "never runs in play" (section 8).

**Disc call-site counts (High).** The disc count of a host call is the number of call instructions to that host row in level-module code that runs in play. A routine runs in play when the reachability rule above reaches it from Init, Update, Exit or the load and unload hooks, through calls and registrations, with no link under a debug-mode test. Data-dependent gates still count. Each distinct module payload is counted once, so a pack whose module repeats another (Repeated modules, Where it lives on the disc) adds nothing. A site counts once however many routes reach it. Sites in routines nothing references, and sites reached only under a debug-mode test, are not counted. Overloads of one name are counted per row. The fade and letterbox counts of rows 136 to 144 and Open question 88 use this rule, and so do the fade and letterbox counts of SPEC-0052 (Screen effect, Disc use). A count that names another basis, such as all 173 modules, keeps that basis.

Coverage: from the sampled modules, roughly a quarter of the original logic is direct action lists, another quarter is state machines that need named variables, and the remaining share needs loops, wait polls or arithmetic. The per-level specs must describe those parts explicitly. The clean side must not infer behavior from the original code.

All 117 callback and thread entry points in 86 modules that the function index of the level modules omits have been decoded; they were only missing from the index (see the section on callbacks that the index does not list). Each is described in its module's section.

### 8. Shared rule: debug-gated branches (Confidence: High)

- The debug-mode query (IsDebugMode) returns 0 on every path in this build. Every branch gated by it is therefore dead in play: the script log lines, the debug-only calls, and button code 7 of IsButtonPressed never run. The query is imported by 53 distinct module files (packs that repeat another payload share its module). Of the 84 item-grant call sites in the level scripts, only 27 are debug-gated; the other 57 run in play (SPEC-0021).
- **What counts as debug-gated.** A call to the debug query gates code only when its result is tested by a branch that skips that code. Several modules call a small debug-print helper that makes the debug query, prints a log line when it is set, and returns without a result. The caller never tests anything from it, so the code after a call to such a helper runs in play. A common case: a control thread calls the helper before each slot load; those slot loads are not debug-gated and run in play under their own flag tests.
- Every per-level spec applies this rule. A debug-gated call is never listed as a game effect; it is marked "never runs in play". A call that only follows a debug-print helper is listed as a normal game effect.

### 9. Small key-value table (Confidence: High for the rule)

A host-global table of 16 entries, each a (key, value) pair. Both routines below were confirmed in full. The script-facing wrapper discards their return values, so a script does not receive a success/failure result from append or set. Module 1174 uses key 1 for its Act 2 start record and keys 5–53 for hazard, phase/checkpoint, breakable, barrier-trigger and floor-tile records; this is a module-specific use, not a complete key map.

- **Append**: if the table has a free entry, the pair is stored there and the entry's index is returned. The table is not extended beyond 16 entries; when no entry is free the call returns -1.
- **Set**: the table is searched for the key. When it is found, its value is replaced and its index is returned. When the key is absent, the call returns -1.

Synthetic check: appending (7, 100) to an empty table returns 0; a later set of key 7 to 250 returns 0 and leaves the table with (7, 250).

No other script uses the table and nothing reads a full-table result in shipped play (Open question 34).

### 10. Id registry with alignment (Confidence: Medium)

- A new id is accepted only when it is a multiple of 128 (its low seven bits are zero).
- The id is written only into an empty slot of the registry. If the slot is taken, or the id is not aligned, the call fails and returns -1.
- The registered values are runtime addresses of DLL class instances. The registry passes an accepted value to the DLL manager and records it on success; registration and unregistration are used by object-lifecycle paths. The specific DLL classes and any level-script API consumer remain unidentified (High for the registry's class-instance role; Medium for the full set of callers).

Synthetic check: id 256 into an empty slot succeeds; id 200 fails because it is not a multiple of 128; a second id 384 fails when its slot is already taken.

Which id space the registry serves is open (Open questions 35).

### 11. State step and state predicates (Confidence: High for the predicates; Medium for the step)

- **State step on a 0x3C-byte record.** State 0 or state 4 fails. State 2 moves to state 5. Any other state, including state 5, moves to state 6. Every state other than 0 and 4 gives success.
- **Global object predicates.** The global object is tested by three predicates: it has state word 0 and type word 4; it has state word 7 or 8 with sub-state 0; or it has state word 9 with sub-state 0.
- **Mode byte.** A routine sets the object's mode byte to 3 when the byte is 2 or 3, and to 1 otherwise. It clears the object's position vector, and when the object's kind is 4, 8 or 10 it runs one more routine.

Which scripts use these states, and what the state numbers mean, are open (Open questions 36).

### 12. Loading a module by id (Confidence: High for the known image-buffer path)

The known path accepts an aligned module-image buffer from world-data parsing or boot startup. It validates the module container (signature, version byte '1' or '2', alignment; a bad image is refused with a nonzero code and nothing is registered), relocates the image in place, and asks the module manager to register it. The manager's key is the image header's name reference at +0x14 (after relocation, the address of the module-name string, the same string in every shipped module), the image address and a check word. After linking the image the loader calls the load hook at header +0x18 once, with no arguments, when that field is set, and only then asks the manager to register it; it returns 0 when registration succeeds. If registration fails because the manager's 1,024-entry table is full, it returns 6, but the load hook has already run by then (High: the hook call comes before the registration in the routine; no shipped run reaches it). This path has no lookup or reuse branch for an already registered image: the manager never compares keys.

The loader's init field is the module container's load hook at +0x18 (section 1), not a separate field, and no path reuses a loaded record: a holder that still has a linked module is unlinked first, which runs the unload hook and frees its manager slot (Open question 37, resolved). A second form checks byte +3 of the SNR image header for ASCII '1'; this is a format-version check, not a module-name test. It asks the manager to unload the exact image pointer first, and proceeds only when that pointer is linked. This form copies and relocates the image, calls the image load hook if set, then registers its id. No direct callsite for this form appears in the known boot, level-module, or DLL records, so no on-disc module name is shown reaching it (Open question 39).

### 13. Object changes by pack (Confidence: High for the sites, the object numbers that are listed and the flag conditions; Medium for the "Persists" column and for the words "other script tests"; Medium-Low for rows marked table-driven or table/argument)

This section lists the places where a level module's script calls change a placed object, a breakable, a placed effect, a character set or a world layer word: InitObjOn, ActivateObj, DeactivateObj, PlayAnim, LoopAnim, StopAnim, SetObjToTime, SetObjToMayaFrame, the locator-follow calls, the offset, rotation, colour and speed setters, SetWorldCollisMask, SetWorldDisplayMask, SetBreakableActive, MakeBreakable, Break, StartEffect, StopEffect, SlotLoad, SlotReset, the movable-block calls, SetTargetActive, ItemActivate, AddItemToWorld, DeactivateEntity, ActivateEntity, CharacterActivate and CharacterDeactivate. It covers 3,612 sites in 125 of the 163 distinct module images. The 38 packs with no such site are 685, 748, 749, 752, 776, 783, 786, 787, 788, 789, 797, 798, 809, 851, 855, 856, 857, 858, 906, 950, 955, 984, 1003, 1074, 1080, 1170, 1172, 1220, 1225, 1245, 1253, 1254, 1255, 1256, 1259, 1299, 1427 and 1429. Slot creation, entity walks and the flag writes are not object changes and are not listed.

How to read the tables. Each row is a group of sites of one pack that make the same change under the same condition.
- **Object / part**: the object, breakable, effect, entity or character-set numbers (ranges are collapsed; "table/argument" means the number is computed from module data or passed in by a caller, so it is not a fixed list). For the two layer-word calls the row has no object.
- **When**: what runs the site. "at load" is the module's load routine or a helper it calls (once per load of the pack). "every tick" is its update. "world-activate callback" and "on-enter callback" run every time the world becomes current, so also after a return from an interior (the exterior is not reloaded then). "script trigger N", "entity proximity N", "teleport callback", "animated trigger N", "breakable hook N", "script-group callback" and "thread" are event handlers; they run only while the pack's world is current.
- **Condition**: the tests that must hold, in words. Flags are game-state flags. "other script tests" stands for module state words, button presses, timers, distances and counters. Debug-only sites (17 in packs 985, 1096 and 1174) never run in play and are left out.
- **Change**: the effect (Behavior section 2, Category 1 rows; objects are hidden at disc state until a call shows them; DeactivateObj also removes the object's walls and ground at once; PlayAnim and LoopAnim set the object's playing state unconditionally, so they bring back an object that was hidden or deactivated, with its walls and ground from the next object step (SPEC-0028, Run-time object changes)).
- **Persists** (what makes the change survive a reload of the pack; the game saves flags, broken bits, kill bits and the collected table, and nothing about placed objects, effects, level state or module words):
  - "rebuilt at each load" / "rebuilt each tick" / "re-run when the world becomes current": the site itself tests its condition there, so the result follows the flags.
  - "rebuilt at each load (thread started at load)": a control thread that the load routine starts tests the flag when it runs, so the result follows the flags at each load unless the thread first waits for an event.
  - "flag N read at load": the event handler writes flag N (itself or through a routine it calls) and the pack's load or update routine tests N.
  - "kill bit, saved": DeactivateEntity or CharacterDeactivate by pointer; the character stays gone after a reload.
  - "broken bit, saved": Break sets the bit for script indexes below 128; on disc data this does not keep the breakable broken (a breakable with its bit set loads intact), so the packs that need it gone also disable it from a flag at load.
  - "no, reverts at reload": the handler writes no flag the pack tests; the object returns to its load state at the next reload of the pack. For cutscene props this is intended. An exterior is not reloaded by an interior visit, so inside a visit the change stays.
  - "flag written, not read at load by this pack": the handler writes flags but this pack never tests them at load; another pack or the main program may. Check before relying on it.
  - "level state only, not saved": the handler writes only level-state flags, which are cleared each time the world's buffer is released for a load (they survive swaps and interior visits of a resident exterior).
  - "unclear": the site is reached through a table or an unlisted routine.

Rules for a rewrite (High):
1. Placed objects start hidden (state 3 almost without exception on disc). A flag-chosen setup shows the objects that belong to the flag value by calling InitObjOn or ActivateObj; an object with no call in that case is simply absent, with no walls and no ground.
2. A flag is tested where the site runs. A scene that sets a flag does not move an object unless the same scene (or a tick routine that tests the flag) calls the object's setter; the load routine picks the flag up at the next load.
3. Some changes are chained to the state of another object through a timer or a state word, not through the object's visibility (pack 1174: the west balcony rails follow the state word of pillar tile 42 and tile 43, SPEC-0044; pack 837: the wall layer words follow breakable 2). A rewrite has to keep the state words.
4. The exterior world keeps its object states through interior visits. A rewrite that rebuilds the exterior on return loses every event-only change and re-runs the load routine instead; the "world-activate callback" rows are the ones the original repeats on return.
5. Init-time calls with no world argument (or world 0) act on the current world; Init runs before the world is made current. In 13 exterior packs (735, 758, 772, 777, 822, 842, 956, 986, 1091, 1249, 1324, 1327, 1382) there are 52 such Init-reached sites (interior and transition modules pass their own world at all 95 of theirs). They act on the owning exterior only when it is the current world at load (a script replacement load, a save load or Continue while E is current, a return-world movie hand-off while E is current); when the exterior is streamed in, or a game starts or loads from the title, or a movie hand-off reloads E with T or I current, they act on T or I at the same object number and E's object stays in its disc state. Resolved (Medium-High): a rewrite must reproduce this by resolving the sites against the current world at the time Init runs (SPEC-0028, Run-time object changes, Which world is current while an exterior's Init runs).

Reload behaviour summary (event-only changes; Confidence: High for the routine list, Medium for the classes marked Medium in the pack sections).

A change that no flag read at load, at each tick or at world activation records is undone at the next rebuild of its pack. An interior or transition pack is rebuilt on every entry. An exterior pack is rebuilt only by a load of the exterior: a game load, a continue after a defeat, a new game, an exterior replacement load, or leaving for another exterior and coming back. A visit to an interior does not rebuild the exterior, which keeps every changed object. 120 routines that show, hide, enable or move something visible belong to this group. They fall into three classes (counts: a 83, b 21, c 16):
- Class a, intended and harmless (83 routines): cutscene props that are hidden again, plates and pressure pads that undo themselves, minigame resets, fight toggles whose state is held by fight flags, and transition packs that are rebuilt on every entry. A rewrite loses nothing by rebuilding them.
- Class b, gameplay-visible (21 routines in packs 790, 990, 1098, 1171, 1244, 1382 and 1392): a gate, lever, platform, panel or pedestal returns to its start state after a reload. In every case but pack 1244 the player can repeat the action, so nobody is locked out and no content is lost; pack 1392 skips its three lever pulls after the start; pack 1244 may seal a route (see its section). Keep each of these as the original does, including the odd ones.
- Class c, persistent through a mechanism that is easy to miss (16 routines in packs 985, 1078, 1248, 1300, 1353, 1371, 1381, 1398 and 1411): a flag read by a table-driven helper, a thread that Init starts, the activate-world callback, or the exit routine rebuilds the change. A rewrite must keep the flag and the place where it is read.

The change is also visible if a rewrite errs the other way. Making a class a or b change persistent shows the post-event state after a reload where the original shows the start state (levers pulled, gates open, rails gone, pedestals lit). Dropping a class c rebuild shows the start state where the original shows the finished state. Each pack section has a "Reload behaviour" bullet with the exact scenario.

| Pack | Family | Routines (a / b / c) | What happens at a reload |
| --- | --- | --- | --- |
| 0735 | Exterior | 3 (3 / 0 / 0) | effect-scene props hidden again; bobbing thread restarts |
| 0758 | Exterior | 2 (2 / 0 / 0) | minigame reset; mask words restart |
| 0790 | Exterior | 2 (0 / 2 / 0) | two levers and their objects 1 and 2 return to start; lever reusable |
| 0822 | Exterior | 1 (1 / 0 / 0) | prop shown before a replacement load |
| 0908 | Transition | 1 (1 / 0 / 0) | swap animations rewound; transition rebuilt each entry |
| 0956 | Exterior | 1 (1 / 0 / 0) | hides objects that no call shows; flags gate the trigger |
| 0985 | Exterior | 1 (0 / 0 / 1) | arena walls re-armed by the activate-world callback; flag 214 keeps the Gnarfas arena open |
| 0990 | Exterior | 2 (1 / 1 / 0) | switch pairs and lever-lifted walkway return to start; reusable |
| 1023 | Exterior | 2 (2 / 0 / 0) | package prop hidden again; item removal and flags kept; mask cycle restarts |
| 1077 | Exterior | 2 (2 / 0 / 0) | threads restart; lever results rebuilt from flags 38, 36, 306 |
| 1078 | Exterior | 2 (1 / 0 / 1) | plate doors close when released; lever doors kept by table flags 669 to 674 |
| 1079 | Transition | 1 (1 / 0 / 0) | plate swaps restored; transition |
| 1089 | Transition | 1 (1 / 0 / 0) | timed crusher run; transition |
| 1091 | Exterior | 5 (5 / 0 / 0) | stepping stones and rope targets restart |
| 1096 | Exterior | 2 (2 / 0 / 0) | tower stages restart while flag 45 is clear |
| 1098 | Exterior | 1 (0 / 1 / 0) | lever pairs and panel object 3 return to start; stage gates make them reusable |
| 1122 | Exterior | 1 (1 / 0 / 0) | intro prop only |
| 1130 | Transition | 3 (3 / 0 / 0) | button pairs swap back; transition |
| 1171 | Exterior | 5 (3 / 2 / 0) | three gate levers: gates 0 to 2 close again after a reload; reusable |
| 1174 | Exterior | 3 (3 / 0 / 0) | rails and pillars rebuilt at every arrival (run restarts) |
| 1217 | Exterior | 15 (15 / 0 / 0) | fight toggles; fight state in flags 82, 83, 145, 278 |
| 1244 | Exterior | 1 (0 / 1 / 0) | rock-fall scene undone by a reload while flag 272 is set and 273 clear; scene cannot repeat |
| 1247 | Exterior | 5 (5 / 0 / 0) | plate gates close when released; arrival gate closes again at a reload |
| 1248 | Exterior | 4 (1 / 0 / 3) | switch objects kept only for one combination (flag 79 at unload) |
| 1249 | Exterior | 5 (5 / 0 / 0) | plate gates close when released; levers kept as all-up (flag 80) |
| 1300 | Exterior | 1 (0 / 0 / 1) | ritual objects kept by flag 594 |
| 1353 | Exterior | 1 (0 / 0 / 1) | statue puzzle rebuilt from flag 694 (turn angles are not kept) |
| 1358 | Exterior | 1 (1 / 0 / 0) | mask minigame restarts |
| 1363 | Exterior | 1 (1 / 0 / 0) | pressure plate swaps back |
| 1371 | Exterior | 1 (0 / 0 / 1) | wave gates kept by flags 342 to 344; mid-wave reload resets the cage |
| 1381 | Exterior | 7 (2 / 0 / 5) | stations and blocks rebuilt from flags 708 to 719 at every activation |
| 1382 | Exterior | 5 (0 / 5 / 0) | pedestals, switches and gate scene reset; kill bits and flags 98, 337 kept |
| 1392 | Exterior | 9 (0 / 9 / 0) | reload with flag 696 set shows all three levers pulled and objects 11, 13, 15 drawn |
| 1393 | Exterior | 14 (14 / 0 / 0) | whole-module reset; no flag read |
| 1394 | Exterior | 6 (6 / 0 / 0) | timed puzzle restarts |
| 1398 | Exterior | 2 (0 / 0 / 2) | finished stages kept by flags 720 to 724; unfinished stage re-armed |
| 1411 | Exterior | 1 (0 / 0 / 1) | shrine light sequences not replayed (flags 688 to 691) |

Quirks worth keeping (High unless marked):
1. Pack 1392 treats all three gate levers as pulled whenever flag 696 is set at load, whether or not they were pulled.
2. Pack 1244 tests flag 273 at load but its scene ends by setting flag 272, so a reload between the two shows the sealed layout and the scene cannot run again (Medium).
3. Pack 1248 keeps its switch results only if, when the pack is unloaded, objects 0 and 2 are inactive and object 1 is active; every other combination is forgotten.
4. A broken bit never keeps a breakable broken (the start-mode pass revives it), so packs that need a breakable gone also disable it from a flag at load; the break scenes of packs 1171 and 1244 are undone by a reload except for those flag disables.
5. Pack 985 re-arms its arena walls at every world activation in the Gelert branch, and only while flag 214 is clear in the Gnarfas branch (Medium).

Test vectors:
- Pack 706, flag 205 clear at load: collision word 1 and display argument 1 (display word 2); flag 205 set: collision word 2 and display argument 2 (display word 4).
- Pack 1184, flag 277 clear: every tick collision word 1 and display argument 1; flag 277 set: collision word 2 and display argument 2.
- Pack 1066: object 0, every tick: with flag 2 set and object 0 shown, object 0 is hidden (DeactivateObj) in the same tick.
- Pack 934, flags 470 and 474 set and object 0 hidden: object 0 is shown at its start pose on the next tick; both flags clear and object 0 shown: it is hidden on the next tick.
- Pack 1392 (reload with the three gate levers): flag 696 clear at load: levers 20, 22 and 24 at their start pose, effects 13 to 18 stopped, objects 11, 13 and 15 not shown, and the first Update runs the fall-in. Flag 696 set at load: levers 20, 22 and 24 at their end pose, objects 11, 13 and 15 shown at their start pose, effects 13 to 18 left running, all three lever words 1, so triggers 25 to 27 do nothing.
- Pack 1244 (reload after the rock-fall scene, Werelupe King alive): flags 272 set and 273 clear at load: objects 5, 0, 1 and 4 shown at their start pose, object 6 not shown, breakables 17 to 24 intact, triggers 0, 3 and 4 do not fire. Flag 273 set: none of those objects is initialised.
- Pack 1171 (reload after pulling lever L5): before the pull, lever object 11 is drawn and gate object 2 is at its start pose; after the pull, 12 is drawn and gate 2 is at its end pose; after a reload (not after an interior visit) lever 11 is drawn again, gate 2 is at its start pose, and the lever can be pulled again.

**Pack 0678** (same code as 0686, 0691): Transition, the road-fight scene chain, flag-gated slot setup and a button prompt

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0,9,10 | every tick, thread | none | entity removed permanently (kill bit) | kill bit, saved |
| - | every tick, thread | none | character slot emptied (characters released) | flag 126,146 read at load |
| 3 | every tick, thread | none | character set loaded into a slot | flag 126,146 read at load |
| 10 | every tick, thread | none | entity force-spawned (kill bit cleared) | flag 126,146 read at load |
| 5,6 | every tick, thread | none | character released (streaming may rebind) | kill bit, saved |
| 2 | table-driven | flag 148 set and flag 149 clear and other script test and other script test | character set loaded into a slot | unclear |
| - | thread | none | character slot emptied (characters released) | flag 149 read at load |
| 0,3 | load thread | none | character set loaded into a slot | no, reverts at reload |
| 1 | table-driven | none | character set loaded into a slot | unclear |
| 0,1 | thread | flag 124 set | shown at start pose (draw, collision, pose at time 0) | flag 126 read at load |
| 0,1 | thread | flag 124 set | hidden, collision and walls off | flag 126 read at load |
| 0,1 | thread | none | shown at start pose (draw, collision, pose at time 0) | flag 146 read at load |
| 0,1 | thread | none | hidden, collision and walls off | flag 146 read at load |
| 0,1 | thread | none | shown at start pose (draw, collision, pose at time 0) | flag 149 read at load |
| 0,1 | thread | none | hidden, collision and walls off | flag 149 read at load |

**Pack 0696**: Transition, map triggers, button prompt and intro movie

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | at load | flag 137 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 0-16 | at load | flag 284 set | effect stops | rebuilt at each load |
| - | every tick | flag 220 set and flag 284 clear | ground-block collision layer word := 4 | rebuilt each tick |
| - | every tick | flag 220 set and flag 284 clear | placed-object display layer word := 2 x 15 | rebuilt each tick |

**Pack 0699**: Exterior, world masks and two triggers

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| - | at load | flag 265 clear | placed-object display layer word := 2 x 1 | rebuilt at each load |
| - | at load | flag 265 set | placed-object display layer word := 2 x 2 | rebuilt at each load |
| - | at load | none | ground-block collision layer word := a computed word | rebuilt at each load |

**Pack 0702**: Transition, button prompt and a message on entry

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| - | at load | flag 252 clear | ground-block collision layer word := 1 | rebuilt at each load |

**Pack 0706**: Transition, flag-gated setup, character sets and a teleport

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0,1 | load thread | flag 67 clear and flag 66 clear and flag 65 clear and flag 64 clear and flag 63 clear | character set loaded into a slot | rebuilt at each load |
| 2 | load thread | flag 67 clear and flag 66 clear and flag 65 clear and flag 64 clear | character set loaded into a slot | rebuilt at each load |
| 3 | load thread | flag 67 clear and flag 66 clear and flag 65 clear | character set loaded into a slot | rebuilt at each load |
| 4 | load thread | flag 67 clear and flag 66 clear | character set loaded into a slot | rebuilt at each load |
| 5 | load thread | flag 67 set | character set loaded into a slot | rebuilt at each load |
| 0 | at load | flag 248 set and item 14 not held | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| - | at load | flag 205 clear | ground-block collision layer word := 1 | rebuilt at each load |
| - | at load | flag 205 clear | placed-object display layer word := 2 x 1 | rebuilt at each load |
| - | at load | flag 205 set | ground-block collision layer word := 2 | rebuilt at each load |
| - | at load | flag 205 set | placed-object display layer word := 2 x 2 | rebuilt at each load |

**Pack 0714**: Transition, intro movie trigger and button prompt

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| table/argument | load thread | none | character set loaded into a slot | no, reverts at reload |
| 9-15 | at load | flag 284 set | effect stops | rebuilt at each load |
| - | every tick | flag 220 set and flag 284 clear | ground-block collision layer word := 4 | rebuilt each tick |
| - | every tick | flag 220 set and flag 284 clear | placed-object display layer word := 2 x 15 | rebuilt each tick |

**Pack 0722**: Exterior, sixteen entity proximity registrations in eight routines, a betting race, signs, five interior doors and a slot-load thread

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 1,2,7 | load thread | flag 76 clear and flag 75 clear | character set loaded into a slot | rebuilt at each load |
| 3 | load thread | flag 76 set | character set loaded into a slot | rebuilt at each load |
| 0,4 | load thread | flag 76 set and flag 562 set | character set loaded into a slot | rebuilt at each load |
| 5 | load thread | flag 76 set and flag 569 set | character set loaded into a slot | rebuilt at each load |

**Pack 0732**: Transition, two entity lines, a blackboard value per teleport index

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 1 | load thread | flag 561 clear or flag 562 set | character set loaded into a slot | rebuilt at each load |
| 0 | load thread | not (flag 561 clear or flag 562 set) | character set loaded into a slot | rebuilt at each load |

**Pack 0735**: Exterior, hero-death and activate-world callbacks, a set of triggers and a cutscene chain

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| - | script trigger 21 | none | character slot emptied (characters released) | flag written, not read at load by this pack |
| 5 | script trigger 21 | none | character set loaded into a slot | flag written, not read at load by this pack |
| 0 | script trigger 21 | none | character activated | flag written, not read at load by this pack |
| 0 | script trigger 21 | none | character released (streaming may rebind) | kill bit, saved |
| 1,3,4 | script trigger 4/5 | flag 263 clear | shown at start pose (draw, collision, pose at time 0) | no, reverts at reload |
| 1-4 | script trigger 4/5 | flag 263 clear | animation plays once | no, reverts at reload |
| 1,2 | script trigger 4/5 | flag 263 clear | effect starts | no, reverts at reload |
| 1,2 | script trigger 4/5 | flag 263 clear | effect stops | no, reverts at reload |
| 1,3,4 | script trigger 4/5 | flag 263 clear | hidden, collision and walls off | no, reverts at reload |
| - | load thread | none | character slot emptied (characters released) | flag 263 read at load |
| 4 | load thread | none | character set loaded into a slot | flag 263 read at load |
| 0 | load thread | none | character activated | flag 263 read at load |
| 12,54 | load thread | none | hidden, collision and walls off | flag 263 read at load |
| 0 | load thread | none | breakable enabled (intact) | flag 263 read at load |
| 0 | load thread | none | breakable broken by script | broken bit, saved |
| 0,13 | load thread | none | shown at start pose (draw, collision, pose at time 0) | flag 263 read at load |
| 0 | load thread | none | animation plays once | flag 263 read at load |
| table/argument | table-driven | other script test | breakable made breakable | unclear |
| table/argument | table-driven | other script test | breakable broken by script | broken bit, saved |
| table/argument | table-driven | none | animation plays once | unclear |
| table/argument | table-driven | none | hidden, collision and walls off | unclear |
| table/argument | table-driven | none | effect starts | unclear |
| table/argument | table-driven | none | shown at start pose (draw, collision, pose at time 0) | unclear |
| table/argument | table-driven | none | effect stops | unclear |
| table/argument | load thread | other script test | pose set to a computed time and frozen | no, reverts at reload |
| table/argument | at load | other script test | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 14-16 | at load | none | animation loops | rebuilt at each load |
| 12,table/argument | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 0 | at load | none | breakable disabled | rebuilt at each load |
| 12 | world-activate callback | flag 263 clear | shown at start pose (draw, collision, pose at time 0) | re-run when the world becomes current |
| 12,54 | world-activate callback | flag 263 set | hidden, collision and walls off | re-run when the world becomes current |
| 0,13 | world-activate callback | flag 263 set | shown at start pose (draw, collision, pose at time 0) | re-run when the world becomes current |
| 2 | load thread | flag 76 clear and flag 75 clear and flag 261 set and (flag 530 clear or flag 531 set) | character set loaded into a slot | rebuilt at each load |
| 3 | load thread | flag 76 clear and flag 75 clear and flag 261 set and flag 531 set | character set loaded into a slot | rebuilt at each load |
| 3 | load thread | flag 76 clear and flag 75 clear and flag 530 set and flag 531 clear | character set loaded into a slot | rebuilt at each load |
| 2 | load thread | flag 76 clear and flag 75 clear | character set loaded into a slot | rebuilt at each load |
| table/argument | load thread | flag 76 set and flag 530 set and flag 531 clear | character set loaded into a slot | rebuilt at each load |
| 0 | load thread | none | character set loaded into a slot | no, reverts at reload |
| 1 | at load | none | effect stops | rebuilt at each load |

**Pack 0742**: Transition, proximity conversation and two watcher threads

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 76 clear and flag 75 clear | character set loaded into a slot | rebuilt at each load |
| 0 | load thread | flag 76 clear | character set loaded into a slot | rebuilt at each load |
| 0 | load thread | flag 76 set | character set loaded into a slot | rebuilt at each load |
| 1 | load thread | flag 76 set and flag 561 set and flag 562 clear | character set loaded into a slot | rebuilt at each load |

**Pack 0745**: Interior, four entity proximity routines, an on-enter callback and a flag-gated item exchange

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0,1 | load thread | flag 76 clear and flag 75 clear | character set loaded into a slot | rebuilt at each load |
| 0 | load thread | flag 76 clear | character set loaded into a slot | rebuilt at each load |
| 1 | load thread | none | character set loaded into a slot | no, reverts at reload |

**Pack 0750**: Interior, on-enter line, four proximity lines and a flag-gated line set

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 76 clear and flag 75 clear | character set loaded into a slot | rebuilt at each load |

**Pack 0753**: Exterior, entity proximity, teleport and an effect group

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 15-48 | at load, load thread, script trigger 1 | other script test | effect stops | flag 258 read at load |
| 15-48 | at load, load thread, script trigger 1 | not other script test | effect starts | flag 258 read at load |
| 0 | load thread | none | character released (streaming may rebind) | kill bit, saved |
| 0 | script trigger 1 | other script test and movie 36 not played | character activated | level state only, not saved |
| - | at load | flag 252 clear | ground-block collision layer word := 1 | rebuilt at each load |

**Pack 0757**: Exterior, boss-arena style encounter with a camera and AI state

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 2-5 | at load | none | shown at end pose (finished) | rebuilt at each load |

**Pack 0758**: Exterior, many entity proximity lines, a script-trigger set with a button prompt and breakable checks, and a world-swap callback

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| - | load thread, thread | none | character slot emptied (characters released) | no, reverts at reload |
| 0,12 | load thread | none | character set loaded into a slot | no, reverts at reload |
| table/argument | load thread | none | character activated | no, reverts at reload |
| - | thread | other script test | character slot emptied (characters released) | no, reverts at reload |
| table/argument,? | thread | other script test | character set loaded into a slot | no, reverts at reload |
| table/argument | thread | other script test | character activated | no, reverts at reload |
| table/argument | thread | other script test | character released (streaming may rebind) | kill bit, saved |
| - | table-driven | none | character slot emptied (characters released) | unclear |
| ? | table-driven | none | character set loaded into a slot | unclear |
| 3,6 | at load, entity proximity 26, entity proximity 28, script trigger 14 | other script test | animation plays once backward from the end | flag written, not read at load by this pack |
| 17 | at load, entity proximity 26, entity proximity 28, script trigger 14 | breakable 17 broken or off | breakable enabled (intact) | flag written, not read at load by this pack |
| 18 | at load, entity proximity 26, entity proximity 28, script trigger 14 | breakable 18 broken or off | breakable enabled (intact) | flag written, not read at load by this pack |
| 19 | at load, entity proximity 26, entity proximity 28, script trigger 14 | breakable 19 broken or off | breakable enabled (intact) | flag written, not read at load by this pack |
| 20 | at load, entity proximity 26, entity proximity 28, script trigger 14 | breakable 20 broken or off | breakable enabled (intact) | flag written, not read at load by this pack |
| 2,3,6 | script trigger 15 | other script test | animation plays once | no, reverts at reload |
| 2 | script trigger 15 | other script test | animation plays once backward from the end | no, reverts at reload |
| table/argument | script trigger 16/17/18/19 | breakable (table) intact | breakable broken by script | broken bit, saved |
| 0,2,3,6 | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| - | at load, script trigger 3 | none | ground-block collision layer word := a computed word | flag written, not read at load by this pack |
| - | at load, script trigger 3 | none | placed-object display layer word := 2 x a computed word | flag written, not read at load by this pack |
| - | table-driven | none | transition scene | unclear |

**Pack 0772**: Exterior, entity conversation chain, a walk-off scene and item prompts

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| - | entity proximity 14 | none | character slot emptied (characters released) | flag written, not read at load by this pack |
| 1 | entity proximity 14 | none | character set loaded into a slot | flag written, not read at load by this pack |
| 2 | world-activate callback | flag 591 set and flag 592 clear | character set loaded into a slot | re-run when the world becomes current |
| 1 | at load | flag 543 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 0 | at load | flag 543 set | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 0 | at load | flag 543 set | animation loops | rebuilt at each load |
| 0 | every tick | other script test and flag 543 set | shown at start pose (draw, collision, pose at time 0) | rebuilt each tick |
| 0 | every tick | other script test and flag 543 set | animation loops | rebuilt each tick |

**Pack 0777**: Exterior, entity proximity dialogue set, script triggers and an AI-death sequence

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| table/argument | script trigger 4/5/6/7... | other script test and flag 256 clear and other script test | animation plays once | unclear |
| table/argument | script trigger 4/5/6/7... | other script test and flag 256 clear and other script test | effect stops | unclear |
| 15 | script trigger 4/5/6/7... | other script test and flag 256 clear and other script test | shown at start pose (draw, collision, pose at time 0) | unclear |
| 14 | script trigger 4/5/6/7... | other script test and flag 256 clear and other script test | hidden, collision and walls off | unclear |
| 12,13 | at load | flag 253 set | effect stops | rebuilt at each load |
| 14 | at load | flag 254 set | effect stops | rebuilt at each load |
| 15 | at load | flag 255 set | effect stops | rebuilt at each load |
| 16 | at load | flag 256 set | effect stops | rebuilt at each load |
| table/argument | at load | other script test | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| table/argument | at load | other script test | shown at end pose (finished) | rebuilt at each load |
| table/argument | at load | other script test and other script test and (other script test or flag (table) clear) | shown at end pose (finished) | rebuilt at each load |
| 1,4 | load thread | flag 592 clear or flag 593 set | character set loaded into a slot | rebuilt at each load |
| 0,2 | load thread | not (flag 592 clear or flag 593 set) | character set loaded into a slot | rebuilt at each load |
| 3 | load thread | flag 591 clear or other script test | character set loaded into a slot | rebuilt at each load |
| 1-6 | at load, thread | other script test | shown at end pose (finished) | flag 593 read at load |
| 1-6 | at load, thread | not other script test and other script test | shown at start pose (draw, collision, pose at time 0) | flag 593 read at load |
| 1,2 | table-driven | none | animation plays once | unclear |
| 3-6 | script trigger 0 | none | animation plays once | flag 252 read at load |
| 0 | thread | none | hidden, collision and walls off | flag 593 read at load |
| - | thread | none | character slot emptied (characters released) | flag 593 read at load |
| 4 | thread | none | character set loaded into a slot | flag 593 read at load |
| 0 | at load | flag 252 set and flag 593 set | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |

**Pack 0784**: Interior, conditional thread, a proximity line and a trigger with a hero walk (no Exit)

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 590 set and flag 591 clear | character set loaded into a slot | rebuilt at each load |
| 2 | script trigger 1 | other script test and flag 543 clear and flag 543 clear and other script test and flag 542 clear | animation plays once | flag 543 read at load |
| 2 | at load | flag 543 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 3 | at load | flag 543 set | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |

**Pack 0790**: Exterior, four triggers, a hero-button prompt and a camera-and-slot thread

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 1,2,18,20 | script trigger 14/15 | other script test | animation plays once | no, reverts at reload |
| 18,20 | script trigger 14/15 | other script test | hidden, collision and walls off | no, reverts at reload |
| 19,21 | script trigger 14/15 | other script test | shown at start pose (draw, collision, pose at time 0) | no, reverts at reload |
| - | thread | other script test | character slot emptied (characters released) | no, reverts at reload |
| 1 | thread | other script test | character set loaded into a slot | no, reverts at reload |
| - | thread | not other script test and other script test | character slot emptied (characters released) | no, reverts at reload |
| 0 | thread | not other script test and other script test | character set loaded into a slot | no, reverts at reload |
| 0-3,18,20 | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |

**Pack 0793**: Transition, one entity line and a thread

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 1 | load thread | flag 417 clear | character set loaded into a slot | rebuilt at each load |
| 0 | load thread | flag 417 set | character set loaded into a slot | rebuilt at each load |

**Pack 0796**: Exterior, a button-gated sequence and a fade-and-load trigger

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0,1,17 | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |

**Pack 0799**: Transition, a button-driven lever sequence and a query-only trigger

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0-2 | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |

**Pack 0800**: Exterior, debug cutscene test chooser and boss-stage threads

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| - | every tick, thread, script trigger 1 | none | character slot emptied (characters released) | flag 55 read at load |
| 7 | every tick, thread | none | character set loaded into a slot | flag 55 read at load |
| - | every tick, thread | none | world item spawned | flag 55 read at load |
| 0 | every tick, thread | none | shown at start pose (draw, collision, pose at time 0) | flag 55 read at load |
| 0,1,5,6 | load thread | flag 58 clear and flag 57 clear and flag 56 clear and flag 55 clear | character set loaded into a slot | rebuilt at each load |
| 2 | load thread | flag 58 clear and flag 57 clear and flag 56 clear | character set loaded into a slot | rebuilt at each load |
| 3 | load thread | flag 58 clear and flag 57 clear | character set loaded into a slot | rebuilt at each load |
| 4 | load thread | flag 58 set | character set loaded into a slot | rebuilt at each load |
| 0 | at load | flag 55 set | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |

**Pack 0816**: Transition, breakable switch and pickup flag

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 11 | at load | flag 516 set and flag 517 clear | breakable disabled | rebuilt at each load |
| 12 | at load | none | breakable disabled | rebuilt at each load |

**Pack 0822**: Exterior, villager and trigger room with a flag-gated conversation

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0,1,4,5,8,9 | load thread | flag 58 clear and flag 57 clear and flag 56 clear and flag 55 clear and other script test | character set loaded into a slot | rebuilt at each load |
| 2,6,10 | load thread | flag 58 clear and flag 57 clear and flag 56 clear and other script test | character set loaded into a slot | rebuilt at each load |
| 3,7,11 | load thread | flag 58 clear and flag 57 clear and other script test | character set loaded into a slot | rebuilt at each load |
| 12 | load thread | flag 58 set and (flag 531 clear or flag 532 set) | character set loaded into a slot | rebuilt at each load |
| 13 | load thread | flag 58 set and flag 532 set | character set loaded into a slot | rebuilt at each load |
| 7 | script trigger 2 | other script test | shown at start pose (draw, collision, pose at time 0) | no, reverts at reload |
| 7 | script trigger 2 | other script test | animation plays once | no, reverts at reload |
| 18-21 | at load, script trigger 8 | none | animation loops | flag 621 read at load |
| 4,5 | at load, script trigger 8 | none | shown at start pose (draw, collision, pose at time 0) | flag 621 read at load |
| 4,5 | at load, script trigger 8 | none | animation plays once | flag 621 read at load |
| 55-60 | at load, script trigger 8 | other script test | effect starts | flag 621 read at load |
| 10-13,15-21 | at load | none | animation loops | rebuilt at each load |
| 55-60 | at load | none | effect stops | rebuilt at each load |
| 14 | script trigger 8 | flag 55 clear and other script test | animation plays once | flag 621 read at load |
| - | script trigger 5/6/7 | other script test | character slot emptied (characters released) | no, reverts at reload |
| 7 | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 14,18-21 | at load | flag 55 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 1-3,14,18 | at load | flag 55 set | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 6 | at load | flag 57 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |

**Pack 0837**: Transition, a flag-driven world mask, a dormant chooser and a break routine called from Update

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| - | at load, every tick | none | ground-block collision layer word := 2 | rebuilt at each load |
| - | at load, every tick | none | placed-object display layer word := 2 x 2 | rebuilt at each load |
| 3 | at load | none | breakable disabled | rebuilt at each load |
| - | at load | none | ground-block collision layer word := 1 | rebuilt at each load |
| - | at load | none | placed-object display layer word := 2 x 1 | rebuilt at each load |
| 0 | at load | flag 636 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 0 | every tick | none | animation plays once | rebuilt each tick |
| 3 | every tick | none | breakable enabled (intact) | rebuilt each tick |
| 3 | every tick | none | breakable broken by script | broken bit, saved |

**Pack 0842**: Exterior, four teleports to interior worlds with a prompt, a story-stage slot thread and quest cutscenes

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| - | every tick, load thread | none | character slot emptied (characters released) | flag 54,56,198 read at load |
| 1 | every tick, load thread | none | character set loaded into a slot | flag 54,56,198 read at load |
| - | every tick, script trigger 2 | none | character slot emptied (characters released) | flag 54,56,59,198 read at load |
| 2,3,6,7 | every tick, script trigger 2 | none | character set loaded into a slot | flag 54,56,59,198 read at load |
| table/argument | at load, load thread, script trigger 1 | other script test | effect stops | flag 53,54,197,198 read at load |
| table/argument | at load, load thread, script trigger 1 | other script test | effect starts | flag 53,54,197,198 read at load |
| 0,1 | load thread | flag 59 clear and flag 58 clear and flag 57 clear and flag 56 clear and flag 55 clear and flag 54 clear | character set loaded into a slot | rebuilt at each load |
| 7 | load thread | flag 59 clear and flag 58 clear and flag 57 clear and flag 56 clear and flag 55 clear | character set loaded into a slot | rebuilt at each load |
| 3 | load thread | flag 59 clear and flag 58 clear and flag 57 clear and flag 56 clear | character set loaded into a slot | rebuilt at each load |
| 4 | load thread | flag 59 clear and flag 58 clear and flag 57 clear | character set loaded into a slot | rebuilt at each load |
| 7 | load thread | flag 59 clear and flag 58 clear | character set loaded into a slot | rebuilt at each load |
| 6 | load thread | flag 59 set | character set loaded into a slot | rebuilt at each load |
| 0 | at load, load thread, script trigger 1 | not other script test | hidden, collision and walls off | flag 53,54,197,198 read at load |
| 0 | at load, load thread, script trigger 1 | not other script test | shown at start pose (draw, collision, pose at time 0) | flag 53,54,197,198 read at load |
| 0 | at load, load thread, script trigger 1 | none | shown at start pose (draw, collision, pose at time 0) | flag 53,54,197,198 read at load |
| 1 | at load | flag 198 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 1 | every tick | flag 198 set and object 1 shown | hidden, collision and walls off | rebuilt each tick |

**Pack 0859**: Interior, proximity dialogue tree with an item-count check and a buy-item line

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| - | entity proximity 0 | flag 506 set and flag 507 clear and other script test | character slot emptied (characters released) | flag written, not read at load by this pack |
| 0 | entity proximity 0 | flag 506 set and flag 507 clear and other script test | character set loaded into a slot | flag written, not read at load by this pack |
| 0 | load thread | flag 507 set | character set loaded into a slot | rebuilt at each load |

**Pack 0864**: Exterior, four entity proximity routines, a boss stage thread, an exit cutscene and a per-tick animated-object sequence

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| - | entity proximity 45, entity proximity 47, entity proximity 48 | none | character slot emptied (characters released) | flag written, not read at load by this pack |
| - | entity proximity 43 | none | character slot emptied (characters released) | no, reverts at reload |
| - | thread | other script test | character slot emptied (characters released) | no, reverts at reload |
| table/argument | thread | other script test | character set loaded into a slot | no, reverts at reload |
| 1 | table-driven | flag 58 clear | character set loaded into a slot | unclear |
| 0 | table-driven | flag 58 set | character set loaded into a slot | unclear |
| table/argument | table-driven | other script test | pose set to a computed time and frozen | unclear |
| table/argument | table-driven | other script test | animation loops | unclear |
| table/argument | table-driven | none | animation loops | unclear |
| table/argument | table-driven | none | pose set to a computed time and frozen | unclear |
| table/argument | table-driven | other script test | animation plays once | unclear |
| 15,16,table/argument | table-driven | none | animation plays once | unclear |
| 16,table/argument | table-driven | object (table) not animating | effect stops | unclear |
| table/argument | every tick | none | animation plays once | rebuilt each tick |
| 44 | table-driven | none | entity removed permanently (kill bit) | kill bit, saved |
| 35,table/argument | table-driven | none | effect starts | unclear |
| 35 | table-driven | other script test and flag 293 set and other script test | effect stops | unclear |
| 93,94 | table-driven | other script test and flag 293 set and other script test | animation plays once | unclear |
| - | table-driven | other script test and flag 293 set and other script test | character slot emptied (characters released) | unclear |
| table/argument | every tick | object (table) finished | pose set to a fixed time and frozen | rebuilt each tick |
| table/argument | every tick | object (table) finished | effect starts | rebuilt each tick |
| table/argument | every tick | object (table) finished | animation plays once | rebuilt each tick |
| table/argument | every tick | object (table) finished | effect stops | rebuilt each tick |
| table/argument | every tick | other script test | effect stops | rebuilt each tick |

**Pack 0874**: Transition, two entity talks, a camera and a slot thread

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| - | entity proximity 6 | none | character slot emptied (characters released) | flag written, not read at load by this pack |
| 0 | load thread | flag 58 clear | character set loaded into a slot | rebuilt at each load |
| 1 | load thread | flag 58 set | character set loaded into a slot | rebuilt at each load |
| 2 | load thread | flag 291 clear | character set loaded into a slot | rebuilt at each load |
| 3 | load thread | flag 290 clear | character set loaded into a slot | rebuilt at each load |

**Pack 0880**: Exterior, boss-entry triggers, a treasure-map exchange and a slot thread

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 52 clear and flag 51 clear and flag 50 clear and flag 49 clear | character set loaded into a slot | rebuilt at each load |
| 1 | load thread | flag 52 clear and flag 51 clear and flag 50 clear | character set loaded into a slot | rebuilt at each load |
| 2 | load thread | flag 52 clear and flag 51 clear | character set loaded into a slot | rebuilt at each load |
| 3 | load thread | flag 52 set | character set loaded into a slot | rebuilt at each load |
| - | script-instance trigger | flag 296 clear | character slot emptied (characters released) | no, reverts at reload |
| 4 | script-instance trigger | flag 296 clear | character set loaded into a slot | no, reverts at reload |
| - | script-instance trigger | none | character slot emptied (characters released) | no, reverts at reload |
| 0,1 | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| - | at load | flag 252 clear | ground-block collision layer word := 1 | rebuilt at each load |

**Pack 0886**: Exterior, a dialogue-prompt trigger, a camera-and-slot thread and an AI sequence

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| - | table-driven | other script test and flag 251 clear | character slot emptied (characters released) | unclear |
| 4 | table-driven | other script test and flag 251 clear | character set loaded into a slot | unclear |
| 3 | every tick | flag 250 clear and entity 16 dead and entity 17 dead and entity 18 dead | hidden, collision and walls off | rebuilt each tick |
| 3 | at load | flag 250 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| - | at load | flag 252 clear | ground-block collision layer word := 1 | rebuilt at each load |

**Pack 0893**: Exterior, slot setup, a slot-load thread, an item-counting exchange, a boss check and button and dialogue triggers

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| - | entity proximity 41 | flag 526 clear | character slot emptied (characters released) | flag 526 read at load |
| 6 | heropickupitemcallback | other script test | character set loaded into a slot | flag 525 read at load |
| - | load thread | none | character slot emptied (characters released) | no, reverts at reload |
| 3 | load thread | not (flag 394 clear and flag 395 clear) and (flag 394 clear or flag 395 set) | character set loaded into a slot | rebuilt at each load |
| table/argument | load thread | none | character set loaded into a slot | no, reverts at reload |
| 36-38 | thread | other script test | entity removed permanently (kill bit) | kill bit, saved |
| - | thread | other script test | character slot emptied (characters released) | flag written, not read at load by this pack |
| 8 | load thread | flag 193 clear | character set loaded into a slot | rebuilt at each load |
| 11 | load thread | flag 524 set and flag 525 clear and item 77 not held | character set loaded into a slot | rebuilt at each load |
| 6 | load thread | flag 524 set and flag 526 clear and flag 525 set | character set loaded into a slot | rebuilt at each load |
| 10 | load thread | flag 47 clear and (flag 490 clear or flag 491 set or flag 64 set) | character set loaded into a slot | rebuilt at each load |
| 9 | load thread | flag 47 clear and flag 64 set | character set loaded into a slot | rebuilt at each load |
| 4 | load thread | flag 47 clear and flag 494 set and flag 495 clear | character set loaded into a slot | rebuilt at each load |
| 7 | load thread | flag 47 set | character set loaded into a slot | rebuilt at each load |

**Pack 0908**: Transition, world-swap with a camera and a hero walk

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0-2,4 | scriptworldswap | none | pose set to a fixed time and frozen | no, reverts at reload |
| 0-2,4 | scriptworldswap | none | animation plays once | no, reverts at reload |
| 0-2,4 | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |

**Pack 0909**: Exterior, object and slot setup, teleports to two interiors, a whinny-pen trigger set and a flag-gated item exchange

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 8 | thread | none | hidden, collision and walls off | flag 706 read at load |
| 8 | thread | none | shown at start pose (draw, collision, pose at time 0) | flag 706 read at load |
| - | every tick | none | character slot emptied (characters released) | rebuilt each tick |
| 16,17 | every tick | none | character set loaded into a slot | rebuilt each tick |
| 6,19 | every tick | none | animation plays once | rebuilt each tick |
| 19 | every tick | none | play speed set (-1.0) | rebuilt each tick |
| 3,4 | every tick | none | effect stops | rebuilt each tick |
| 7,8 | every tick | none | hidden, collision and walls off | rebuilt each tick |
| 6,8 | every tick | none | shown at start pose (draw, collision, pose at time 0) | rebuilt each tick |
| - | thread | none | character slot emptied (characters released) | flag 706 read at load |
| 4,10,16,table/argument | thread | none | character set loaded into a slot | flag 706 read at load |
| - | thread | other script test | character slot emptied (characters released) | flag 706 read at load |
| 2,4,10,14-16,22,23 | thread | other script test | character set loaded into a slot | flag 706 read at load |
| - | thread | other script test and other script test and flag 119 set and flag 121 clear | character slot emptied (characters released) | flag 706 read at load |
| 4,table/argument | thread | other script test and flag 126 clear | character set loaded into a slot | flag 706 read at load |
| 10,table/argument | thread | other script test and flag 122 clear | character set loaded into a slot | flag 706 read at load |
| 4,13,16 | thread | other script test and flag 147 clear | character set loaded into a slot | flag 706 read at load |
| 0,1 | thread | flag 6 clear and flag 5 clear and flag 4 clear and flag 3 clear and flag 2 clear and flag 110 clear | character set loaded into a slot | flag 706 read at load |
| 2 | thread | flag 6 clear and flag 5 clear and flag 4 clear and flag 3 clear | character set loaded into a slot | flag 706 read at load |
| 3 | thread | flag 6 clear and flag 5 clear and flag 4 clear | character set loaded into a slot | flag 706 read at load |
| 9 | script trigger 5 | not other script test | shown at start pose (draw, collision, pose at time 0) | flag 112 read at load |
| 17-20 | script trigger 5 | not other script test | animation plays once | flag 112 read at load |
| 20 | script trigger 5 | not other script test | animation frozen at its current time | flag 112 read at load |
| 9,20 | script trigger 5 | not other script test | hidden, collision and walls off | flag 112 read at load |
| 3,4 | script trigger 5 | not other script test | effect starts | flag 112 read at load |
| 7,8,17-20 | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 3,4 | at load | none | effect stops | rebuilt at each load |
| 4,15,16 | at load | flag 122 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 15,16 | at load | flag 122 set | shown at end pose (finished) | rebuilt at each load |
| 5 | at load | flag 122 set | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 3,10,11 | at load | flag 110 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 10,11 | at load | flag 110 set | shown at end pose (finished) | rebuilt at each load |
| 2 | at load | flag 110 set | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 10,11 | every tick | flag 110 set and other script test | animation plays once | rebuilt each tick |
| 3 | every tick | flag 110 set and other script test | hidden, collision and walls off | rebuilt each tick |
| 2 | every tick | flag 110 set and other script test | shown at start pose (draw, collision, pose at time 0) | rebuilt each tick |
| 15,16 | every tick | other script test and flag 122 set | animation plays once | rebuilt each tick |
| 4 | every tick | other script test and flag 122 set | hidden, collision and walls off | rebuilt each tick |
| 5 | every tick | other script test and flag 122 set | shown at start pose (draw, collision, pose at time 0) | rebuilt each tick |
| - | every tick | flag 117 set and flag 118 clear | character slot emptied (characters released) | rebuilt each tick |
| 18 | every tick | flag 117 set and flag 118 clear | character set loaded into a slot | rebuilt each tick |

**Pack 0934**: Exterior, a timed find-the-petpet game with a yes-or-no prompt, trigger lines, slot threads and an AI-kill gate

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | teleport callback | (flag 470 set or flag 474 set) and object 0 shown | hidden, collision and walls off | flag 470,474 read at load |
| 10,11 | teleport callback | flag 374 set and other script test | animation plays once | flag 470,474 read at load |
| 0,9 | load thread | other script test | character set loaded into a slot | rebuilt at each load |
| - | load thread | other script test | character slot emptied (characters released) | no, reverts at reload |
| 6 | load thread | none | character set loaded into a slot | flag 372 read at load |
| - | load thread | not other script test | character slot emptied (characters released) | flag 372 read at load |
| 4 | load thread | none | character set loaded into a slot | flag 378 read at load |
| - | load thread | none | character slot emptied (characters released) | flag 378 read at load |
| 10,11 | world-activate callback | none | shown at start pose (draw, collision, pose at time 0) | re-run when the world becomes current |
| 0 | script trigger 3/4/5 | other script test | hidden, collision and walls off | flag 470,474 read at load |
| 10,11 | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 0 | every tick | (flag 470 set or flag 474 set) and object 0 hidden | shown at start pose (draw, collision, pose at time 0) | rebuilt each tick |
| 0 | every tick | flag 470 clear and flag 474 clear and object 0 shown | hidden, collision and walls off | rebuilt each tick |

**Pack 0947**: Exterior, two threads, a slot load, a death watch and a hit-point watch

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 375 clear | character set loaded into a slot | rebuilt at each load |

**Pack 0951**: Interior, proximity conversations, a story thread and a per-tick entity wander

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 1 | load thread | flag 220 clear and (flag 2 clear or flag 147 set) and flag 147 clear and (flag 121 clear or flag 123 set) and flag 123 set | character set loaded into a slot | rebuilt at each load |
| 2 | load thread | flag 220 clear and (flag 2 clear or flag 147 set) and flag 147 clear and (flag 121 clear or flag 123 set) and flag 123 set | character activated | rebuilt at each load |
| 1 | load thread | flag 220 clear and (flag 2 clear or flag 147 set) and flag 147 clear and flag 123 set | character set loaded into a slot | rebuilt at each load |
| 0 | at load | flag 121 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 0 | at load | flag 121 set | shown at end pose (finished) | rebuilt at each load |
| 0,1 | every tick | flag 119 set and flag 123 clear and item 98 held and other script test | character activated | rebuilt each tick |

**Pack 0956** (same code as 0970): Exterior, long intro sequence and trigger ladder

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| - | load thread | flag 248 clear and flag 64 clear | character slot emptied (characters released) | rebuilt at each load |
| 7,8 | load thread | flag 248 clear and flag 64 clear | character set loaded into a slot | rebuilt at each load |
| - | load thread | flag 248 set and flag 67 clear | character slot emptied (characters released) | rebuilt at each load |
| 1,9 | load thread | flag 248 set and flag 67 clear | character set loaded into a slot | rebuilt at each load |
| - | thread | flag 67 set | character slot emptied (characters released) | no, reverts at reload |
| 10-12 | thread | flag 67 set and other script test | character set loaded into a slot | no, reverts at reload |
| table/argument | at load | other script test | effect starts | rebuilt at each load |
| table/argument | at load | not other script test and other script test | effect stops | rebuilt at each load |
| 0 | at load | not other script test and other script test | hidden, collision and walls off | rebuilt at each load |
| table/argument | at load | not other script test | effect stops | rebuilt at each load |
| 0,17 | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| - | thread | none | character slot emptied (characters released) | no, reverts at reload |
| table/argument | thread | none | character set loaded into a slot | no, reverts at reload |
| table/argument | script trigger (table) | other script test | shown at end pose (finished) | flag 211 read at load |
| - | script trigger (table) | other script test | character slot emptied (characters released) | flag 211 read at load |
| table/argument | script trigger (table) | other script test | breakable made breakable | flag 211 read at load |
| table/argument | at load | flag 211 clear and other script test | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| table/argument | at load | flag 211 clear and other script test | breakable made unbreakable | rebuilt at each load |
| table/argument | at load | flag 211 set and other script test | shown at end pose (finished) | rebuilt at each load |
| table/argument | at load | flag 211 set and other script test | breakable broken by script | broken bit, saved |
| 17 | animated trigger 9, script trigger 8/9 | object 9 not animating and other script test and other script test | animation loops | flag 211 read at load |
| 9 | animated trigger 9, script trigger 8/9 | object 9 not animating and other script test and other script test | animation plays once | flag 211 read at load |
| 17 | animated trigger 9, script trigger 8/9 | object 9 not animating and other script test and other script test | animation frozen at its current time | flag 211 read at load |
| 9 | animated trigger 9, script trigger 8/9 | object 9 not animating and other script test and other script test | animation plays once backward from the end | flag 211 read at load |
| 9 | at load | none | shown (pose from a variable) | rebuilt at each load |
| 10 | at load | other script test | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 3-5 | script trigger 14 | other script test and flag 214 set and flag 216 clear | hidden, collision and walls off | no, reverts at reload |
| 9 | script trigger 10 | other script test and flag 248 set and flag 281 clear | hidden, collision and walls off | flag 66,281 read at load |
| 10 | script trigger 10 | other script test and flag 248 set and flag 281 clear | shown at start pose (draw, collision, pose at time 0) | flag 66,281 read at load |
| 0-2 | script trigger 10 | other script test and flag 248 set and flag 281 clear | aim target active := true | flag 66,281 read at load |
| 1,2 | at load | flag 214 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 2 | at load | flag 214 set and flag 283 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 0-2 | every tick | other script test and flag 281 clear | aim target active := false | rebuilt each tick |
| 0-2 | every tick | other script test and flag 281 clear | aim target active := true | rebuilt each tick |

**Pack 0985**: Exterior, two fight branches with level-state cutscene chains and a debug skip

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| - | world-activate callback | other script test | ground-block collision layer word := a computed word | re-run when the world becomes current |
| - | every tick, load thread | other script test | ground-block collision layer word := a computed word | flag written, not read at load by this pack |
| 7-12 | every tick, load thread | none | effect stops | flag written, not read at load by this pack |
| 0-4,7,8 | every tick, load thread | none | entity removed permanently (kill bit) | kill bit, saved |
| 5 | load thread | level state 0 clear or other script test | entity removed permanently (kill bit) | kill bit, saved |
| 6 | load thread | level state 0 clear or other script test | entity force-spawned (kill bit cleared) | rebuilt at each load |

**Pack 0986**: Exterior, navmesh table with per-tick character assignment, trigger handlers and a debug-gated per-tick loop

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 86 clear and flag 85 clear | character set loaded into a slot | rebuilt at each load |
| - | at load | none | ground-block collision layer word := a computed word | rebuilt at each load |
| - | at load | none | placed-object display layer word := 2 x a computed word | rebuilt at each load |
| - | at load, script trigger 1 | none | ground-block collision layer word := a computed word | flag 89 read at load |
| - | at load, script trigger 1 | none | placed-object display layer word := 2 x a computed word | flag 89 read at load |
| 0 | script trigger 2/3 | other script test | breakable broken by script | broken bit, saved |
| 7 | script trigger 2/3 | other script test | hidden, collision and walls off | flag 88 read at load |
| 2 | script trigger 1 | other script test | hidden, collision and walls off | flag 89 read at load |
| 1 | script trigger 1 | other script test | shown at start pose (draw, collision, pose at time 0) | flag 89 read at load |
| 1 | script trigger 1 | other script test | animation plays once | flag 89 read at load |
| 3,4,8 | script trigger 7 | other script test | animation plays once | flag 87 read at load |
| 3,4,6 | script trigger 7 | other script test | shown at start pose (draw, collision, pose at time 0) | flag 87 read at load |
| 5 | script trigger 7 | other script test | hidden, collision and walls off | flag 87 read at load |
| 2-5,7,8 | at load | other script test | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 2 | at load | other script test | animation loops | rebuilt at each load |
| 1,3,4,8 | at load | not other script test | shown at end pose (finished) | rebuilt at each load |
| 0 | at load | not other script test | breakable disabled | rebuilt at each load |
| 6 | at load | not other script test | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |

**Pack 0989**: Exterior, effect locators, a looping animation thread and a staged sequence

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | table-driven | none | entity removed permanently (kill bit) | kill bit, saved |

**Pack 0990**: Exterior, object setup, trigger registrations, lever-driven bobbing walkway and one control thread

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0-5,7,9,11,table/argument | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| table/argument | load thread | other script test | pose set to a computed time and frozen | no, reverts at reload |
| table/argument | script trigger 3/4/5/6... | other script test | hidden, collision and walls off | no, reverts at reload |
| table/argument | script trigger 3/4/5/6... | other script test | shown at start pose (draw, collision, pose at time 0) | no, reverts at reload |
| 0,table/argument | script trigger 3/4/5/6... | other script test | animation plays once | no, reverts at reload |
| 13,15,17,19,21,23 | at load | none | shown at end pose (finished) | rebuilt at each load |
| table/argument | at load | none | follows a table-chosen locator of object 0 | rebuilt at each load |

**Pack 0991**: Exterior, entity proximity set and villager group controller

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 8 | load thread | flag 86 clear | character set loaded into a slot | rebuilt at each load |
| 3 | load thread | flag 86 set | character set loaded into a slot | rebuilt at each load |
| - | table-driven | other script test | character slot emptied (characters released) | unclear |
| - | table-driven | not other script test | character slot emptied (characters released) | unclear |
| - | thread | none | character slot emptied (characters released) | no, reverts at reload |
| - | thread | other script test | character slot emptied (characters released) | no, reverts at reload |
| table/argument,? | thread | other script test | character set loaded into a slot | no, reverts at reload |
| table/argument | thread | other script test | character activated | no, reverts at reload |
| table/argument | thread | other script test | character released (streaming may rebind) | kill bit, saved |
| - | table-driven | none | character slot emptied (characters released) | unclear |
| ? | table-driven | none | character set loaded into a slot | unclear |

**Pack 1005**: Interior, proximity, on-enter and item conversations

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 86 clear | character set loaded into a slot | rebuilt at each load |
| 1 | load thread | flag 86 set | character set loaded into a slot | rebuilt at each load |

**Pack 1008**: Interior, item-ask lines, on-enter line and an arrival

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 86 clear | character set loaded into a slot | rebuilt at each load |
| 1 | load thread | flag 86 set | character set loaded into a slot | rebuilt at each load |

**Pack 1011**: Interior, item-ask lines, on-enter line and an arrival

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 86 clear | character set loaded into a slot | rebuilt at each load |
| 1 | load thread | flag 86 set | character set loaded into a slot | rebuilt at each load |

**Pack 1014**: Interior, proximity pair, on-enter line and a flag-gated item removal

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 86 clear | character set loaded into a slot | rebuilt at each load |
| 1 | load thread | flag 86 set | character set loaded into a slot | rebuilt at each load |

**Pack 1017**: Exterior, eight entity proximity routines with four dialogue sets, a three-thread setup and teleports

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 86 clear | character set loaded into a slot | rebuilt at each load |
| 1 | load thread | flag 86 set | character set loaded into a slot | rebuilt at each load |

**Pack 1020**: Interior, item-ask lines, on-enter line and an arrival

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 86 clear | character set loaded into a slot | rebuilt at each load |
| 1 | load thread | flag 86 set | character set loaded into a slot | rebuilt at each load |

**Pack 1023**: Exterior, the largest level script in this spec: 46 entity proximity registrations, 31 script-trigger registrations, 11 doors and a tattered-cloak sequence

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 2 | load thread | flag 11 clear and flag 10 clear and (flag 127 clear or flag 136 set) | character set loaded into a slot | rebuilt at each load |
| 0 | load thread | flag 11 clear and flag 10 clear and flag 136 set | character set loaded into a slot | rebuilt at each load |
| 6 | load thread | flag 11 clear and flag 10 clear | character set loaded into a slot | rebuilt at each load |
| - | thread | none | character slot emptied (characters released) | no, reverts at reload |
| - | thread | other script test | character slot emptied (characters released) | no, reverts at reload |
| 0,1,? | thread | other script test | character set loaded into a slot | no, reverts at reload |
| 0,table/argument | thread | other script test | character activated | no, reverts at reload |
| table/argument | thread | other script test | character released (streaming may rebind) | kill bit, saved |
| - | thread | none | character slot emptied (characters released) | flag written, not read at load by this pack |
| ? | thread | none | character set loaded into a slot | flag written, not read at load by this pack |
| - | thread | not other script test and other script test | character slot emptied (characters released) | no, reverts at reload |
| table/argument | thread | not other script test and other script test | character set loaded into a slot | no, reverts at reload |
| 4 | script trigger 17 | none | shown at start pose (draw, collision, pose at time 0) | flag written, not read at load by this pack |
| 4 | script trigger 17 | none | animation plays once | flag written, not read at load by this pack |
| 4 | script trigger 17 | none | pose set to a fixed time and frozen | flag written, not read at load by this pack |
| 4 | script trigger 17 | none | placed at world locator 498 | flag written, not read at load by this pack |
| 4 | script trigger 17 | none | hidden, collision and walls off | flag written, not read at load by this pack |
| - | every tick, load thread | other script test | character slot emptied (characters released) | flag written, not read at load by this pack |
| 5 | every tick, load thread | other script test | character set loaded into a slot | flag written, not read at load by this pack |
| 0 | every tick, load thread | other script test | character activated | flag written, not read at load by this pack |
| - | every tick, load thread | not other script test and other script test | character slot emptied (characters released) | flag written, not read at load by this pack |
| 3-5 | every tick, load thread | not other script test and other script test | character set loaded into a slot | flag written, not read at load by this pack |
| 0 | every tick, load thread | not other script test and other script test | character activated | flag written, not read at load by this pack |
| - | thread | flag 136 clear and other script test | character slot emptied (characters released) | flag written, not read at load by this pack |
| 2 | thread | flag 136 clear and other script test | character set loaded into a slot | flag written, not read at load by this pack |
| - | at load, script trigger 36 | none | ground-block collision layer word := a computed word | flag written, not read at load by this pack |
| - | at load, script trigger 36 | none | placed-object display layer word := 2 x a computed word | flag written, not read at load by this pack |
| 1-3 | at load | flag 64 set and flag 248 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 5 | at load | (flag 150 clear or flag 10 set) and flag 15 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 5 | at load | not ((flag 150 clear or flag 10 set) and flag 15 clear) | shown at end pose (finished) | rebuilt at each load |
| 0 | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 1 | every tick | (flag 64 clear or flag 248 set) and object 1 shown | hidden, collision and walls off | rebuilt each tick |
| 2 | every tick | (flag 64 clear or flag 248 set) and object 2 shown | hidden, collision and walls off | rebuilt each tick |
| 3 | every tick | (flag 64 clear or flag 248 set) and object 3 shown | hidden, collision and walls off | rebuilt each tick |

**Pack 1055**: Transition, world swap with two paths

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |

**Pack 1056**: Exterior, arena combat lessons, per-entity dialogue and teleport callbacks

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 30 clear and flag 191 clear and flag 26 clear and flag 24 clear and flag 23 clear and flag 22 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 0 | load thread | flag 30 clear and flag 191 clear and flag 26 clear and flag 24 clear and flag 23 clear and flag 22 clear | character set loaded into a slot | rebuilt at each load |
| 1,7 | load thread | flag 30 clear and flag 191 clear and flag 26 clear and flag 24 clear and flag 23 clear | character set loaded into a slot | rebuilt at each load |
| 0-7,10 | load thread | flag 30 clear and flag 191 clear and flag 26 clear and flag 24 clear and flag 23 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 2 | load thread | flag 30 clear and flag 191 clear and flag 26 clear and flag 24 clear | character set loaded into a slot | rebuilt at each load |
| 7,8 | load thread | flag 30 clear and flag 191 clear and flag 26 clear and flag 24 clear and flag 174 clear | character set loaded into a slot | rebuilt at each load |
| 12-14 | load thread | flag 30 clear and flag 191 clear and flag 26 clear and flag 24 clear | entity force-spawned (kill bit cleared) | rebuilt at each load |
| 15-19 | load thread | flag 30 clear and flag 191 clear and flag 26 clear and flag 24 clear and other script test | entity force-spawned (kill bit cleared) | rebuilt at each load |
| 0-7 | load thread | flag 30 clear and flag 191 clear and flag 26 clear and flag 24 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 4,7 | load thread | flag 30 clear and flag 191 clear and flag 26 clear | character set loaded into a slot | rebuilt at each load |
| 0-7,10 | load thread | flag 30 clear and flag 191 clear and flag 26 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 7 | load thread | flag 30 clear and flag 29 clear | character set loaded into a slot | rebuilt at each load |
| 0-7,10 | load thread | flag 30 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 6 | load thread | flag 30 set | character set loaded into a slot | rebuilt at each load |
| 22-36 | at load | other script test | breakable disabled | rebuilt at each load |
| 24,27,30,33 | breakable hook 24, breakable hook 27, breakable hook 30, breakable hook 33... | other script test | breakable broken by script | broken bit, saved |
| 36 | breakable hook 24, breakable hook 27, breakable hook 30, breakable hook 33... | other script test and flag 629 clear | breakable broken by script | broken bit, saved |
| - | every tick | none | world item spawned | rebuilt each tick |

**Pack 1066**: Exterior, per-tick world masks with a button pass

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0,3 | load thread | other script test | character set loaded into a slot | no, reverts at reload |
| 0,2,3,5 | load thread | not other script test | character set loaded into a slot | rebuilt at each load |
| table/argument | load thread | none | character set loaded into a slot | no, reverts at reload |
| - | at load, every tick | none | ground-block collision layer word := a computed word | rebuilt at each load |
| - | at load, every tick | none | placed-object display layer word := 2 x a computed word | rebuilt at each load |
| 0 | every tick | flag 2 set and object 0 shown | hidden, collision and walls off | rebuilt each tick |

**Pack 1077** (same code as 1129): Exterior, a flag-38 switch set with three threads and lever triggers

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| table/argument | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| table/argument | load thread | other script test | pose set to a computed time and frozen | no, reverts at reload |
| table/argument | thread | other script test | hidden, collision and walls off | no, reverts at reload |
| table/argument | thread | other script test | shown at start pose (draw, collision, pose at time 0) | no, reverts at reload |
| 24,table/argument | thread, load thread | other script test | animation plays once | no, reverts at reload |
| table/argument | thread | not other script test and other script test | shown at start pose (draw, collision, pose at time 0) | no, reverts at reload |
| table/argument | thread | not other script test and other script test | animation plays once | no, reverts at reload |
| 28,44 | script trigger 21 | flag 307 clear and other script test | animation plays once | flag 36 read at load |
| 27 | script trigger 21 | flag 307 clear and other script test | hidden, collision and walls off | flag 36 read at load |
| 25,34 | script trigger 2 | flag 306 clear and other script test | animation plays once | flag 306 read at load |
| 26 | script trigger 2 | flag 306 clear and other script test | hidden, collision and walls off | flag 306 read at load |
| 24 | at load | flag 38 clear | shown at end pose (finished) | rebuilt at each load |
| 12,46 | at load | flag 38 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 24 | at load | flag 38 set | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 46 | at load | flag 38 set | shown at end pose (finished) | rebuilt at each load |
| 27,28,44 | at load | flag 36 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 28,44 | at load | flag 36 set | shown at end pose (finished) | rebuilt at each load |
| 25,26,34 | at load | flag 306 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 25,34 | at load | flag 306 set | shown at end pose (finished) | rebuilt at each load |
| 32 | at load | none | follows locator 1 of object 24 | rebuilt at each load |
| 33 | at load | none | follows locator 2 of object 24 | rebuilt at each load |

**Pack 1078**: Exterior, object state and button-press room with four trigger scripts registered from Init

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| table/argument | at load, script trigger 1 | other script test | effect stops | flag 309,310 read at load |
| table/argument | at load, script trigger 1 | other script test | effect starts | flag 309,310 read at load |
| table/argument | script trigger 3/4/5/6... | none | hidden, collision and walls off | no, reverts at reload |
| table/argument | script trigger 3/4/5/6... | none | shown at start pose (draw, collision, pose at time 0) | no, reverts at reload |
| table/argument | script trigger 3/4/5/6... | none | animation plays once | no, reverts at reload |
| table/argument | script trigger 3/4/5/6... | not other script test and other script test | effect starts | no, reverts at reload |
| table/argument | script trigger 3/4/5/6.../26/27... | other script test | hidden, collision and walls off | no, reverts at reload |
| table/argument | script trigger 3/4/5/6.../26/27... | other script test | shown at start pose (draw, collision, pose at time 0) | no, reverts at reload |
| table/argument | script trigger 3/4/5/6.../26/27... | other script test | animation plays once | no, reverts at reload |
| table/argument | script trigger 3/4/5/6... | other script test | effect stops | no, reverts at reload |
| 0 | script trigger 1 | flag 310 clear and other script test | hidden, collision and walls off | flag 310 read at load |
| 1 | script trigger 1 | flag 310 clear and other script test | shown at start pose (draw, collision, pose at time 0) | flag 310 read at load |
| 1 | script trigger 1 | flag 310 clear and other script test | animation plays once | flag 310 read at load |
| 1 | script trigger 0 | other script test and flag 310 set and other script test and other script test and item 18 not held | hidden, collision and walls off | flag 311 read at load |
| 0 | script trigger 0 | other script test and flag 310 set and other script test and other script test and item 18 not held | shown at start pose (draw, collision, pose at time 0) | flag 311 read at load |
| 0 | script trigger 0 | other script test and flag 310 set and other script test and other script test and item 18 not held | animation plays once | flag 311 read at load |
| table/argument | at load | none | shown (pose from a variable) | rebuilt at each load |
| table/argument | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| table/argument | at load | other script test | effect stops | rebuilt at each load |
| 1 | at load | flag 310 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 0 | at load | flag 310 set | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |

**Pack 1079**: Transition, two-trigger object sequence with item gate

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| table/argument | at load, script trigger 0/1 | other script test | effect stops | flag written, not read at load by this pack |
| table/argument | at load, script trigger 0/1 | other script test | effect starts | flag written, not read at load by this pack |
| table/argument | script trigger 0/1 | other script test and item 18 not held | hidden, collision and walls off | flag written, not read at load by this pack |
| table/argument | script trigger 0/1 | other script test and item 18 not held | shown at start pose (draw, collision, pose at time 0) | flag written, not read at load by this pack |
| table/argument | script trigger 0/1 | other script test and item 18 not held | animation plays once | flag written, not read at load by this pack |
| 0,1,4,5 | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |

**Pack 1081**: Exterior, effect groups and a flag 41 scripted sequence

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| table/argument | at load, script trigger 2 | other script test | effect stops | flag 41 read at load |
| table/argument | at load, script trigger 2 | other script test | effect starts | flag 41 read at load |
| 3,5,6,8 | script trigger 2 | other script test and flag 41 clear | animation plays once | flag 41 read at load |
| 4,5 | script trigger 2 | other script test and flag 41 clear | shown at start pose (draw, collision, pose at time 0) | flag 41 read at load |
| 0 | script trigger 2 | other script test and flag 41 clear | breakable broken by script | broken bit, saved |
| 3,4,6,7 | script trigger 2 | other script test and flag 41 clear | hidden, collision and walls off | flag 41 read at load |
| 3,6-8 | at load | flag 41 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 5,8 | at load | flag 41 set | shown at end pose (finished) | rebuilt at each load |
| 0 | at load | flag 41 set | breakable disabled | rebuilt at each load |

**Pack 1083**: Exterior, object cascade on flag 620 and a slot-load thread

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 1,4 | load thread | flag 52 clear and flag 20 clear and flag 19 clear | character set loaded into a slot | rebuilt at each load |
| 3 | load thread | flag 52 clear and flag 20 clear | character set loaded into a slot | rebuilt at each load |
| 2 | load thread | flag 52 set | character set loaded into a slot | rebuilt at each load |
| 0 | load thread | none | character set loaded into a slot | no, reverts at reload |
| 1 | script trigger 0 | other script test and flag 620 clear | shown at start pose (draw, collision, pose at time 0) | flag 620 read at load |
| 5-19 | script trigger 0 | other script test and flag 620 clear | animation plays once | flag 620 read at load |
| 0 | script trigger 0 | other script test and flag 620 clear | hidden, collision and walls off | flag 620 read at load |
| table/argument | script trigger 1/2/3 | other script test | character activated | no, reverts at reload |
| table/argument | script trigger 1/2/3 | other script test | animation plays once | no, reverts at reload |
| 0,5-19 | at load | flag 620 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 5-19 | at load | flag 620 set | shown at end pose (finished) | rebuilt at each load |
| 1 | at load | flag 620 set | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 2-4 | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |

**Pack 1089**: Transition, a timed crusher sequence with button stops, a hazard thread and a kill on timeout

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| table/argument | at load, every tick, animated trigger 0 | other script test | effect stops | no, reverts at reload |
| table/argument | at load, every tick, animated trigger 0 | other script test | effect starts | no, reverts at reload |
| table/argument | every tick | none | pose set to a fixed time and frozen | rebuilt each tick |
| table/argument | every tick | object (table) not animating | pose set to a computed time and frozen | rebuilt each tick |
| table/argument | at load | none | follows a table-chosen locator of object 0 | rebuilt at each load |
| table/argument | every tick | none | shown (pose from a variable) | rebuilt each tick |
| table/argument | script trigger 0 | none | animation plays once backward from the end | no, reverts at reload |
| table/argument | animated trigger 0 | other script test | pose set to a fixed time and frozen | no, reverts at reload |
| 0,7-11,table/argument | animated trigger 0, script trigger 0 | other script test | animation plays once | no, reverts at reload |
| 0,7-11 | every tick | none | shown at start pose (draw, collision, pose at time 0) | rebuilt each tick |
| 0 | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 0,8-11 | every tick | other script test | play speed set from a module value | rebuilt each tick |
| 0,8-11 | every tick | other script test | animation frozen at its current time | rebuilt each tick |

**Pack 1090**: Exterior, hazard timer step, firing loops and hurt zones

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| table/argument | every tick | other script test | animation plays once | rebuilt each tick |
| table/argument | every tick | other script test | hidden, collision and walls off | rebuilt each tick |

**Pack 1091**: Exterior, object setups, a button-prompt trigger, hazard triggers and character navmesh assignments

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0-17, 41, 71-133, 137-140, 135, 51-54, 136 | at load (Init and its set-up helper) | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 135, 51, 52, 53, 136, 54 | at load | none | follows locator 0 of pillars 75, 100, 111, 116, 130, 134 (follow true) | rebuilt at each load |
| 71-134 (pillar i = object 71 + i) | load thread (start-up snap) | none (stage 4) | pose set to the stage-4 layout time (40/30 s each) and frozen; this also first shows pillar 134, which the set-up loop skips | rebuilt at each load |
| 71-134 | every tick | busy word 1 (after a button press) | pose set to a computed time stepping toward the current stage's layout value (0.01, 0.005 or 0.0025 s per tick; snap within 0.0025 s) | rebuilt each tick |
| 135, 51, 52, 53, 136, 54 | every tick | busy word 1 | pose set to time 0 and frozen (rides its pillar) | rebuilt each tick |
| 138, 139, 140, 137 | load thread | none (available in stage 4) | shown at end pose (finished) | rebuilt at each load |
| 135, 51, 52, 53, 136, 54 | load thread | none (not available in stage 4) | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 135, 51, 52, 53, 136, 54, 138, 139, 140, 137 | press: six animated triggers, script triggers 67-70 | hero inside, busy 0, button at its end time and not animating, row's stage differs from the current stage | every button not at time 0 shown at end pose, then animation plays once backward from the end (no wait) | no, reverts at reload |
| the same ten | every tick | busy word 1 ended (no pillar moved); button not animating | row flag for the stage 1: at time 0 animation plays once, otherwise shown at end pose; flag 0: at end time animation plays once backward, otherwise shown at start pose | no, reverts at reload |
| 1-6 | press, after a 30-tick wait | none | hidden, collision and walls off (platform piece 1, draw-only 2, hazard bodies 3-6; a later play call revives a hazard body) | no, reverts at reload |
| aim targets 1-6 | press and load thread | stage: on only 1, 3, 6 in stage 1; 2, 5 in stage 0; 4 in stage 2 | aim target active := true / false | no, reverts at reload |
| script groups 3-10 | press and load thread | stage: 3, 4 in stage 0; 5-8 in stage 1; 9, 10 in stage 2 | all released, then the stage's groups restored | no, reverts at reload |
| slot A | press; busy ended; load thread | none | character slot emptied (press); then character set 0-3 loaded for stages 0-3, set 3 for stage 4 | no, reverts at reload |
| 3 | script trigger 41 (mode-2 sequencer); animated trigger 3 | trigger 41 started and the mode word 0 | animation plays once at 0.6 (wait), 1.0 s later backward at -0.6 (wait), repeating; a forward play sets the body's hazard flag | no, reverts at reload |
| 6, 10, 5, 4, 9, 8, 7 | script trigger 42 (mode-1 sequencer); animated triggers 4-10 | trigger 42 started and the mode word 1 | animations play forward at 1.6 and backward at -0.4 in the repeating order given in the moving-hazard bullet (SPEC-0044); a forward play sets the hazard flag | no, reverts at reload |
| 2 (trigger 41), 17 (42), 16 down to 11 (43-48) | script triggers 41-48 | hero enters (state 1) | pose set to k/30 s for k = 0 to 10 (one per tick); after the hero leaves, k = 13 to 25 (clamped at the 0.8 s end) | no, reverts at reload |
| 36-40, 33-35, 28-32, 23-27, 21-22, 18-20 | script triggers 43, 44, 45, 46, 47, 48 | hero enters (state 1) | animation plays once at 5.0 (no wait), 0.02 s apart; objects start hidden, stay at the end pose; a second entry plays nothing | no, reverts at reload |
| 41, 0, 1 | script trigger 51 | button pressed, object 41 not finished | 41 plays once (no wait); 0 plays once (no wait); 1 plays once (wait) | no, reverts at reload |

**Pack 1096**: Exterior, level state machine with hero pickup, timed boss sequence and debug shortcuts

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 6,10-13,16,19,22,table/argument | script trigger 5/0/1/2/3.../13... | other script test | animation plays once | no, reverts at reload |
| 6,8,10,16 | script trigger 0/1/2/3.../13/14... | other script test | hidden, collision and walls off | no, reverts at reload |
| table/argument | script trigger 0/1/2/3... | none | animation plays once | no, reverts at reload |
| 1 | every tick | none | animation plays once | rebuilt each tick |
| 1 | every tick | none | aim target active := true | rebuilt each tick |
| 0 | load thread | none | entity removed permanently (kill bit) | kill bit, saved |
| 0 | load thread | none | hidden, collision and walls off | flag 45 read at load |
| 2-5 | load thread | none | animation plays once | flag 45 read at load |
| - | load thread | other script test | world item spawned | flag 45 read at load |
| 0 | world-activate callback | none | character set loaded into a slot | re-run when the world becomes current |
| 0-5,14 | world-activate callback | none | shown at start pose (draw, collision, pose at time 0) | re-run when the world becomes current |
| 1 | world-activate callback | none | aim target active := false | re-run when the world becomes current |
| 0 | world-activate callback | none | entity force-spawned (kill bit cleared) | re-run when the world becomes current |
| 6,8,10,12,13,16,17,19,22,31,33,35 | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |

**Pack 1098**: Exterior, flag-gated proximity dialogue set, hero-death callback and a large entity and slot routine set

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 10,11 | script trigger 9 | other script test | animation plays once | flag 622 read at load |
| 14 | script trigger 9 | other script test | hidden, collision and walls off | flag 622 read at load |
| - | script trigger 4 | flag 204 clear and flag 203 set | character slot emptied (characters released) | flag written, not read at load by this pack |
| 6 | script trigger 4 | flag 204 clear and flag 203 set | character set loaded into a slot | flag written, not read at load by this pack |
| - | script trigger 0 | none | character slot emptied (characters released) | flag 22,27,29,642 read at load |
| 4,5 | script trigger 0 | none | character set loaded into a slot | flag 22,27,29,642 read at load |
| - | every tick, entity proximity 35, entity proximity 36, script trigger 10 | none | character slot emptied (characters released) | flag 151,502 read at load |
| 21 | load thread | flag 31 clear and flag 30 clear and flag 501 set and item 72 not held and flag 504 clear and flag 503 clear | character set loaded into a slot | rebuilt at each load |
| 0,9 | load thread | flag 31 clear and flag 30 clear | character set loaded into a slot | rebuilt at each load |
| 1 | load thread | flag 31 set | character set loaded into a slot | rebuilt at each load |
| - | thread | other script test | character slot emptied (characters released) | no, reverts at reload |
| 19,table/argument | thread | other script test | character set loaded into a slot | no, reverts at reload |
| 9,10 | script trigger 17/18/19/20 | not other script test | effect stops | flag written, not read at load by this pack |
| table/argument | script trigger 17/18/19/20 | none | animation plays once | flag written, not read at load by this pack |
| table/argument | script trigger 17/18/19/20 | none | hidden, collision and walls off | flag written, not read at load by this pack |
| table/argument | script trigger 17/18/19/20 | none | shown at start pose (draw, collision, pose at time 0) | flag written, not read at load by this pack |
| 7 | script trigger 17/18/19/20 | not other script test | animation plays once | flag written, not read at load by this pack |
| 8 | table-driven | other script test | animation plays once | unclear |
| 1 | at load | flag 30 clear and flag 31 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 10,11 | at load | flag 622 set | shown at end pose (finished) | rebuilt at each load |
| 10,11,14 | at load | flag 622 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 0,2,3,7,8,15,17,19,21 | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 4 | at load | other script test | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 9,10 | at load | flag 30 clear or flag 91 set | effect stops | rebuilt at each load |
| 0-3 | at load | none | effect stops | rebuilt at each load |
| 0-2 | every tick | flag 642 clear | effect starts | rebuilt each tick |
| 0-2 | every tick | flag 642 set | effect stops | rebuilt each tick |
| 3 | every tick | flag 642 set and flag 632 clear | effect starts | rebuilt each tick |
| 3 | every tick | flag 642 set and flag 632 clear | effect stops | rebuilt each tick |

**Pack 1122**: Exterior, per-tick navmesh assignments, a wave sequence and the Faerie Sisters arena

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 2 | load thread | flag 31 clear and flag 30 clear | character set loaded into a slot | rebuilt at each load |
| table/argument | load thread | none | character set loaded into a slot | no, reverts at reload |
| 4-18 | load thread | none | entity removed permanently (kill bit) | kill bit, saved |
| 3 | load thread | none | shown at start pose (draw, collision, pose at time 0) | flag written, not read at load by this pack |
| 0-2,4,5 | script trigger 5 | other script test and flag 320 clear and other script test | hidden, collision and walls off | flag 320 read at load |
| 0-2,4,5 | script trigger 5 | other script test and flag 320 clear and other script test | shown at start pose (draw, collision, pose at time 0) | flag 320 read at load |
| 1,2,4,5 | script trigger 5 | other script test and flag 320 clear and other script test | animation plays once | flag 320 read at load |
| 3-5 | script trigger 5 | other script test and flag 320 clear and other script test | character set loaded into a slot | flag 320 read at load |
| - | script trigger 5 | other script test and flag 320 clear and other script test | character slot emptied (characters released) | flag 320 read at load |
| 2,4 | at load | other script test | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 4 | at load | other script test | shown at end pose (finished) | rebuilt at each load |
| 1,4 | at load | not other script test | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |

**Pack 1130**: Transition, three button zones, a one-shot switch and a movie zone

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0,8,9,15-17,29,31,33 | script trigger 4/5/6/7 | other script test | hidden, collision and walls off | flag 675 read at load |
| 7,10,12-14,18-22 | script trigger 4/5/6/7 | not other script test and other script test | hidden, collision and walls off | flag 675 read at load |
| 7-10,12-22,27,28 | script trigger 4 | none | hidden, collision and walls off | flag 675 read at load |
| 7,9-12,15,18,30,32,34 | script trigger 4 | other script test | shown at start pose (draw, collision, pose at time 0) | flag 675 read at load |
| 14,17,22 | script trigger 4 | not other script test | shown at start pose (draw, collision, pose at time 0) | flag 675 read at load |
| 14,17,22 | script trigger 4 | not other script test | animation plays once | flag 675 read at load |
| 18 | script trigger 4 | not other script test and other script test | shown at start pose (draw, collision, pose at time 0) | flag 675 read at load |
| 8,19,27,28 | script trigger 4 | none | shown at start pose (draw, collision, pose at time 0) | flag 675 read at load |
| 8,19,27,28 | script trigger 4 | none | animation plays once | flag 675 read at load |
| 1,7,9-11,30,32,34,table/argument | script trigger 4 | other script test | animation plays once | flag 675 read at load |
| 1 | script trigger 2 | other script test and object 2 hidden and flag 675 clear | hidden, collision and walls off | flag 675 read at load |
| 0,2 | script trigger 2 | other script test and object 2 hidden and flag 675 clear | shown at start pose (draw, collision, pose at time 0) | flag 675 read at load |
| 2 | script trigger 2 | other script test and object 2 hidden and flag 675 clear | animation plays once | flag 675 read at load |
| 29-34 | script trigger 5/6/7 | other script test and flag 675 clear and other script test | hidden, collision and walls off | no, reverts at reload |
| 12-21,29-34 | script trigger 5/6/7 | other script test and flag 675 clear and other script test | shown at start pose (draw, collision, pose at time 0) | no, reverts at reload |
| 12-21,29-34 | script trigger 5/6/7 | other script test and flag 675 clear and other script test | animation plays once | no, reverts at reload |
| 31,33 | table-driven | other script test | animation plays once | unclear |
| 0,1,11,12,15,18,27,29,31,33 | at load | flag 675 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 1,27,29,31,33 | at load | flag 675 set | shown at end pose (finished) | rebuilt at each load |
| 11,14,17,19 | at load | flag 675 set | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |

**Pack 1131**: Interior, dialogue with an item check, a slot thread and four ask lines

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 203 set | character set loaded into a slot | rebuilt at each load |

**Pack 1133**: Interior, a multi-line dialogue with item checks and a slot thread

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 10 clear | character set loaded into a slot | rebuilt at each load |
| 1 | load thread | flag 10 set | character set loaded into a slot | rebuilt at each load |

**Pack 1136**: Interior, dialogue with a flag-491 to flag-492 cascade and a slot thread

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 10 clear | character set loaded into a slot | rebuilt at each load |
| 1 | load thread | flag 10 set | character set loaded into a slot | rebuilt at each load |

**Pack 1139**: Exterior, a teleport with a blackboard value and an entity dialogue

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | at load | flag 35 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 0 | at load | flag 35 set | shown at end pose (finished) | rebuilt at each load |

**Pack 1140**: Interior, dialogue with a buy callback and a sequence of item checks

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 10 clear | character set loaded into a slot | rebuilt at each load |
| 1 | load thread | flag 10 set | character set loaded into a slot | rebuilt at each load |

**Pack 1143**: Interior, dialogue with item trades and a slot chooser

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 10 clear | character set loaded into a slot | rebuilt at each load |
| 1 | load thread | flag 10 set | character set loaded into a slot | rebuilt at each load |

**Pack 1148**: Interior, dialogue with item trades, a camera and a walk-to-entity pattern

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 496 set | character set loaded into a slot | rebuilt at each load |

**Pack 1150**: Interior, per-tick wandering for a table of entities, with idle loops, a slot thread and five proximity callbacks set at init

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 142 clear | character set loaded into a slot | rebuilt at each load |

**Pack 1152**: Exterior, a proximity ladder, two teleports, a world-activate branch and a slot choice

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 31 clear and flag 30 clear | character set loaded into a slot | rebuilt at each load |
| table/argument | load thread | none | character set loaded into a slot | no, reverts at reload |

**Pack 1156**: Interior, library teleport and a slot-set chooser

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 31 clear and flag 30 set | character set loaded into a slot | rebuilt at each load |
| 1 | load thread | flag 31 set | character set loaded into a slot | rebuilt at each load |

**Pack 1159**: Interior, teleport, slot chooser and a looping entity animation

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 31 clear and flag 30 clear | character set loaded into a slot | rebuilt at each load |
| table/argument | load thread | none | character set loaded into a slot | no, reverts at reload |

**Pack 1163**: Exterior, the Miamice catch game, a teleport with a prompt, a button prompt and a multi-stage entity and camera sequence

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| - | entity proximity 7 | none | character slot emptied (characters released) | flag written, not read at load by this pack |
| 3,4 | entity proximity 7 | none | character set loaded into a slot | flag written, not read at load by this pack |
| 0 | load thread | flag 31 clear and flag 11 clear and flag 30 clear | character set loaded into a slot | rebuilt at each load |
| table/argument | load thread | none | character set loaded into a slot | no, reverts at reload |
| 5 | load thread | flag 29 clear | character set loaded into a slot | rebuilt at each load |
| 0 | at load | flag 155 clear or other script test | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |

**Pack 1171**: Exterior, lever cutscenes, breakable flags and script group type 12

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 12,15,16,table/argument | script-group callback, script trigger 7/5/29/30/31/32 | other script test | shown at start pose (draw, collision, pose at time 0) | no, reverts at reload |
| 1,2,11,15,table/argument | script-group callback, script trigger 7/5/29/30/31/32 | other script test | animation plays once | no, reverts at reload |
| 11,15,table/argument | script-group callback, script trigger 7/5/29/30/31/32 | other script test | hidden, collision and walls off | no, reverts at reload |
| table/argument | script-group callback | none | shown at start pose (draw, collision, pose at time 0) | no, reverts at reload |
| table/argument | script-group callback | none | pose set to a computed time and frozen | no, reverts at reload |
| table/argument | script-group callback | none | animation loops | no, reverts at reload |
| table/argument | script-group callback | other script test | pose set to a fixed time and frozen | no, reverts at reload |
| table/argument | script-group callback | other script test | animation loops | no, reverts at reload |
| 0-2,11,13,15,17,20,23,26 | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| table/argument | at load | flag (table) set | breakable disabled | rebuilt at each load |
| 17 | every tick | other script test and object 17 shown | animation loops | rebuilt each tick |
| 20 | every tick | other script test and object 20 shown | animation loops | rebuilt each tick |
| 23 | every tick | other script test and object 23 shown | animation loops | rebuilt each tick |
| 26 | every tick | other script test and object 26 shown | animation loops | rebuilt each tick |

**Pack 1173**: Exterior, teleport and arrival with a button-gated trigger

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0,table/argument | script trigger 1 | object (table) shown and other script test and flag 167 clear | animation plays once | flag 630 read at load |
| table/argument | script trigger 1 | object (table) shown and other script test and flag 167 clear | hidden, collision and walls off | flag 630 read at load |
| table/argument | script trigger 1 | object (table) shown and other script test and flag 167 clear | shown at start pose (draw, collision, pose at time 0) | flag 630 read at load |
| 1 | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 0 | at load | flag 630 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 0 | at load | flag 630 set | shown at end pose (finished) | rebuilt at each load |

**Pack 1174**: Exterior, the Act 1 escape run and the Act 2 start: four arena sections of collapsing pillars, checkpoints, a stage thread and the act-start trigger

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| breakables 3-10 | world-activate callback | none | breakable enabled (intact) | re-run when the world becomes current |
| breakables 3-10 | script trigger 22-29 (one each, in order; fired with object word 0 by the Sisters' ball) | breakable still intact | breakable broken by script; sound 9 at locator 65-72; the trigger is cleared from the act-1 trigger data | no, reverts at reload (the broken bit is saved for script indexes below 128, but the next arrival enables the breakable again) |
| 37,49,61,73,85,97,109,120,133,145,157,168,180,192,204,216,228,240 (pillar solids) | world-activate callback | none | shown at start pose (draw, four wall records) | re-run when the world becomes current |
| every piece, stump and fallen object of the 18 tiles (ids 26-36, 38-48, 50-60 and so on up to 229-239, plus 110-119, 121-132 and 158-167; the pillar tile table in SPEC-0044 lists them) | world-activate callback | none | hidden, collision and walls off | re-run when the world becomes current |
| solid of tile 36-41, 44, 45, 47-53 | script trigger 36-41, 44, 45, 47-53 (object thread) | tile solid, fired with object word 0 (Sisters' ball) | hidden, collision and walls off | no, reverts at reload |
| pieces of tile 36-41, 44, 45, 47-53 (11 objects, first one a static stump) | same, same pass | same | shown, animation plays once at speed 1.0; the stump brings four wall records | no, reverts at reload |
| fade pieces of those tiles (the 10 pieces after the stump) | same tile, 0.9 s (sections 0-1) or 1.0 s (sections 2-3) after the hit, for 0.5 s | none | colour multiplier set (128, 128, 128, alpha falling 255 to 0) | no, reverts at reload |
| fade pieces of those tiles | same tile, first pass after the fade end with no piece animating | none | hidden; only the stump stays shown (tile 53: the stump is object 26) | no, reverts at reload |
| solid 109, 120, 157 | script trigger 42, 43, 46; tile 42 also by tile 43's link 1.7 s after tile 43's stamp | tile solid, fired with object word 0 | hidden, collision and walls off | no, reverts at reload |
| pieces 110-118, 121-131, 158-166 | same, same pass | same | shown, animation plays once at speed 1.2 (the first piece is the stump with wall records) | no, reverts at reload |
| fade pieces 111-116, 122-130, 159-165 | same tile, 1.1 s (tiles 42, 43) or 1.0 s (tile 46) after the hit, for 0.5 s | none | colour multiplier set (128, 128, 128, alpha falling 255 to 0) | no, reverts at reload |
| fallen 119, 132, 167 | same tile, first pass after the fade end with no piece animating (tile 42 4.0 s, tile 43 3.17 s, tile 46 1.5 s) | none | all pieces including the stump hidden; fallen object shown at start pose with its ramp or column and wall records | no, reverts at reload |
| effects 92-109 (one per tile, tile order) | script trigger 36-53 (object thread) | tile fired | effect starts (camera as listener) | no, reverts at reload |
| 6 | object thread | tile 42 left its solid state and 0.85 s have passed | hidden (rail A whole rail) | no, reverts at reload |
| 9-12 | same pass | same | shown, animation plays once at speed 1.0 (rail A falling pieces); hidden again when none still animates (about 49 ticks) | no, reverts at reload |
| 6 | world-activate callback | none | shown at start pose (rail A) | re-run when the world becomes current |
| 8 | world-activate callback | none | shown at start pose (rail B) | re-run when the world becomes current |
| 8 | stage thread, camera 6 scene | tile 43 reached its rest state; two stage passes later | hidden (rail B) | no, reverts at reload |
| 13,14 | object thread, first pass after the arrival | none | shown at time zero (barrier leaves) | re-run when the world becomes current |
| 15,16 | object thread, first pass after the arrival | none | hidden | re-run when the world becomes current |
| 13,14 | script trigger 5, hit 1 | barrier idle | shown at time zero, animation plays once (wobble 0.733 s) | no, reverts at reload |
| 13,14 | script trigger 5, hit 2 | barrier idle after hit 1 | hidden | no, reverts at reload |
| 15,16 | script trigger 5, hit 2 | same | shown at time zero, animation plays once (2.133 s and 1.2 s) | no, reverts at reload |
| 15,16 | object thread, 1.5 s to 2.5 s after hit 2 | none | colour multiplier set (fade to clear) | no, reverts at reload |
| 13-16 | object thread, first pass after 2.5 s with no piece animating | none | hidden; collision value 1 stays clear | no, reverts at reload |
| 17-19 and 7 | world-activate callback | none | shown at time zero (17-19 break scene pieces; 7 collision wall box) | re-run when the world becomes current |
| 17-19 | object thread | tile 53 out of its solid state (once) | animation plays once (0.933 s, 0.933 s, 0.8 s); stay shown at end pose | no, reverts at reload |
| 7 | object thread | same | hidden, collision and walls off | no, reverts at reload |
| effects 15-21 | world-activate callback | none | effect stops | re-run when the world becomes current |
| effects 15-21 | object thread | tile 53 out of its solid state (once) | effect starts | no, reverts at reload |
| 22,20 and 24,21 (gate leaf and helper) | world-activate callback; object thread, request 0 | section 2 scene requests rest | shown at time zero (rest look); 23, 25 hidden | re-run when the world becomes current |
| 22,20 and 24,21 | object thread, gate request 1 (section 3 scene) | none | shown at time zero, animation plays once; at the end 22 and 24 hidden, 23 and 25 shown at time zero, helpers shown at end pose | no, reverts at reload |
| 5 and 4 (collision boxes) with effects 50-53 and 35-38 | stage thread (sections 1, 2, 3) | barrier group raised or lowered | shown or hidden, collision and walls on or off, with their effects started or stopped | no, reverts at reload |
| effects 22-23, 29-30, 32-34, 39-40, 42-49 | stage thread (sections 1, 2, 3) | barrier group raised or lowered | effects start or stop | no, reverts at reload |
| effects 9-14 | at load | none | effect stops | rebuilt at each load |
| effects 9-14 | stage thread, section 3 scene | none | effects start | no, reverts at reload |
| - | object thread | tile 43 in state 1, once | world collision word := word OR 4 | level state only, not saved |
| - | object thread | barrier first pass; hit 2; gate pairs closed or reopened | world collision word value 1 set, then cleared; value 2 set when a gate pair closes and cleared when it reopens | level state only, not saved |
| 0 | load thread | none | character set loaded into a slot | no, reverts at reload |
| 0-3 | load thread | none | character activated | no, reverts at reload |

**Pack 1176**: Exterior, hero death watch and scripted encounter threads

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| table/argument | load thread | none | entity removed permanently (kill bit) | kill bit, saved |
| table/argument | load thread | none | character set loaded into a slot | no, reverts at reload |

**Pack 1184**: Exterior, world-entry triggers with thread-driven slot loads and an activate-world camera

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| - | thread | none | character slot emptied (characters released) | no, reverts at reload |
| table/argument | thread | flag 512 clear or flag 513 set | character set loaded into a slot | no, reverts at reload |
| table/argument | thread | not (flag 512 clear or flag 513 set) | character set loaded into a slot | no, reverts at reload |
| 7 | load thread | flag 84 clear and flag 82 clear and (flag 512 clear or flag 513 set) | character set loaded into a slot | rebuilt at each load |
| 8 | load thread | flag 84 clear and flag 82 clear and flag 513 set | character set loaded into a slot | rebuilt at each load |
| 15 | load thread | flag 84 clear and (flag 512 clear or flag 513 set) | character set loaded into a slot | rebuilt at each load |
| 16 | load thread | flag 84 clear and flag 513 set | character set loaded into a slot | rebuilt at each load |
| 30 | load thread | flag 84 set | character set loaded into a slot | rebuilt at each load |
| - | every tick | flag 277 clear | placed-object display layer word := 2 x 1 | rebuilt each tick |
| - | every tick | flag 277 clear | ground-block collision layer word := 1 | rebuilt each tick |
| - | every tick | flag 277 set | placed-object display layer word := 2 x 2 | rebuilt each tick |
| - | every tick | flag 277 set | ground-block collision layer word := 2 | rebuilt each tick |

**Pack 1217**: Exterior, effect and object toggles, activate-world and player-activate callbacks, and scripted cutscenes

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| effects 10-23 | at load (record fill); world-activate callback | none | effect stops | rebuilt at each load; re-run when the world becomes current |
| A groups 13+27r to 21+27r (r = 0-10) | record thread, outer state 1 entry | request 1 from intro A's stagger (10-20 ticks apart; Act 1 only, none after the skip intro) | shown at start pose, animation plays once forward (10.0 s); effect 10+r starts | no, reverts at reload |
| A groups | record thread, outer state 1 later pass | no A object animating | hidden | no, reverts at reload |
| B groups 22+27r to 30+27r | record thread, outer state 1 end | same | shown (idle loop: each member set to frame 0 and replayed whenever all have stopped); effect 10+r stops | no, reverts at reload |
| B groups | record thread, outer state 2 (Act 3: request 2 from intro B; Act 1: after the rise) | none | idle loop as above | no, reverts at reload |
| A and B groups, C groups 31+27r to 39+27r | record thread, outer state 3 entry | request 3: the hero hit the record's entity while the record is in state 2 | A, B hidden; C shown, animation plays once (1.0 s); effect 10+r starts | no, reverts at reload |
| B groups, C groups | record thread, outer state 3 | no C object animating | B shown, A and C hidden; effect stops | no, reverts at reload |
| A, B, C groups | record thread, outer state 4 (close) | request 4 from the outros or from the stage-end flags | A shown at end pose, animation plays backward; B and C hidden; then all hidden | no, reverts at reload |
| P 364+4r | inner state 1 entry | inner request 1 (with the rise, or the set-back flags 15-17) | shown, animation plays once (1.0 s); Q, R, S hidden; R shown when it ends | no, reverts at reload |
| Q 365+4r | inner state 3 entry | inner request 3 with the hit | shown, animation plays once (1.0 s), effect 23 moved to locator 55+3r and started; R shown when it ends | no, reverts at reload |
| R 366+4r, 367+4r | inner state 4 entry | inner request 4: death of the record's entity, or the close | shown alone, animation plays once (0.53 s); 0.4 s later sound 10 and effect 21 at locator 56+3r | no, reverts at reload |
| T 2+r (collision object) | inner state 4, when both R objects have stopped and the 0.4 s step has happened | none | shown (R stays shown) | no, reverts at reload |
| S 408+13r to 420+13r | inner state 7 entry | inner request 7: a stage-locator flag of the boss (slam) | shown, animation plays once (1.0 s), sound 11 at locator 57+3r, effect 22; P, Q, R, T hidden; all inner objects hidden when it ends | no, reverts at reload |
| inner objects of all eleven records | record thread, request 2 (intro B) | Act 3 | hidden (the records start in the idle loop) | no, reverts at reload |
| gate groups 611-637 (A0), 638-664 (B0), 665-691 (A1), 692-718 (B1) and singles 0, 1 | gate stepper, request 1 (intro A and the skip intro) | gate state 1 | B shown at end pose and played backward (4.0 s), A hidden; single shown at start pose at the end of the entry pass; when B stops: B hidden, A and single shown, then A replayed from frame 0 whenever all stop | no, reverts at reload |
| gate groups and singles | gate stepper, request 2 (intro B on both gates; the Act 3 branch where the fight is not due, on gate 0) | gate state 2 | A and single shown at start pose, B hidden; A looped | no, reverts at reload |
| gate groups and singles of gate 1 | gate stepper, request 4 (outros) | gate state 4 | B shown at start pose and played forward (4.0 s), A and single hidden; then everything hidden | no, reverts at reload |
| cage groups 597-603 (A) and 604-610 (B); extra 554 | thread routine 12 and intro A / skip intro: request 2; outro A: request 4 | Act 1 | A replayed at speed 2.5 while closed; request 4: B shown and played forward, A hidden, extra object played once (5.0 s), entity 3 animation 0, then both groups hidden | no, reverts at reload |
| cage groups 555-596 (A and B of three cages); extras 553, 552, 551 | thread routine 12 and intro B: request 2; outro B: request 4 in turn | Act 3 | as the Act 1 cage, for entities 4, 5, 6 | no, reverts at reload |
| entities 0-2 (Tentacles) | fight controller (stage spawns 1-3, and the set-back reruns) | stage start | entity force-spawned (kill bit cleared), moved to locators 13-18, aggressive | flag 82,278 read at load |
| 0 | load thread | flag 83 clear | character set loaded into a slot | rebuilt at each load |
| 1 | load thread | flag 83 set | character set loaded into a slot | rebuilt at each load |
| 0 | load thread | none | character activated | flag 82,278 read at load |
| 3-8 | load thread | none | entity removed permanently (kill bit) | kill bit, saved |
| effects 10-23 | world-activate callback | none | effect stops | re-run when the world becomes current |

**Pack 1230**: Exterior, a slot chooser, a treasure-map exchange and button-prompt triggers

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | at load | flag 64 set and flag 248 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| - | at load | flag 248 set | ground-block collision layer word := 1 | rebuilt at each load |
| - | at load | flag 248 set | placed-object display layer word := 2 x 1 | rebuilt at each load |
| 0 | every tick | (flag 64 clear or flag 248 set) and object 0 shown | hidden, collision and walls off | rebuilt each tick |

**Pack 1238**: Exterior, a pickup watch, slot sets by flag and two threads

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 50 clear and flag 49 clear | character set loaded into a slot | rebuilt at each load |
| 3 | load thread | flag 50 set | character set loaded into a slot | rebuilt at each load |
| 2 | load thread | flag 50 set and (flag 567 clear or flag 568 set) | character set loaded into a slot | rebuilt at each load |
| 1 | load thread | flag 50 set and flag 568 set | character set loaded into a slot | rebuilt at each load |
| 4 | load thread | flag 578 set and flag 579 clear | character set loaded into a slot | rebuilt at each load |

**Pack 1244**: Exterior, a hero swap with dialogue, an entity talk and a break sequence

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 5,6 | script trigger 3 | other script test and flag 272 clear | animation plays once | flag written, not read at load by this pack |
| 17-24 | script trigger 3 | other script test and flag 272 clear | breakable broken by script | broken bit, saved |
| 1,4,5 | script trigger 3 | other script test and flag 272 clear | hidden, collision and walls off | flag written, not read at load by this pack |
| 6 | script trigger 3 | other script test and flag 272 clear | shown at start pose (draw, collision, pose at time 0) | flag written, not read at load by this pack |
| 0,1,4,5 | at load | flag 273 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |

**Pack 1246**: Exterior, the Werelupe King arena: an entry walk, a level-state fight cascade and the reward

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 16-24 | script trigger 0 | level state 8 set | entity removed permanently (kill bit) | kill bit, saved |
| 0,1,4 | script trigger 0 | flag 273 clear and level state 7 clear and level state 5 clear and level state 3 clear | shown at start pose (draw, collision, pose at time 0) | flag 273 read at load |
| 5 | script trigger 0 | flag 273 clear and level state 7 clear and level state 5 clear and level state 3 clear | hidden, collision and walls off | flag 273 read at load |
| 21 | script trigger 0 | flag 273 clear and level state 7 clear and level state 5 clear and level state 3 clear | entity removed permanently (kill bit) | kill bit, saved |
| 21 | script trigger 0 | flag 273 clear and level state 7 clear and level state 5 clear and level state 3 clear | entity force-spawned (kill bit cleared) | flag 273 read at load |
| 16-24 | script trigger 0 | flag 273 set | entity removed permanently (kill bit) | kill bit, saved |
| 0,1,3 | script trigger 0 | flag 273 set | hidden, collision and walls off | flag 273 read at load |
| 2 | script trigger 0 | flag 273 set | shown at start pose (draw, collision, pose at time 0) | flag 273 read at load |
| 3,5 | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |

**Pack 1247**: Exterior, trigger set with object swaps, an item-and-AI check and two world callbacks

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 2, 3, 8, 10, 16, 18, 21, 24, 26, 29, 32, 34 | at load | flag 78 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 5, 7, 12, 15, 23, 31 | at load | flag 78 clear | shown at end pose (finished); triggers registered | rebuilt at each load |
| 0, 1, 4, 7, 9, 11, 12, 15, 17, 19, 21, 23, 25, 27, 28, 31, 33, 35 | at load | flag 78 set | shown at start pose (draw, collision, pose at time 0); no trigger registered, nothing can move | rebuilt at each load |
| effects 0-23 | at load | none | effect stops | rebuilt at each load |
| 0, 1 | script trigger 0 | level state 0 clear, object word non-zero, state 1 | hidden, collision and walls off | no, reverts at reload |
| 3, 2 | script trigger 0 | same | shown at start pose (draw, collision, pose at time 0) | no, reverts at reload |
| 2, then 3 | script trigger 0 | same | animation plays once, one after the other (waited); effects 20-23 start then stop | no, reverts at reload |
| 9, 11, 7 | script trigger 1 | state 1 at the start | hidden, collision and walls off | no, reverts at reload |
| 8, 10, 6 | script trigger 1 | state 1 at the start | shown at start pose (draw, collision, pose at time 0) | no, reverts at reload |
| 6, then 8, then 10 | script trigger 1 | state 1 at the start; 8 and 10 only if the plate is still held at the re-read | animation plays once, each waited; effects 13, 14, 15 start, 12-15 stop | no, reverts at reload |
| 6 / 7 | script trigger 1 | released at the re-read | 6 hidden, 7 shown at start pose and plays once (no wait); end | no, reverts at reload |
| 8, 10, 6 | script trigger 1 | state back to 0 | hidden, collision and walls off | no, reverts at reload |
| 9, 11, 7 | script trigger 1 | state back to 0 | shown at start pose, then animation plays once (no wait) | no, reverts at reload |
| 25, 27, 28, 23 | script trigger 2 | state 1 at the start | hidden, collision and walls off | no, reverts at reload |
| 24, 26, 29, 22 | script trigger 2 | state 1 at the start | shown at start pose (draw, collision, pose at time 0) | no, reverts at reload |
| 22, then 24, 26, 29 | script trigger 2 | state 1; 24, 26, 29 only if still held at the re-read | animation plays once, each waited (4.33 s, 1.0 s, 1.53 s); effects 16-19 start then stop | no, reverts at reload |
| 24, 26, 29, 22 hidden; 25, 27, 28, 23 shown and play | script trigger 2 | state 0 or released | swap back; plays once (no wait) | no, reverts at reload |
| 33, 35, 31 hidden; 32, 34, 30 shown | script trigger 3 | state 1 at the start | hidden / shown at start pose | no, reverts at reload |
| 30, then 34 (no wait), 32 (wait) | script trigger 3 | state 1; 34 and 32 only if still held at the re-read | animation plays once (4.67 s, 7.33 s); effects 0-3 start then stop | no, reverts at reload |
| 32, 34, 30 hidden; 33, 35, 31 shown and play | script trigger 3 | state 0 or released | swap back; plays once (no wait) | no, reverts at reload |
| 17, 19, 12 hidden; 16, 18, 13 shown | script trigger 5 (trigger 6: 17, 19, 15 hidden; 16, 18, 14 shown) | state 1 and the other plate's record 0 at the start | hidden / shown at start pose | no, reverts at reload |
| 13, then 16, 18 (trigger 6: 14, then 16, 18) | script triggers 5, 6 | same; 16 and 18 only if still held at the re-read | animation plays once, each waited; effects 4-7 (trigger 5 stops them, trigger 6 never does) | no, reverts at reload |
| 12 hidden, 13 shown (trigger 6: 15 hidden, 14 shown) | script triggers 5, 6 | state 1 and the other plate's record non-zero | short branch: the pressed plate plays once (waited); effects 4-7 (trigger 6: 8-11) start then stop; panels untouched | no, reverts at reload |
| 16, 18, 13 hidden; 17, 19, 12 shown and play (trigger 6: 14, 15) | script triggers 5, 6 | state 0 and the other plate's record 0 | swap back; plays once (no wait) | no, reverts at reload |
| 13 hidden, 12 shown and plays (trigger 6: 14, 15) | script triggers 5, 6 | state 0 and the other plate's record non-zero | plate only | no, reverts at reload |
| 5 hidden, 4 shown, 4 plays | script trigger 4 (registered only while flag 78 is clear) | first fire, hero lacks item 29 | altar gate shuts (waited, 1.53 s) | no, reverts at reload |
| 4 hidden, 5 shown, 5 plays | script trigger 4 | 1.8 s after the hero holds item 29 | altar gate opens (waited, 1.0 s); flags 78, 269, 270 set | flag 78 saved; the pose is not |

**Pack 1248**: Exterior, a multi-object sequence with sound and a dialogue exit state

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0-5,31,33,35,37,39,41,43-46 | script trigger 27/16/18/17 | other script test | animation plays once | no, reverts at reload |
| 25 | script trigger 27 | other script test | shown at end pose (finished) | no, reverts at reload |
| 25 | script trigger 27 | other script test | animation plays once backward from the end | no, reverts at reload |
| 26,43-46 | script trigger 27 | other script test | shown at start pose (draw, collision, pose at time 0) | no, reverts at reload |
| 0-5 | script trigger 16/18/17 | other script test | hidden, collision and walls off | no, reverts at reload |
| 0-5 | at load | flag 79 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 25-30,33,37,41,43,45 | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 27-30 | at load | none | animation loops | rebuilt at each load |

**Pack 1249**: Exterior, trigger set with object swaps and teleport and world callbacks

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 13, 16, 20, 24 | at load | flag 80 clear | shown at end pose (finished) | rebuilt at each load |
| 0, 2, 3, 4, 8, 10, 15, 19, 23, 44, 46, 48 | at load | flag 80 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 1, 5, 6, 7, 9, 10, 13, 16, 17, 20, 21, 24, 25, 43, 45, 47 | at load | flag 80 set | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| effects 0-15 | at load | none | effect stops | rebuilt at each load |
| 12 | script trigger 2 | state 1, object word non-zero, fire-once word 0 | hidden, collision and walls off | no, reverts at reload |
| 10 | script trigger 2 | same | pose set to time 0, then animation plays once (waited, 1.27 s): the entrance gate lowers | no, reverts at reload |
| 12, 10 | script trigger 1 | state 1, object word non-zero | 12 hidden; 10 shown and set to time 0 (gate reopens at once); fire-once word cleared | no, reverts at reload |
| 10, 13, 1 | script trigger 3 | state 1 at the start | hidden, collision and walls off | no, reverts at reload |
| 0, 12, 11 | script trigger 3 | state 1 at the start | shown at start pose (draw, collision, pose at time 0) | no, reverts at reload |
| 11, then 12, 0 | script trigger 3 | state 1; 12 and 0 only if still held at the re-read | animation plays once, each waited (0.333 s, 3.33 s, 1.6 s); effects 12-15 start then stop | no, reverts at reload |
| 12, 11, 0 hidden; 10, 13, 1 shown and play | script trigger 3 | state 0 or released | swap back; plays once (no wait; 1 has no wall) | no, reverts at reload |
| 17, 16 hidden; 15, 14 shown; 14, then 15 play | script trigger 4 | state 1; 15 only if still held | effects 8-11 start then stop; waited | no, reverts at reload |
| 15, 14 hidden; 17, 16 shown and play | script trigger 4 | state 0 or released | swap back (no wait) | no, reverts at reload |
| 21, 20 hidden; 19, 18 shown; 18, then 19 play | script trigger 5 | state 1; 19 only if still held | effects 4-7 start then stop; waited | no, reverts at reload |
| 19, 18 hidden; 21, 20 shown and play | script trigger 5 | state 0 or released | swap back (no wait) | no, reverts at reload |
| 25, 24 hidden; 23, 22 shown; 22, then 23 play | script trigger 6 | state 1; 23 only if still held | effects 0-3 start then stop; waited | no, reverts at reload |
| 23, 22 hidden; 25, 24 shown; 24 plays (no wait), 25 plays (wait) | script trigger 6 | state 0 or released | swap back | no, reverts at reload |
| 47, 48, 5, 2 | script trigger 18 | object word non-zero, state non-zero, button 0 | lever down (48 shown): 47, 5 hidden; 48, 2 shown; 48 plays (wait); 48 hidden, 47 shown; 2 plays (wait). Lever up (47 shown): 2, 48 hidden; 47, 5 shown; 47 plays (wait); 47 hidden, 48 shown; 5 plays (wait) | no, reverts at reload |
| 45, 46, 6, 3 | script trigger 17 | same | same swap with 45, 46, 6, 3 | no, reverts at reload |
| 43, 44, 7, 4 | script trigger 16 | same | same swap with 43, 44, 7, 4 | no, reverts at reload |
| 9, 8 | script triggers 16, 17, 18 | the lever went up and the other two levers' up objects (47, 43, 45) are shown | 9 hidden; 8 shown at start pose and plays once (waited, 1.27 s); flag 80 set | flag 80 saved; the pose is not |

**Pack 1250**: Exterior, a flag-77 pickup ritual and a world-143 entrance

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 7,8 | table-driven | other script test and flag 77 clear and other script test and object 8 hidden | hidden, collision and walls off | unclear |
| 7,8 | table-driven | other script test and flag 77 clear and other script test and object 8 hidden | shown at start pose (draw, collision, pose at time 0) | unclear |
| 0,7,8 | table-driven | other script test and flag 77 clear and other script test and object 7 shown and item 29 not held | hidden, collision and walls off | unclear |
| 7,8 | table-driven | other script test and flag 77 clear and other script test and object 7 shown and item 29 not held | shown at start pose (draw, collision, pose at time 0) | unclear |
| 7 | table-driven | other script test and flag 77 clear and other script test and object 7 shown and item 29 not held | animation plays once | unclear |
| 0-9 | table-driven | other script test and flag 77 clear and other script test and object 7 shown and item 29 not held | effect starts | unclear |
| 0-9 | table-driven | other script test and flag 77 clear and other script test and object 7 shown and item 29 not held | effect stops | unclear |
| 8 | table-driven | other script test and flag 273 set and flag 275 clear | hidden, collision and walls off | unclear |
| 0,7 | table-driven | other script test and flag 273 set and flag 275 clear | shown at start pose (draw, collision, pose at time 0) | unclear |
| 7 | table-driven | other script test and flag 273 set and flag 275 clear | animation plays once backward from the end | unclear |

**Pack 1251**: Interior, an item-activation thread, an on-enter callback and an item-ask dialogue chain

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| - | entity proximity 0 | (flag 585 clear or flag 586 set) and flag 457 set and item 49 not held | world item spawned | flag written, not read at load by this pack |

**Pack 1257**: Exterior, a long trigger table, a teleport pair and a shop-style world swap

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | script trigger 0 | other script test and flag 676 clear | character activated | flag written, not read at load by this pack |
| 0 | script trigger 0 | other script test and flag 676 clear | character released (streaming may rebind) | kill bit, saved |

**Pack 1300**: Exterior, large entity-proximity set with flag-chained lines, an item-gated line, teleports with prompts and a boss-area set

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 2 | every tick | other script test | hidden, collision and walls off | rebuilt each tick |
| 2 | every tick | other script test | shown at start pose (draw, collision, pose at time 0) | rebuilt each tick |
| 2 | every tick | other script test | colour multiplier set | rebuilt each tick |
| 1 | script trigger 5 | other script test | hidden, collision and walls off | no, reverts at reload |
| 0 | script trigger 5 | other script test | shown at start pose (draw, collision, pose at time 0) | no, reverts at reload |
| 5 | entity proximity 18 | other script test | hidden, collision and walls off | flag 400 read at load |
| 5 | entity proximity 18 | other script test | shown at start pose (draw, collision, pose at time 0) | flag 400 read at load |
| 5 | entity proximity 18 | other script test | colour multiplier set | flag 400 read at load |
| 4 | entity proximity 18 | none | hidden, collision and walls off | flag 400 read at load |
| 8 | script trigger 0 | other script test | effect stops | flag 227 read at load |
| - | load thread, thread | none | character slot emptied (characters released) | no, reverts at reload |
| 4,6,7,10-13,table/argument | load thread, thread | none | character set loaded into a slot | no, reverts at reload |
| 0 | load thread, thread | none | character activated | no, reverts at reload |
| - | script trigger 1 | other script test and flag 222 clear | character slot emptied (characters released) | flag 222 read at load |
| 5,7,10 | load thread | flag 94 clear and flag 93 clear and flag 92 clear and flag 222 clear | character set loaded into a slot | rebuilt at each load |
| 12 | load thread | flag 94 clear and flag 93 clear and flag 92 clear and flag 222 clear and flag 397 clear | character set loaded into a slot | rebuilt at each load |
| 0 | load thread | flag 94 clear and flag 93 clear and flag 92 clear and flag 397 clear | character set loaded into a slot | rebuilt at each load |
| 2,14 | load thread | flag 94 clear and flag 93 clear and flag 92 clear | character set loaded into a slot | rebuilt at each load |
| table/argument | load thread | flag 94 clear | character set loaded into a slot | rebuilt at each load |
| 17 | thread | other script test | character set loaded into a slot | no, reverts at reload |
| table/argument | thread | other script test | entity force-spawned (kill bit cleared) | no, reverts at reload |
| table/argument | thread | not other script test | character set loaded into a slot | no, reverts at reload |
| - | script trigger 2 | other script test | character slot emptied (characters released) | flag written, not read at load by this pack |
| 21 | script trigger 2 | other script test | character set loaded into a slot | flag written, not read at load by this pack |
| table/argument | script trigger 2 | other script test | entity force-spawned (kill bit cleared) | flag written, not read at load by this pack |
| 1 | at load | flag 594 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 0,2 | at load | flag 594 set | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 4 | at load | flag 400 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 5 | at load | flag 400 set | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 8 | at load | flag 227 clear | effect starts | rebuilt at each load |
| 8 | at load | flag 227 set | effect stops | rebuilt at each load |

**Pack 1324**: Exterior, scripted triggers, object sets and a camera sequence

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 9-11,21-28 | script trigger 0/1/2/3/4 | none | hidden, collision and walls off | flag 239 read at load |
| 11 | script trigger 0/1 | not (other script test) and other script test | animation loops | flag 239 read at load |
| table/argument | script trigger 0/1 | none | animation loops | flag 239 read at load |
| 9,10,15,17,19,table/argument | at load, script trigger 0/1/2.../2/3.../3... | other script test | hidden, collision and walls off | flag 239 read at load |
| 14,16,18,20 | at load, script trigger 0/1/2.../2/3/4 | other script test | shown at start pose (draw, collision, pose at time 0) | flag 239 read at load |
| table/argument | at load, script trigger 0/1/2... | not other script test | animation loops | flag 239 read at load |
| 9,11 | script trigger 0/1/2/3... | not other script test | hidden, collision and walls off | flag 239 read at load |
| table/argument | script trigger 0/1/2/3... | none | movable block moved to a locator | flag 239 read at load |
| 0,table/argument | every tick | other script test | pose set to a computed time and frozen | rebuilt each tick |
| 21,table/argument | script trigger 2/3/4 | none | shown at start pose (draw, collision, pose at time 0) | flag 239 read at load |
| table/argument | script trigger 2/3/4 | none | movable block active := false | flag 239 read at load |
| 22,28 | script trigger 2/3/4 | not other script test | shown at start pose (draw, collision, pose at time 0) | flag 239 read at load |
| table/argument | script trigger 2/3/4 | none | movable block active := true | flag 239 read at load |
| 0-2 | script trigger 2/3/4 | not other script test | movable block active := false | flag 239 read at load |
| 16,18,20 | script trigger 2/3/4 | other script test | animation plays once | flag 239 read at load |
| 0-2 | load thread | none | movable block moved to a locator | no, reverts at reload |
| 29 | at load | none | animation loops | rebuilt at each load |
| 0,14,15,17,19,21,table/argument | at load | other script test | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 21 | at load | not other script test | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 16,18,20 | at load | not other script test | shown at end pose (finished) | rebuilt at each load |
| 0-4 | at load | not other script test | movable block active := false | rebuilt at each load |
| 3,4 | at load | not other script test | movable block moved to a locator | rebuilt at each load |

**Pack 1327**: Exterior, seven entity proximity routines, the Act 3 start, a target-practice thread, per-tick item checks and an animated-object offset loop

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| table/argument | at load | other script test | effect stops | rebuilt at each load |
| table/argument | at load | other script test | effect starts | rebuilt at each load |
| 1,5 | load thread | flag 94 clear and flag 93 clear and flag 92 clear | character set loaded into a slot | rebuilt at each load |
| table/argument | load thread | flag 94 clear | character set loaded into a slot | rebuilt at each load |
| table/argument | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| table/argument | at load | none | extra position offset set | rebuilt at each load |
| table/argument | every tick | other script test | extra position offset set | rebuilt each tick |
| - | load thread | other script test | character slot emptied (characters released) | flag 228,233 read at load |
| 0,6 | load thread | other script test | character set loaded into a slot | flag 228,233 read at load |
| 0 | load thread | other script test | entity removed permanently (kill bit) | kill bit, saved |
| 0 | load thread | other script test | entity force-spawned (kill bit cleared) | flag 228,233 read at load |
| - | load thread | other script test and flag 230 set | character slot emptied (characters released) | rebuilt at each load |
| 0 | load thread | other script test and flag 230 set | character set loaded into a slot | rebuilt at each load |
| 0 | load thread | none | entity removed permanently (kill bit) | kill bit, saved |
| 0 | load thread | none | entity force-spawned (kill bit cleared) | flag 228,233 read at load |
| 0,1 | at load | flag 92 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 2 | at load | flag 92 set | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |

**Pack 1335**: Exterior, six proximity routines, slot choices, a walk-to-entity sequence and movie triggers

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| table/argument | thread | other script test | character set loaded into a slot | no, reverts at reload |
| - | script trigger 5 | not other script test | character slot emptied (characters released) | flag 92 read at load |
| 0 | script trigger 5 | not other script test | character set loaded into a slot | flag 92 read at load |
| 1 | script trigger 7 | other script test and flag 240 set and flag 241 clear | hidden, collision and walls off | flag 241 read at load |
| 0 | script trigger 7 | other script test and flag 240 set and flag 241 clear | shown at start pose (draw, collision, pose at time 0) | flag 241 read at load |
| - | script trigger 7 | other script test and flag 240 set and flag 241 clear | character slot emptied (characters released) | flag 241 read at load |
| 1 | script trigger 7 | other script test and flag 240 set and flag 241 clear | character set loaded into a slot | flag 241 read at load |
| 7 | load thread | flag 94 clear and flag 93 clear and flag 92 clear | character set loaded into a slot | rebuilt at each load |
| table/argument | load thread | flag 94 clear | character set loaded into a slot | rebuilt at each load |
| 1 | at load | flag 241 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 0 | at load | flag 241 set | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |

**Pack 1344**: Transition, two movie triggers, an animated toggle pair and a slot thread

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 3 | load thread | flag 94 clear and flag 93 clear and flag 92 clear | character set loaded into a slot | rebuilt at each load |
| table/argument | load thread | none | character set loaded into a slot | no, reverts at reload |
| 2-5 | script trigger 3/4 | flag 322 set and other script test and other script test and other script test | animation plays once | no, reverts at reload |
| 2-5 | script trigger 3/4 | flag 322 set and other script test and other script test and other script test | animation plays once backward from the end | no, reverts at reload |
| 0-5 | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |

**Pack 1350**: Exterior, entity conversations and three doors gated on flag 100

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 100 clear and flag 99 clear | character set loaded into a slot | rebuilt at each load |
| 1 | load thread | flag 100 set | character set loaded into a slot | rebuilt at each load |

**Pack 1353**: Exterior, proximity conversations, button triggers and three threads

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 104 clear | character set loaded into a slot | rebuilt at each load |
| 2 | load thread | flag 104 set | character set loaded into a slot | rebuilt at each load |
| - | thread | none | character slot emptied (characters released) | no, reverts at reload |
| 1 | thread | other script test | character set loaded into a slot | no, reverts at reload |
| 3 | thread | not other script test and other script test | character set loaded into a slot | no, reverts at reload |
| 2,table/argument | load thread | none | shown at start pose (draw, collision, pose at time 0) | no, reverts at reload |
| table/argument | load thread | none | extra yaw set | no, reverts at reload |
| 3,4,6 | load thread | flag 694 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 5 | load thread | flag 694 set | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 3,4 | load thread | flag 694 set | shown at end pose (finished) | rebuilt at each load |
| table/argument | script trigger 0/1/2 | other script test | extra yaw set | no, reverts at reload |
| 3,4 | every tick | other script test | animation plays once | rebuilt each tick |
| 6 | every tick | other script test | hidden, collision and walls off | rebuilt each tick |
| 5 | every tick | other script test | shown at start pose (draw, collision, pose at time 0) | rebuilt each tick |
| 0 | world-activate callback | other script test | effect starts | re-run when the world becomes current |
| 0 | at load | none | effect stops | rebuilt at each load |

**Pack 1358**: Exterior, six teleports, proximity dialogue set and boss-area triggers

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| - | at load, script trigger 0 | none | ground-block collision layer word := a computed word | no, reverts at reload |
| - | at load, script trigger 0 | none | placed-object display layer word := 2 x a computed word | no, reverts at reload |
| 0 | load thread | flag 102 clear | character set loaded into a slot | rebuilt at each load |
| 1 | load thread | flag 102 set | character set loaded into a slot | rebuilt at each load |
| - | thread | not other script test and other script test | character slot emptied (characters released) | no, reverts at reload |
| table/argument | thread | not other script test | character set loaded into a slot | no, reverts at reload |
| 0 | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |

**Pack 1363**: Exterior, gated init, teleports, triggers and a long Mortog thread

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 2 | script trigger 0 | other script test | extra position offset set | no, reverts at reload |
| 0,1 | script trigger 0 | other script test | hidden, collision and walls off | no, reverts at reload |
| 0,1 | script trigger 0 | other script test | shown at start pose (draw, collision, pose at time 0) | no, reverts at reload |
| 0,1 | script trigger 0 | other script test | animation plays once | no, reverts at reload |
| 3,4 | script trigger 1 | other script test | animation plays once | flag 693 read at load |
| 0 | load thread | flag 106 clear | character set loaded into a slot | rebuilt at each load |
| 2 | load thread | other script test | extra position offset set | flag 692 read at load |
| 1 | load thread | other script test | hidden, collision and walls off | flag 692 read at load |
| 0 | load thread | other script test | shown at start pose (draw, collision, pose at time 0) | flag 692 read at load |
| 0 | load thread | other script test | animation plays once | flag 692 read at load |
| - | thread | other script test | character slot emptied (characters released) | no, reverts at reload |
| - | thread | not other script test and other script test | character slot emptied (characters released) | no, reverts at reload |
| 1 | thread | not other script test and other script test | character set loaded into a slot | no, reverts at reload |
| - | thread | not other script test | character slot emptied (characters released) | no, reverts at reload |
| table/argument | thread | none | character set loaded into a slot | no, reverts at reload |
| 3,4 | at load | flag 693 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 3,4 | at load | flag 693 set | shown at end pose (finished) | rebuilt at each load |
| 0,2 | at load | flag 692 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 0,2 | at load | flag 692 set | shown at end pose (finished) | rebuilt at each load |

**Pack 1369**: Exterior, two triggers with a button-gated walk

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0,1 | script trigger 0 | other script test | animation plays once | no, reverts at reload |
| 0,1 | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |

**Pack 1371**: Exterior, blackboard-gated start, triggers, hero death and a walk-in sequence

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 8 | thread | other script test | character set loaded into a slot | no, reverts at reload |
| 6,7 | thread | not other script test and other script test | character set loaded into a slot | no, reverts at reload |
| 4,9,10 | script trigger 2/3/4 | other script test and flag 100 clear and other script test | shown at start pose (draw, collision, pose at time 0) | no, reverts at reload |
| table/argument | script trigger 2/3/4 | other script test and flag 100 clear and other script test | animation plays once | no, reverts at reload |
| - | script trigger 2/3/4 | other script test and flag 100 clear and other script test | character slot emptied (characters released) | no, reverts at reload |
| 10 | every tick | none | shown at end pose (finished) | rebuilt each tick |
| 10 | every tick | none | animation plays once | rebuilt each tick |
| 4 | every tick | none | hidden, collision and walls off | rebuilt each tick |
| 9 | every tick | flag 342 set and flag 343 set and flag 344 set | animation plays once | rebuilt each tick |
| 8 | script trigger 1 | other script test | hidden, collision and walls off | flag 100,345 read at load |
| 0 | script trigger 1 | other script test | effect starts | flag 100,345 read at load |
| table/argument | thread | other script test | entity removed permanently (kill bit) | kill bit, saved |
| table/argument | load thread | not other script test | character set loaded into a slot | no, reverts at reload |
| 0 | at load | none | effect stops | rebuilt at each load |
| 8 | at load | other script test and flag 345 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 1 | at load | other script test and flag 100 clear and flag 342 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 1 | at load | other script test and flag 100 clear and flag 342 clear | shown at end pose (finished) | rebuilt at each load |
| 2 | at load | other script test and flag 100 clear and flag 344 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 3 | at load | other script test and flag 100 clear and flag 343 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 10 | at load | other script test | shown at end pose (finished) | rebuilt at each load |
| 9 | at load | other script test | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 4,9,10 | at load | not other script test | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |

**Pack 1381**: Exterior, a hub with six warp pads, six outer stations, a block-push group and a return-to-hub routine

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| effects 0, 2-13 | at load | none | effect stops | rebuilt at each load |
| 36,72,74, 37,76,78, 32,56,58, 33,60,62, 34,64,66, 35,68,70 | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 36,37,32,33,34,35 | at load | none | animation loops (0.4 s blend) | rebuilt at each load |
| 18,19,26-31,38-43,50-55,170-187 | at load | none | shown at start pose (draw, collision, pose at time 0); 20-25 are empty slots and do nothing | rebuilt at each load |
| blocks 0-5 | at load | none | movable block active := false | rebuilt at each load |
| 80-169 (light objects) | at load, world-activate callback, script trigger 31 (pad of station 3) | none | shown at start pose (draw only); trigger 138-227 bound to the return-to-hub routine | rebuilt at each load; re-run when the world becomes current |
| 84,88,91,98,101,105,108,112,113,122,126,131,140,146,153,157,158,162,166 | every tick | the object's own clock above 6.0 (clock := 0) | shown at start pose; trigger object + 58 bound to the return-to-hub routine | rebuilt each tick |
| same 19 objects | every tick | the object's clock above 2.0 and the object shown | hidden; trigger object + 58 bound to the no-op routine | rebuilt each tick |
| 0-5 (hazards, no mesh) | thread (six hazard threads, started 0.4 s apart) | none | shown at start pose, animation plays once (1.47 s), effect 2 + 2k follows it | no, reverts at reload |
| 0-5 | same thread, animation finished | none | hidden; effect 2 + 2k stops; effect 3 + 2k for one tick; trigger 12 + k bound to the damage routine for 1.5 s | no, reverts at reload |
| 36,37,32,33,34,35 (loop object of station k) | script trigger 63, 5, 18, 33, 39, 54 | hero melee or shot hits the target while object 50 + k is shown | animation stops where it is | no, reverts at reload; the restore does not stop or start it |
| piece objects (72,74), (76,78), (56,58), (60,62), (64,66), (68,70) | same | same | animation plays once at speed 0.5 (second one waited for) | no, reverts at reload; the restore leaves them at their start pose |
| 170+3k, 171+3k | same | same | animation plays once (3.0 s) | flag 708 + k, read at the world-activate callback |
| 172+3k and 38+k | same (172 + 3k first, 38 + k after the second camera track) | same | hidden, collision and walls off | flag 708 + k, read at the world-activate callback |
| 50+k | same | same | animation plays once (4.0 s), then hidden | flag 708 + k, read at the world-activate callback |
| 44+k | same | same | shown at start pose | flag 708 + k, read at the world-activate callback |
| block k, effects 31 + 4k to 34 + 4k | same | same | movable block active := true; four effects attached | flag 708 + k, read at the world-activate callback |
| 26+k (20+k is an empty slot) | movable-block contact (block group routine) | block k done word clear and armed word set | hidden, collision and walls off; flag 714 + k set | flag 714 + k, read at the world-activate callback |
| 18 | same routine | the sixth piece is done (once) | animation plays once with a wait (4.0 s) | flags 714-719 all set and flag 337 clear, read at the world-activate callback |
| 19 | same routine | same | hidden, collision and walls off after 18 ends | same |
| blocks 6-56 | pad of station 5 (trigger 52) | none | movable blocks 6-56 moved to locators 12-62 | no, reverts at reload |
| effects 2-13 and 0 | thread, script trigger (pads, return-to-hub routine) | pad or return used | effect starts (sparkle, hazard effects) or stops | no, reverts at reload |
| 170,171 | world-activate callback | flag 708 set | shown at end pose (finished) | re-run when the world becomes current |
| 38,50,172 | world-activate callback | flag 708 set | hidden, collision and walls off | re-run when the world becomes current |
| 44 | world-activate callback | flag 708 set | shown at start pose (draw, collision, pose at time 0) | re-run when the world becomes current |
| 0 | world-activate callback | flag 714 set | movable block moved to a locator | re-run when the world becomes current |
| 20,26 | world-activate callback | flag 714 set | hidden, collision and walls off | re-run when the world becomes current |
| 173,174 | world-activate callback | flag 709 set | shown at end pose (finished) | re-run when the world becomes current |
| 39,51,175 | world-activate callback | flag 709 set | hidden, collision and walls off | re-run when the world becomes current |
| 1 | world-activate callback | flag 709 set | movable block active := true | re-run when the world becomes current |
| 45 | world-activate callback | flag 709 set | shown at start pose (draw, collision, pose at time 0) | re-run when the world becomes current |
| 1 | world-activate callback | flag 715 set | movable block moved to a locator | re-run when the world becomes current |
| 21,27 | world-activate callback | flag 715 set | hidden, collision and walls off | re-run when the world becomes current |
| 176,177 | world-activate callback | flag 710 set | shown at end pose (finished) | re-run when the world becomes current |
| 40,52,178 | world-activate callback | flag 710 set | hidden, collision and walls off | re-run when the world becomes current |
| 2 | world-activate callback | flag 710 set | movable block active := true | re-run when the world becomes current |
| 46 | world-activate callback | flag 710 set | shown at start pose (draw, collision, pose at time 0) | re-run when the world becomes current |
| 2 | world-activate callback | flag 716 set | movable block moved to a locator | re-run when the world becomes current |
| 22,28 | world-activate callback | flag 716 set | hidden, collision and walls off | re-run when the world becomes current |
| 179,180 | world-activate callback | flag 711 set | shown at end pose (finished) | re-run when the world becomes current |
| 41,53,181 | world-activate callback | flag 711 set | hidden, collision and walls off | re-run when the world becomes current |
| 3 | world-activate callback | flag 711 set | movable block active := true | re-run when the world becomes current |
| 47 | world-activate callback | flag 711 set | shown at start pose (draw, collision, pose at time 0) | re-run when the world becomes current |
| 3 | world-activate callback | flag 717 set | movable block moved to a locator | re-run when the world becomes current |
| 23,29 | world-activate callback | flag 717 set | hidden, collision and walls off | re-run when the world becomes current |
| 182,183 | world-activate callback | flag 712 set | shown at end pose (finished) | re-run when the world becomes current |
| 42,54,184 | world-activate callback | flag 712 set | hidden, collision and walls off | re-run when the world becomes current |
| 4 | world-activate callback | flag 712 set | movable block active := true | re-run when the world becomes current |
| 48 | world-activate callback | flag 712 set | shown at start pose (draw, collision, pose at time 0) | re-run when the world becomes current |
| 4 | world-activate callback | flag 718 set | movable block moved to a locator | re-run when the world becomes current |
| 24,30 | world-activate callback | flag 718 set | hidden, collision and walls off | re-run when the world becomes current |
| 185,186 | world-activate callback | flag 713 set | shown at end pose (finished) | re-run when the world becomes current |
| 43,55,187 | world-activate callback | flag 713 set | hidden, collision and walls off | re-run when the world becomes current |
| 5 | world-activate callback | flag 713 set | movable block active := true | re-run when the world becomes current |
| 49 | world-activate callback | flag 713 set | shown at start pose (draw, collision, pose at time 0) | re-run when the world becomes current |
| 5 | world-activate callback | flag 719 set | movable block moved to a locator | re-run when the world becomes current |
| 25,31 | world-activate callback | flag 719 set | hidden, collision and walls off | re-run when the world becomes current |
| 18 | world-activate callback | flag 714 set and flag 715 set and flag 716 set and other script test | shown at end pose (finished) | re-run when the world becomes current |
| 19 | world-activate callback | flag 714 set and flag 715 set and flag 716 set and other script test | hidden, collision and walls off | re-run when the world becomes current |

**Pack 1382**: Exterior, six element pedestals, floor switches and the final-fight gate

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 2,4,6,7,10,12,13,28,table/argument | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| table/argument | script trigger (table) | other script test | effect starts | no, reverts at reload |
| 0,1,table/argument | script trigger (table)/2/3/4/5/6... | other script test | shown at start pose (draw, collision, pose at time 0) | no, reverts at reload |
| table/argument | script trigger (table) | other script test | character activated | no, reverts at reload |
| table/argument | at load, every tick | other script test | effect stops | rebuilt at each load |
| table/argument | at load, every tick | other script test | hidden, collision and walls off | rebuilt at each load |
| - | script trigger (table) | other script test | world item spawned | no, reverts at reload |
| 32-40,43,48,49 | every tick | none | entity removed permanently (kill bit) | kill bit, saved |
| - | every tick | none | character slot emptied (characters released) | rebuilt each tick |
| 28 | every tick | none | hidden, collision and walls off | rebuilt each tick |
| 1 | every tick | none | effect starts | rebuilt each tick |
| table/argument | script trigger 3/4/5/6... | none | hidden, collision and walls off | no, reverts at reload |
| table/argument | script trigger 3/4/5/6... | none | shown at start pose (draw, collision, pose at time 0) | no, reverts at reload |
| 0-3,table/argument | table-driven | none | character activated | unclear |
| table/argument | table-driven | none | entity force-spawned (kill bit cleared) | unclear |
| 0 | table-driven | other script test and entity 32 dead | character released (streaming may rebind) | kill bit, saved |
| 1 | table-driven | other script test and entity 33 alive | character released (streaming may rebind) | kill bit, saved |
| 2 | table-driven | other script test and entity 34 alive | character released (streaming may rebind) | kill bit, saved |
| 3 | table-driven | other script test and entity 35 alive | character released (streaming may rebind) | kill bit, saved |
| 1,2,21-26,table/argument | script trigger 2/3/4/5/6... | other script test | hidden, collision and walls off | no, reverts at reload |
| 27 | script trigger 2 | other script test | shown at end pose (finished) | no, reverts at reload |
| 15-20,27 | script trigger 2 | other script test | animation plays once backward from the end | no, reverts at reload |
| table/argument | script trigger 3/4/5/6... | other script test | animation plays once | no, reverts at reload |
| - | script trigger 11/12/13/14... | other script test | character slot emptied (characters released) | no, reverts at reload |
| table/argument | script trigger 11/12/13/14... | other script test | character set loaded into a slot | no, reverts at reload |
| 1-3 | at load | none | effect stops | rebuilt at each load |

**Pack 1391**: Transition, three triggers with a camera shake and an animation poll

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |

**Pack 1392**: Exterior, many scripted triggers, button prompts and a per-tick hero fall

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 10,12,14,20,22,24,table/argument | script trigger (table)/1/2/3/4.../25... | other script test | animation plays once | no, reverts at reload |
| 11,13,15 | script trigger 1/2/3/4.../11/12... | none | hidden, collision and walls off | no, reverts at reload |
| 10,12,14 | script trigger 1/2/3/4.../11/12... | not other script test and other script test | animation plays once | no, reverts at reload |
| 11,13,15 | script trigger 6/25/13/26/23/27 | other script test | shown at start pose (draw, collision, pose at time 0) | no, reverts at reload |
| 13-18 | script trigger 25/26/27 | other script test | effect starts | no, reverts at reload |
| 1,10,12,14,table/argument | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 11,13,15 | at load | other script test | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 20,22,24 | at load | other script test | shown at end pose (finished) | rebuilt at each load |
| 20,22,24 | at load | not other script test | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 13-18 | at load | not other script test | effect stops | rebuilt at each load |

**Pack 1393**: Exterior, object setups, triggers and threads

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 1, 10, 12, 27, 29, 49, 51 (room 0); 3, 14, 16, 31, 33, 53, 55 (room 1); 5, 18, 20, 35, 37, 57, 59 (room 2); 7, 22, 46, 39, 41, 61, 63 (room 3); 9, 24, 46, 43, 45, 65, 67 (room 4) | at load | none | shown at start pose (draw, collision, pose at time 0); the closed form (29, 33, 37, 41, 45) is the collision | rebuilt at each load |
| 49, 51 | script trigger 0 (room 0 opening) | hero inside, state 1, area-active 0, once-only latch clear | shown; 48, 50 hidden; 49, 51 play once (no wait); room 0 gate-state := 1 | no, reverts at reload |
| odd gates (49, 51; 53, 55; 57, 59; 61, 63; 65, 67) | script triggers 1, 7, 13, 19 (room entry; 19 does rooms 3 and 4) | state 1, area-active 1, room's gate-state 0 | shown at start pose; even gates (48, 50; 52, 54; 56, 58; 60, 62; 64, 66) hidden, collision and walls off (none); odd gates play once (no wait); gate-state := 1 | no, reverts at reload |
| entry levers 12, 16, 20, 46 (rooms 3 and 4 share 46) | script triggers 1, 7, 13, 19 | state 1, area-active 1 | animation plays once forward (0.75, wait) then backward (-0.75, no wait) | no, reverts at reload |
| plate levers 10, 14, 18, 22, 24 | script triggers 1, 7, 13, 19 | plate latch 1 | animation plays once backward (-0.6, no wait); latch := 0 | no, reverts at reload |
| 26, 30, 34, 38, 42 (door pictures); 27, 31, 35, 39, 43 and 28, 32, 36, 40, 44 hidden | script triggers 1, 7, 13, 19, alarm triggers 5, 9, 15, 16, 22 and the room threads | guard-door state 0 (open) and no hero on the plate | door picture shown at start pose; animation object and open form hidden, collision and walls off; door picture plays once (no wait, 4.0 s) | no, reverts at reload |
| 29, 33, 37, 41, 45 (closed forms) | same | same | shown at start pose (collision on at once, before the door picture finishes closing); door state := 1 | no, reverts at reload |
| plate levers 10, 14, 18, 22, 24 | script triggers 2, 8, 14, 23, 24 (plates) | state 1, area-active 1, plate latch 0, gate-state 1 | animation plays once (0.6, waited, 0.889 s); latch := 1 | no, reverts at reload |
| 27, 31, 35, 39, 43 shown; 26, 30, 34, 38, 42 hidden | same | guard-door state 1 (closed) | animation object shown at start pose; door picture hidden; then the animation object plays once (waited, 2.2 s) | no, reverts at reload |
| 28, 32, 36, 40, 44 shown; 29, 33, 37, 41, 45 hidden | same | same, 1.0 s after the door animation ends | open form shown (ground and contact), closed form hidden, collision and walls off; 1.0 s later the camera returns and door state := 0 | no, reverts at reload |
| even gates (48, 50; 52, 54; 56, 58; 60, 62; 64, 66) | alarm triggers 5, 9, 15, 16, 22 and room threads 0-3 (every 10 ticks while a guard is aggressive) | area-active non-zero, gate-state 1 | shown at start pose; odd gates hidden; even gates play once (no wait, 0.933 s); gate-state := 0 | no, reverts at reload |
| statues 0, 2, 4, 6, 8 shown; 1, 3, 5, 7, 9 hidden | same | gate-state 1 | shown at start pose and plays once (no wait, 4.2 s); the other hidden | no, reverts at reload |
| statues 1, 3, 5, 7, 9 shown; 0, 2, 4, 6, 8 hidden | same | statue latch 0, 2.0 s after the camera starts | swapped back to the static pose; latch := 1 (once per load) | no, reverts at reload |
| statues 1, 3, 5, 7, 9 | script triggers 4, 11, 18, 21 and the alarm handlers | hero inside; gate-state 0 | extra rotation about axis 2 toward the hero (rate 0.015) | no, reverts at reload |
| entity records of each room (guards) | room entry; trigger 0; alarm | entry: every populated record; alarm: gate-state 1 | entry: deactivated, moved to home locator, activated (a kill bit is cleared), behaviour 6; alarm: made visible and aggressive | kill bit cleared by the entry reset; otherwise no, reverts at reload |

**Pack 1394**: Exterior, timer and button hero-swap triggers with a hero-death gate

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 2 | every tick, herodeathcallback, script trigger 0/16... | other script test | extra offset and rotation cleared | no, reverts at reload |
| 2 | herodeathcallback, script trigger 0/1/10... | not other script test | extra offset and rotation cleared | no, reverts at reload |
| 2 | script trigger 16/17/18/15 | other script test | extra position offset set | no, reverts at reload |
| 0,2 | every tick | other script test | hidden, collision and walls off | rebuilt each tick |
| 0,2 | every tick | other script test | shown at start pose (draw, collision, pose at time 0) | rebuilt each tick |
| 0 | every tick | other script test | animation plays once | rebuilt each tick |
| 1,2,7,8,10,12,14,16 | script trigger 15/16/18/17/13 | other script test | hidden, collision and walls off | no, reverts at reload |
| 2-8,10,12,14,16 | script trigger 15/16/18/17/13 | other script test | animation plays once | no, reverts at reload |
| 1,2,7,8,11,13,15,17 | script trigger 15/16/18/17/13 | other script test | shown at start pose (draw, collision, pose at time 0) | no, reverts at reload |
| 5-8 | script trigger 17/13 | other script test | animation plays once backward from the end | no, reverts at reload |
| 9 | script trigger 11 | other script test | hidden, collision and walls off | flag written, not read at load by this pack |
| 3 | script trigger 11 | other script test | effect starts | flag written, not read at load by this pack |
| 1,3-6,10,12,14,16 | at load | none | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 7,8 | at load | none | shown at end pose (finished) | rebuilt at each load |
| 9 | at load | flag 0 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 3 | at load | flag 0 clear | effect stops | rebuilt at each load |
| 2 | every tick | other script test | extra position offset set | rebuilt each tick |

**Pack 1397**: Exterior, block-pushing puzzle room with button callbacks and ambient effects

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| - | table-driven | none | world item spawned | unclear |
| table/argument | table-driven | other script test and flag 687 clear | movable block active := false | unclear |
| 1,6 | table-driven | other script test and flag 687 clear | hidden, collision and walls off | unclear |
| 2,7 | table-driven | other script test and flag 687 clear | shown at start pose (draw, collision, pose at time 0) | unclear |
| 0,2,3 | table-driven | other script test and flag 687 clear | animation plays once | unclear |
| table/argument | table-driven | none | entity force-spawned (kill bit cleared) | unclear |
| table/argument | table-driven | none | animation plays once | unclear |
| 25,27,68,69,table/argument | table-driven | other script test | animation plays once | unclear |
| 8 | table-driven | other script test and other script test and flag 699 clear | animation plays once | unclear |
| 8 | table-driven | other script test and other script test and flag 699 clear | hidden, collision and walls off | unclear |
| 26,28,46,47,49-55,68,69,table/argument | table-driven | other script test | shown at start pose (draw, collision, pose at time 0) | unclear |
| 25,27,35,40,45,46,49-55,68,69,table/argument | table-driven | other script test | hidden, collision and walls off | unclear |
| 40,49-54,table/argument | table-driven | other script test | extra yaw set | unclear |
| 49-54 | table-driven | other script test | extra offset and rotation cleared | unclear |
| 41,48,table/argument | table-driven | other script test | extra position offset set | unclear |
| table/argument | table-driven | none | effect starts | unclear |
| table/argument | table-driven | none | effect stops | unclear |
| 12,table/argument | table-driven | other script test | effect starts | unclear |
| - | table-driven | other script test | world item spawned | unclear |
| table/argument | table-driven | other script test | effect stops | unclear |
| 0-4 | every tick | other script test | entity removed permanently (kill bit) | kill bit, saved |
| 41,48 | every tick | other script test | extra position offset set | rebuilt each tick |

**Pack 1398**: Exterior, triggers for a boss cage and a dialogue set

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| table/argument | at load, script trigger 34 | other script test | effect stops | flag 723 read at load |
| table/argument | at load, script trigger 34 | other script test | effect starts | flag 723 read at load |
| - | load thread | none | character slot emptied (characters released) | no, reverts at reload |
| table/argument | load thread | none | character set loaded into a slot | no, reverts at reload |
| 0-2,8,10 | every tick | other script test | hidden, collision and walls off | rebuilt each tick |
| 3,4,11 | every tick | other script test | shown at start pose (draw, collision, pose at time 0) | rebuilt each tick |
| 3-7,11 | every tick | other script test | animation plays once | rebuilt each tick |
| 3,4,11,table/argument | script trigger 38/40/41/42/43 | other script test | hidden, collision and walls off | no, reverts at reload |
| 1,2,10,table/argument | script trigger 38/40/41/42/43 | other script test | shown at start pose (draw, collision, pose at time 0) | no, reverts at reload |
| 1,2,8,10,table/argument | script trigger 38/35/40/41/42/43 | other script test | animation plays once | no, reverts at reload |
| 0,1,table/argument | script trigger 34 | other script test | character released (streaming may rebind) | kill bit, saved |
| 0,1 | script trigger 34 | other script test | character activated | flag 723 read at load |
| 8 | script trigger 34 | other script test | hidden, collision and walls off | flag 723 read at load |
| 7,9 | script trigger 34 | other script test | animation plays once | flag 723 read at load |
| 15 | every tick | other script test | effect starts | rebuilt each tick |
| 16 | script trigger 37 | flag 356 set | effect starts | no, reverts at reload |
| 0-2,5-7,9,10,15,17,19,21 | at load | flag 106 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 15 | at load | flag 106 clear | effect stops | rebuilt at each load |
| 1,2,10 | at load | flag 106 clear and flag 720 clear | shown at end pose (finished) | rebuilt at each load |
| 5,6 | at load | flag 106 clear and flag 721 clear | shown at end pose (finished) | rebuilt at each load |
| 7 | at load | flag 106 clear and flag 722 clear | shown at end pose (finished) | rebuilt at each load |
| 9 | at load | flag 106 clear and flag 723 set | shown at end pose (finished) | rebuilt at each load |
| 15,17,19,21 | at load | flag 106 clear and flag 724 clear | shown at end pose (finished) | rebuilt at each load |
| 1,2,5-7,9,10,15,17,19,21 | at load | flag 106 set | shown at end pose (finished) | rebuilt at each load |
| 16 | at load | none | effect stops | rebuilt at each load |

**Pack 1405**: Interior, a long entity-control helper with treasure-map and item-ask lines

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 104 set | character set loaded into a slot | rebuilt at each load |

**Pack 1407**: Interior, the same long entity-control helper as 1405, with a different dialogue

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 100 set | character set loaded into a slot | rebuilt at each load |

**Pack 1409**: Interior, proximity conversations and per-tick entity wander

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 106 set | character set loaded into a slot | rebuilt at each load |

**Pack 1411**: Exterior, script-trigger set with a shared sign routine, six entity dialogue routines, the mosaic and the Altador story steps

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| table/argument | every tick, load thread | other script test | effect stops | flag written, not read at load by this pack |
| table/argument | every tick, load thread | other script test | effect starts | flag written, not read at load by this pack |
| table/argument | load thread | other script test | hidden, collision and walls off | flag written, not read at load by this pack |
| table/argument | load thread | other script test | shown at start pose (draw, collision, pose at time 0) | flag written, not read at load by this pack |
| table/argument | load thread | other script test | animation plays once | flag written, not read at load by this pack |
| 1 | load thread | flag 98 clear and flag 96 set | character set loaded into a slot | rebuilt at each load |
| 0 | load thread | flag 98 set | character set loaded into a slot | rebuilt at each load |
| 2-4 | script trigger 0 | other script test and (other script test) | shown at start pose (draw, collision, pose at time 0) | flag 334 read at load |
| 0 | script trigger 0 | other script test and (other script test) | animation plays once | flag 334 read at load |
| 1 | script trigger 17 | flag 335 set and flag 336 clear | hidden, collision and walls off | flag 336 read at load |
| 0 | script trigger 16 | flag 337 set and flag 338 clear | effect starts | flag 338 read at load |
| 0 | at load | flag 334 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 2-4 | at load | flag 334 set | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 0 | at load | flag 334 set | shown at end pose (finished) | rebuilt at each load |
| 1 | at load | flag 336 clear | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |
| 0 | at load | flag 338 clear | effect stops | rebuilt at each load |

**Pack 1414**: Exterior, proximity conversations, teleport loads and a world-activation hero move

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 102 clear | character set loaded into a slot | rebuilt at each load |
| 1 | load thread | flag 102 set | character set loaded into a slot | rebuilt at each load |

**Pack 1417**: Interior, an on-enter dialogue, a random line and a long entity-control helper

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 102 set | character set loaded into a slot | rebuilt at each load |

**Pack 1419**: Interior, an on-enter dialogue, a random line and a long entity-control helper

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 100 set | character set loaded into a slot | rebuilt at each load |

**Pack 1421**: Interior, entity proximity conversation room

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 102 set | character set loaded into a slot | rebuilt at each load |

**Pack 1423**: Interior, entity proximity room with item ask-about

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 106 set | character set loaded into a slot | rebuilt at each load |

**Pack 1425**: Interior, a blackboard value on init, an on-enter dialogue and a long entity-control helper

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0 | load thread | flag 106 set | character set loaded into a slot | rebuilt at each load |

**Pack 1428**: Exterior, flag-gated trigger set, hero swap and act-start sequence

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| table/argument | at load, every tick, load thread | other script test | effect stops | flag 330 read at load |
| table/argument | at load, every tick, load thread | not other script test | effect starts | flag 330 read at load |
| 1,table/argument | every tick | other script test | effect stops | rebuilt each tick |
| 1,table/argument | every tick | not other script test and other script test | effect starts | rebuilt each tick |
| 1,table/argument | every tick | not other script test and other script test | effect stops | rebuilt each tick |
| 1 | load thread | none | entity force-spawned (kill bit cleared) | flag 330 read at load |
| 3,table/argument | load thread | none | shown at start pose (draw, collision, pose at time 0) | flag 330 read at load |
| 24-26 | load thread | none | effect starts | flag 330 read at load |
| table/argument | load thread | none | animation plays once | flag 330 read at load |
| 0,1 | load thread | none | entity removed permanently (kill bit) | kill bit, saved |
| 0 | load thread | flag 329 set and flag 330 clear and other script test and other script test | effect starts | rebuilt at each load |
| 0 | world-activate callback | none | effect stops | re-run when the world becomes current |
| 0,1,11-20,24-26 | at load | none | effect stops | rebuilt at each load |
| 2-10 | at load | other script test | effect stops | rebuilt at each load |
| 0-2 | at load | not other script test | shown at start pose (draw, collision, pose at time 0) | rebuilt at each load |

**Pack 1432**: Exterior, the final fight (hero swaps, fight clock, finale and timeout ending)

| Object / part | When | Condition | Change | Persists |
| --- | --- | --- | --- | --- |
| 0-2 | table-driven | none | shown at start pose (draw, collision, pose at time 0) | unclear |
| 3,4 | table-driven | none | effect starts | unclear |
| 18-25 | world-activate callback | other script test | effect stops | re-run when the world becomes current |
| 18-25 | world-activate callback | not other script test | effect starts | re-run when the world becomes current |
| 0-2 | table-driven | none | hidden, collision and walls off | unclear |
| 0,5-7 | table-driven | none | entity removed permanently (kill bit) | kill bit, saved |
| table/argument | table-driven | other script test | entity force-spawned (kill bit cleared) | unclear |
| table/argument | load thread | other script test and level state 11 set and other script test and other script test | entity force-spawned (kill bit cleared) | no, reverts at reload |
| 0-2 | world-activate callback | none | shown at start pose (draw, collision, pose at time 0) | re-run when the world becomes current |
| 3,4 | at load | none | effect stops | rebuilt at each load |

## Edge cases and error handling

- **Null world.** A registration with a null world is refused: nothing is registered (High).
- **Out-of-range object or entity index.** Object wrappers ignore an index outside the object table: nothing happens (High). Query functions then return false or zero (Medium).
- **Trigger index out of range.** Ignored (High).
- **No free thread slot.** ControlThreadStart starts nothing and returns no value; no caller reads a result (High). The same holds when ControlThreadsAllocate was never called.
- **Second ControlThreadsAllocate.** It replaces the table; threads started from the old table are no longer stepped. No disc module does this (High).
- **51st thread (no defined outcome in the original).** When all 50 thread slots of a world are alive and another thread is needed (a callback run, control thread, Update run or sequence), the original continues with a null thread; its behavior is not defined. No disc case is known to reach it. A clean build may refuse the start (Medium). Confirmed 2026-11-09 (High): the thread allocator prints a debug message and returns null when the pool is exhausted.
- **Re-enable after break (quirk).** SetBreakableActive(true) on a broken object returns it to state 1; its broken pieces are not restored (Medium).
- **Lock entry kept after opening (quirk).** A chest's key lock entry is not removed when the chest opens, so a later press of the same chest is blocked by the key rule again unless the object's active state gates the press first (Medium).
- **Failure messages (Medium).** Two failure paths carry debug messages in the original: an activation of an AI index with no matching entity (nothing is activated), and a script control-thread start with every thread slot busy (no thread is started). Whether this build prints either message is not established. The clean side must not depend on the output (Open question 62; no effect on play). Default if unresolved (rewrite choice): print nothing and take the no-effect path.
- **Debug calls.** DebugPrintf and the log-state call have no effect on game state. IsDebugMode returns 0, so every debug-gated branch is dead in play (High; section 8 of Behavior). Code that only follows a call to a debug-print helper (one that makes the debug query, prints and returns nothing) is not debug-gated and runs in play (High).
- **Waits have no timeout in SlotLoad.** The slot-loading wait (row 205) yields until the slot is no longer busy, with no counter, and then yields once more (High for the loop; a load that never completes stalls its thread).
- **Slot number -1 and out-of-range slots (High).** Every slot call (rows 205, 208 and 216) rejects a slot number outside 0 to n - 1, including -1. SlotLoad returns 0 with no teardown, load or spawn; SlotReset does nothing; SlotCharacterCount returns 0. The runtime passes the slot number unchanged, and no remap of -1 exists on the call path (High). A module-data slot word whose file value is -1 remains invalid until code stores a slot number in it. The static survey covers 170 module-data words used by slot calls in 92 modules; the earlier no-store/candidate classifications were rechecked. Four words remain supported as never stored (modules 837, 851, 864 and 956); the 1098 words are settled by their Init stores, and the non-word slot arguments of modules 758, 991 and 1023 are zone-record handles stored by Init (Open question 26). Rule for every module: a call whose slot argument is -1, or a slot word that holds -1 because nothing has stored a SlotCreate result into it yet, does nothing; a module whose Init stores a real slot number acts on that slot.
- **Slot word not yet stored.** A level module whose slot word is read before it is stored gets -1 and the load does nothing (returns 0 after one yield step). The survey confirms stores for the previously missed words in modules 722, 735, 745, 750, 842, 864, 906, 986, 1066, 1091, 1163 and 1327. Four words remain never stored: module 837 (one), 851 (one), 864 (one) and 956 (one). The five words in module 1098 are stored in its Init, on two alternative paths (Open question 26); three non-word slot arguments in modules 758, 991 and 1023 remain unresolved (Default if unresolved (rewrite choice): evaluate the argument as the module computes it and let a slot number outside 0 to n - 1 do nothing; needs a runtime check). Store-before-first-load ordering is established when Init stores the word before starting its load threads; callback-root ordering outside that pattern remains unconfirmed (Open question 26).
- **Failed region read returns 0 (High).** The release at the start of every SlotLoad sets the result to 0, and the completion step writes 1 only when the loaded data starts with the IDM signature. So a failed or missing read returns 0; no result of an earlier load survives (Open question 27).
- **Item ask-about never fires (quirk; High).** The ask-about record search accepts only item ids below 10, the table size, at registration and at fire time, and every registration on disc names an id from 57 to 174 (Behavior section 3, Item ask-about). The clean build either reproduces this (no callback, caption or collect block) or records the choice to make it work.
- **KillSound stops one voice.** Both KillSound forms (rows 225 and 229) stop only the first matching voice. With two voices of the same cue playing, a second call is needed for the second voice (High).
- **LoopSound without a refresh.** A LoopSound voice that no VisitLoopSound refreshes stops about 4 updates after it starts (row 228; High).
- **Leave-interior entry bound (quirk, reproduce; High).** StartLeaveInterior (row 278) bounds its entry index by the interior slot's entry count but takes the record from the exterior's list. An index valid for the exterior but at or above the interior's count lands on exterior entry 0; an index below the interior's count but at or above the exterior's count reads past the exterior's list. No level module passes an index other than -1.
- **SetGameOver ignores its argument.** Any value sets the same flag to 1 (High).
- **Animation state codes** (Medium): 0 = loop, 2 = play once, 4 = finished (after deactivation), 5 = animating (not written by the sampled wrappers), 6 = animating and finished, 8 = finished at the end of a seek, 9 = seeking to a time, 10 = stopped. IsObjAnimating is true for 0, 2, 5 and 6. IsObjAnimFinished is true for 4, 6 and 8. Writers (High for the constants): the boot program stores the constant codes 0, 2, 4, 8, 9 and 10 and no constant 5 or 6. The per-tick advance moves a clip that reaches its end from 6 to 4 and from 2 to 8, and for codes 0 and 5 it wraps the clip and keeps the code. The clip time grows once per tick by the 1/60 step times the speed, each product and sum truncated toward zero in single precision (SPEC-0008); the clip is finished on the first tick whose time is at or above its length, the length being (key count - 1) x 0.06666667 (single precision, also truncated). Because the truncated sum falls just short, the finished state arrives one tick later than the ideal arithmetic says for forward play (a clip of K keys at speed 1.0 takes 4 (K - 1) + 1 ticks); the rule for every speed, the backward case and the test vectors are stated once in SPEC-0028, Clip duration in ticks (High). The first advance after the call is in the next tick's world step. Nothing else writes the state word of a placed object: the on-disc values are 3 and 7, no script call passes a state number, and the other stores to a word at that offset belong to different records (characters, effects, a module's own set-up block). States 5 and 6 are therefore never reached in play (Medium-High): IsObjAnimating is true only in states 0 and 2, and IsObjAnimFinished only in states 4 and 8 (SPEC-0048, Open questions, R-0048 part 3). Resolved: states 1, 5, 6 and 7 have no writer anywhere (state 7 is a disc value of four objects of the unused pack 1298); the generic rules (drawn unless 3 or 4, stepped only in 0, 2, 5 and 6, no collision rule reads the state) are all they have, and a rewrite adds no special case (SPEC-0028, States 1, 5, 6 and 7).
- **Entity and hero request codes** (Medium): 0x01 = return to idle; 0x33 = one-shot animation (the wait polls for 0x33); 0x37 = loop animation; 0x34 = walking and orienting (WalkHeroAndOrient polls for it); 0x32 = walking (WalkHero polls for 0x32; WalkEntity polls the behaviour class 23 or 24 instead, row 106).
- **Waits without a timeout.** The waiting wrappers do not time out (Medium-High: the walk, play-animation and wait-till-leave-trigger wrappers each poll their state with a one-tick sleep and keep no counter or limit). A thread waiting on a state that never changes stays paused.
- **Repeated modules.** Ten packs use a module identical to another one (see Where it lives on the disc). A clean implementation can share one module definition across them.
- **Missing Exit.** 10 packs do not export Exit (section 1.2). Nothing is run for them on exit.
- **Module load failure (High for the cause).** A load-by-id returns 6 only when the module manager's 1,024-entry table is full. The disc has fewer than 200 level modules, so a normal run never reaches it; a clean build may treat a full table as a fatal error (Open question 38, Resolved).
- **Name-based load variant (High; no live caller).** The second form of the load-by-id routine tests the fourth byte of the module image header for the ASCII digit one (format version 1) and then unloads that same image if it is already linked. It is not a module-name test, and no caller of this form exists on the disc, so it never runs in play (Open question 39, Resolved).
- **Empty Update.** 96 packs have an empty Update. No per-tick work is done for them.
- **Empty no-op routines (High).** Rows 329, 330 and 373 have empty bodies. Each does nothing, whatever its arguments.
- **Argument-ignoring quirks (Medium-Low).** Each of these reads fewer of its call arguments than its name suggests, so a level that passes values gets no visible effect from them in this build. The behavior may be unfinished in this build; it is not confirmed as intended.
  - SetTargetColor (row 338): the four floats are not visible in the body, so whether a colour is set is open (Open question 58).
  - CharacterOnScreen in its world form (row 343): the arguments are not used; the current character is used.
- **Object index for animated-object queries.** Rows 296 to 302 and 311 pass their int to the object lookup, which selects the animated object with that index (Medium; the call-site argument is not shown). Rows 15 and 20 use the same index.
- **Hero-class methods.** Nine hero methods (rows 52, 53, 54, 171, 178, 182, 192, 193, 195) have their bodies confirmed. Their effects are High except row 178 (Medium). A hero index other than 0 or 1 is outside the defined range: IsHeroDead returns false for it, and the hit-point rules of SPEC-0036 do not apply to it. The five further hero methods (rows 348, 349, 350, 357 and 361) have rows in categories 20 and are Medium or High as marked.
- **Revive gate.** NeedHeroAlive (row 178) returns 0 and does nothing unless the swap mode equals 2 (free both ways) and the named hero is in the death state. When both hold, it revives the partner of the active hero if that partner is dead, and returns 1 (SPEC-0036, Revive by script).
- **ActivateHero block does not survive a swap (Medium; no disc site sets block).** With block set and the switch accepted, BlockPlayerControl puts h in the blocked state (0x21, that is 33) before the swap task runs; the swap task then sets the incoming hero to the one-shot state (0x33) and finally to the state carried from the outgoing hero, which is 1 because the move has just set it, so h ends up unblocked (row 187).
- **Template arguments.** A "T" argument reuses the type of the argument at that position. Where a call's last argument is a template reuse, the value is the same kind as the argument it repeats (bool or position).
- **Always-false and always-true queries.** DFS_Act1_IsTriggerHit (row 291) returns false in every case in this build. IsActStartDone (row 56) returns true in every case, so a wait on it ends at once (High). No level module calls it (Open question 25).
- **Wrong name in an error message.** RegisterOnEnterCallback (row 255) prints a null-pointer error that names RegisterActivateWorldCallback instead of its own name. This is a copy error in the original (Medium). A clean build may reproduce the null-pointer path; the wrong name is not behavior to copy.
- **SetToNormalCamera overloads.** Row 33 is the (int, bool) record and behaves exactly like ReturnToNormalCamera; row 43 is the (float, float, bool) record (distance, angle in radians, release). One site in module 1077 passes 180.0 as the angle; it is read as 180 radians, which is reproduced as is (High).
- **Duplicate host records.** Rows 135 and 332 (MoveEntity, three floats and CTL) share one host record. Their effects are as stated in the rows; both rows describe the same host record. Rows 201 and 364 (AddItemToWorld) are two distinct host records with the same name; only row 364 is imported by a module.

## Test vectors

- Sleep(3) called on tick 100 resumes on tick 103.
- Sleep(0) and Sleep(-2) yield nothing.
- Wait(0.5) yields 30 ticks at 60 ticks per second. Wait(0.7) yields 41 ticks (the single-precision 0.7 x 60 truncates to 41).
- LoopEntityAnim(e, a, 0.01, CTL) loops forever: 0.01 x 60 truncates to 0, which is stored as -1. LoopEntityAnim(e, a, 0.02, CTL) loops for 1 tick. LoopEntityAnim(e, a, -1.0, CTL) loops for 1 tick.
- SetAI_Behavior(e, 5, block with radius 20.0) on a code-0 townsfolk character: the mode is 0 at once, and on the next class tick it is 5, wandering with radius 20.
- Animation time 0.5 s: frame 7, fraction 0.5 (0.5 x 15 = 7.5).
- SetObjToTime(id, 2.0) on a clip of duration 3.0: time 2.0, stopped state (10). SetObjToTime(id, 5.0) on the same clip: time clamped to 3.0, finished state (8).
- Module load by image: a valid module image with one free manager slot is registered under its header name reference, and its load hook (header +0x18) runs before return 0. When all 1,024 manager slots are occupied and registration fails for capacity, the routine returns 6 after the load hook has already run.
- IsObjAnimFinished is true for states 4, 6 and 8, and false for 0, 2, 5 and 10.
- SlotLoad(W, slot 1, j 2) in pack 678 loads resource 681 (678 + 1 + 2).
- SlotLoad(W, slot 1, j 2, keep false, activate true) in pack 678, with the slot below the created count: the slot's previous characters are released and its result is set to 0, the character cap is set to 15, the load of resource 681 runs, the region's characters are set up when the read completes, and if the result is nonzero the region's item placements with flag 0 or 1 are spawned. Because keep is false, the per-tick update afterwards only releases characters that die and finish their death countdown. With keep true the cap is 6 (set by the release) and the keep flag is set, so the per-tick update streams the slot: it releases dead characters and characters farther than 280 units from the active hero, and binds the nearest eligible records up to the cap.
- SlotLoad(W, -1, j 2, keep true, activate true) returns 0 after one yield step and does nothing else.
- SlotAllocate(W, 1) followed by SlotCreate(W, 2, js) returns 0; a second SlotCreate on the same world returns -1 and reserves nothing.
- A level module whose slot word still holds -1 when SlotLoad runs: the call returns 0 after one yield step and nothing is loaded, whatever j and keep are.
- A level module whose init stores the SlotCreate result (0) into its slot word, then calls SlotLoad with that word: the value 0 is in range, so the load goes to slot 0.
- SlotCreate(W, 3, js = {0, 1, 4}) in pack 678 reserves a buffer sized from the largest of resources 679, 680 and 683 and returns slot number 0. Three further calls on the same world return 1 and 2, then -1 once all 3 slots have buffers (a fourth call with n = 3 returns -1 and reserves nothing).
- SlotCreate(W, 0, {}) still uses a slot number (it returns the next number, not an error).
- SlotCreate buffer size (row 207): R = 300,000 and c = 4 give 300,000 + 6388 x 4 + 20612 x 4 + 153 = 408,153 bytes. c = 0 gives R + 153. R = 100,000 and c = 20 give 100,000 + 6388 x 15 + 20612 x 20 + 153 = 608,213.
- SlotLoad whose read fails (the data does not start with the IDM signature), after an earlier successful load of the same slot: the call returns 0 and SlotCharacterCount then returns 0.
- SlotLoad(W, s, j, keep 1, activate 0) on a region with 3 character records and 5 flag-0 item placements: the 3 characters are set up (active count 3 when every setup succeeds) and no item placement spawns. The same call with activate 1 also spawns the 5 placements that pass the luck gate and the vetoes.
- SlotReset(W, s) after a successful load: SlotCharacterCount(W, s) returns 0, the cap is 6 and the region number is -1.
- ControlThreadsAllocate(W, 1); ControlThreadStart(W, f, p) twice in the same tick: the second call starts nothing, and neither call returns a value. When f's thread ends during step 4 of tick 100, its entry is free for a start made by a later step-4 thread in tick 100.
- FadeScreenOut(c, 0.7, WAIT 1): 41 frames, step 128 / 41. FadeScreenOut(c, 0.0, WAIT 0): 1 frame. A colour argument of 0xFF0000FF gives R 255, G 0 and B 0 (the top byte is ignored).
- Module 678's unresolved FadeScreenOut call: duration 1.0 s, so 60 frames and alpha step 128 / 60.
- Module 800's FadeScreenOut call: duration 0.5 s with WAIT 0, so 30 frames and alpha step 128 / 30.
- Module 800's FadeScreenIn call: duration 0.4 s with WAIT 0, so 24 frames and alpha step -128 / 24.
- A second module 800 FadeScreenIn call: duration 0.5 s with WAIT 0, so 30 frames and alpha step -128 / 30.
- Module 777's FadeScreenIn call: duration 0.5 s with WAIT 0, so 30 frames and alpha step -128 / 30.
- Module 678's second FadeScreenIn call: duration 0.8 s, so 48 frames and alpha step -128 / 48. Module 1371's FadeScreenIn calls: 1.0 s with WAIT 1, so 60 frames and step -128 / 60, and the caller yields until alpha reaches its target.
- Module 1174's LetterBoxOff calls: 0.5 s (30 frames) and 0.2 s (12 frames; the single-precision product 0.2 x 60 truncates to 12). Module 1217's LetterBoxOff call: 1.0 s, 60 frames. Module 1023's LetterBoxOff call: 0 s, 1 frame.
- LetterBoxOn(0.5, WAIT 1) while the bars are already held: the request is ignored, the done flag keeps its old value, and the wait follows that value (it ends once the flag is set).
- PlaySound(cue, null, noRestart 1) while a voice of that cue plays on the area slot: nothing starts. PlaySound(cue, p, noRestart 1) with p an object's position pointer (row 26) while a voice of that cue plays for another emitter: a new voice starts, and it follows the object.
- KillSound(cue) with two voices of that cue playing on the area slot: the first in pool order stops and the second keeps playing.
- LoopSound(cue, k, 0) with no VisitLoopSound: the counter starts at 4 and loses 1 per update, so the voice stops about 4 updates later. With VisitLoopSound(cue, k, 0) called every tick, the voice keeps playing.
- WalkEntity(e, k, WAIT 0, 0.0, CTL 1) with locator k at (10, 3, 5) and the ground at height 2.5 under it: the move goal is (10, 3, 5), the walk target is (10, 2.5, 5), class 23, base state 1 and control-flag word 1. IsEntityWalkToPointActive(e) is true at once.
- WalkEntityAlongPath(e, 2, WAIT 0, CTL 0): IsEntityWalkToPointActive(e) is true while class 35 is the current requested class.
- WalkEntityAlongPath(Wymond of pack 1023, -1, WAIT 0, CTL 1): the placement already holds path 8 (entries 146 to 156), so the walk starts at (-8.57, 29.37, 655.11) and ends after 11 points (SPEC-0056, TV-0056-R127a).
- WalkEntity(e, k, WAIT 0, 0.0, CTL 1), then GrabEntityControl(e) while the walk clip is at progress 0.5: the entity keeps moving along its frozen facing until the first tick the clip's progress is at least 0.99, then requests its idle and stands. ReleaseEntityControl(e): the walk resumes toward the same target. IsEntityWalkToPointActive(e) is true throughout.
- DoesHeroHaveItem(id) with a held count of 3 returns 3. RemoveItemFromHero(id, 1) then leaves 2; RemoveItemFromHero(id, 5) then empties the slot, and DoesHeroHaveItem(id) returns 0.
- GiveItemToHero(135, 20, false) adds 20 Neopoints. GiveItemToHero(136, 3, false) adds 5, and GiveItemToHero(139, 1, false) adds 100.
- RegisterItemAskAboutCallback(W, 57, cb, true), then the hero stands 2.0 units from a ready world item 57: no caption shows, the item can be collected and cb never runs.
- GetScriptGroupInstance(5, W) in a world whose script-group list has 5 entries returns 0. GetScriptGroupMainTrigger(0, W) on an entry with no trigger returns 0.
- A synthetic symbol record with name offset 0x100, value 0x220 and kind bits 0x00020000 is an export. A synthetic import with link number 7 binds to the host record whose key has link number 7, whatever the kind bits of the key.
- A module whose exports include the Transition Init entry belongs to the Transition family, whatever else it exports. At run time it gets Init, Update and Exit only when its pack is in the transition buffer T.
- Module lifetime (module 1139, one-time sale word with file value -1): the offer is accepted; entering an interior from 1139 and leaving gives no new offer, and walking into the adjoining transition and back gives no new offer. Walking on through the transition into another exterior and returning makes the offer again (the word is -1 again).
- Exterior X current, a movie with no return world: T is reloaded and X keeps its words. The same movie with T current: X (in E) is reloaded and its words reset. Loading a save: both resident modules are fresh.
- Pack 1429 streamed into E from 1427: its Init never runs, and its NPC proximity talk never fires.
- RegisterScriptTrigger(W, cb, index -1) registers nothing.
- A thread running PlayAnim(id, WAIT, 1.0) on an object in play-once state yields once per tick until the state changes from play-once.
- PlayCamera with track id out of range (for example 999 when the world has 50 tracks): the call does nothing and does not wait, whatever the wait option is (High for the range test).
- Hero HP ratio (row 52): hero 0 at 75 of a maximum 150 gives GetHitPointsPercentage 0.5, not 50.
- Hero HP set by fraction (row 53): SetHitPointsPercentage(hero 0, 0.35) with maximum 150 sets HP to 52 (52.5 truncated). SetHitPointsPercentage(hero 1, 1.5) with maximum 120 sets HP to 120.
- Hero HP queries (row 54): GetHitPoints on hero 0 at 40 HP returns 40.
- Death check (row 171): IsHeroDead(1) with hero 1 in state 16 returns true; IsHeroDead(0) with hero 0 in state 1 returns false; IsHeroDead(2) returns false.
- Revive (row 178): active hero 0, hero 1 in state 16, swap mode 2: NeedHeroAlive(1) finds the named hero 1 dead, then revives the partner of the active hero 0, which is hero 1: it is set to state 1 with HP at its maximum, and the call returns 1. The same call with swap mode 1 changes nothing and returns 0.
- Revive, named hero alive (row 178): active hero 0, hero 1 in state 16, swap mode 2: NeedHeroAlive(0) finds the named hero 0 alive, so the helper does not run; hero 1 stays in state 16 and the call returns 0.
- Revive with a fixed selector (row 178): active hero 1, hero 0 in state 16, swap mode 2: NeedHeroAlive(0) passes the named-hero death check and revives hero 0, returning 1. With active hero 0 dead, hero 1 alive and the same call, the check passes but the helper finds its target (hero 1) alive, so nothing changes and the call returns 0. The script selector is an absolute hero id, not an alias for the active hero.
- Magic (row 182): FillBothHerosMagicPoints sets the magic of heroes 0 and 1 to their maximum.
- Wallet at 500 Neopoints (rows 192, 193, 195): RemoveNeopoints(800) leaves 0. RemoveNeopoints(-50) leaves 500. RemoveNeopoints(0) leaves 500. GiveNeopoints(2000) gives 2500. GiveNeopoints(-10) leaves 500. GetNeopoints returns 0 when no wallet exists.
- Load by id, synthetic: id 7 with no loaded record loads the module, reads its record and returns 0.
- Load by id, synthetic: id 7 with a record already loaded uses that record and returns 0 without loading the module again.
- Load by id, synthetic: a load that fails returns 6.
- Reanimate timer (rows 374 and 375): SetReanimateTime(player, 2.0) then GetReanimateTime(player) returns 2.0. SetReanimateTime(player, 0.01) stores 0 ticks, so GetReanimateTime then returns 0.0. GetReanimateTime on an invalid player returns -1.

- Proximity, synthetic: a reference point 399 units from the record's position gives a squared distance of 159,201, below 160,000, so the action runs. A point 400 units away gives 160,000, which is not below the bound, so no action runs.
- Dispatch, synthetic: an entry of type 2 does nothing. An entry of type 4 with the float at offset 0x2C equal to 0 takes the first type-4 branch; with 0.5 it takes the second.
- Thread countdown, synthetic: a node whose counter is set to 3 gets its callback with state 2 on the third tick after the set. Ticks one and two do nothing.
- MoveHero(13, 1, 0, 0) in a world with at least 14 entries: the active hero's position becomes entry 13's position with y probed from entry 13's y + 4.0; the hero's state becomes 33; its facing and start pose are unchanged. MoveHero(13, 0, 1, 1): position and facing of entry 13, no probe, state 1. MoveHero(-1, ...) or an index at or above the count: nothing happens.
- SetHeroStartPos(8) in a world with 71 entries: no hero moves; both start poses become entry 8's position and facing. SetHeroStartPos(71): nothing changes.
- Exterior load, synthetic, p = -1, loaded world's entry 0 at (1, 2, 3) facing (0, 0, 1): start pose and position (1, 2, 3), facing (0, 0, 1), no probe.
- Exterior load, p = -2: the hero stays at its current position and forward, which become its start pose.
- Exterior load, p = 5 in a world with 2 teleport planes: the plane is refused and the entry 0 rule of p = -1 applies.
- Leave interior, synthetic, a = -1, b = 5, exterior entry count 71, interior entry count 3: b is at or above the interior's count, so exterior entry 0 is used, unprobed.
- PlayCamera(5, WAIT 1, CAMW 1, 0.5) with track 5 of length 2.0 s: the caller waits 4.0 s (240 ticks); the play then stays ended, holding its last pose, until ReturnToNormalCamera(-1, true), which also frees the hero.
- PlayCamera(5, WAIT 0, CAMW 2, 1.0) with the same track: at the end of the 2.0 s the hero is set to state 1 and the play goes idle without any further call.
- PlayCamera(5, WAIT 1, CAMW 0, 0.0): the track time never advances, so the play never ends and the caller never returns.
- StartCameraShakeMode(0, 0.5, 1.0): duration 0.5 s and amplitude 0.3; with preset 0's ramp of 0.5 s (SPEC-0027) the ramp starts at 0.0 s. StartCameraShakeMode(1, 5.0, 1.0): duration 5 s, amplitude 0.3, no ramp. StartCameraShakeMode(0, 1.0, 3.0): amplitude 0.9.
- SetToNormalCamera(10.0, 3.14159, true): the eye is at orbit distance 10 on the far side of the orbit (angle pi), and the hero is set to state 1. SetToNormalCamera(10, false) behaves exactly like ReturnToNormalCamera(10, false).
- Pack 909 locator 29 (position (-1949.414, 50.000, 1055.381), facing +z), MoveHero(29, 1, 1, 1): the probe ceiling is 54.0 and the one accepted patch there is 45.335, so the hero ends at (-1949.414, 45.335, 1055.381), 4.665 below the locator height, in state 1, facing +z, velocity zero, and does not fall (its ground gap is 0 at the next ground attach).
- Probe result cases: pack 732 locator 0 (x 3189.873, left of the grid, which starts at 3645.9) with probe on: the hero's y becomes 0.0. Pack 1098 locator 79 (y -128.279, only surface above the ceiling at -107.878): the hero is placed at -107.878. Pack 956 locator 2 (y 1.001) with probe off and control 0: state 33, the hero falls to the ground at -9.717 (gap 10.718, about 25 ticks) and stays in state 33.
- MoveHero(3, true, 0, true) in pack 678 (50 locators): the hero is placed on locator 3 at the probed ground height, facing the locator, in state 0x21. MoveHero(50, ...) in pack 678 does nothing. MoveEntity(entity 0, locator 2, probe on, CTL 1, face on) in pack 1432: the locator is at x 28.713, z -326.005, height 272.489; the probe ceiling is 276.489 and the highest accepted ground there is 270.628, so the entity ends at (28.713, 270.628, -326.005) in state 1; with probe off it ends at height 272.489.
- RegisterScriptTrigger(W, cb, 8) in pack 678 (trigger capacity 8) is ignored; index 7 registers.
- PlayAnim(obj, 0, -1.0) on a 3.0 s clip: the clip time is set to 3.0 s and the clip then plays backwards to 0.
- SetAnimObjAdditionRotY(obj, 90): additional rotation quaternion (0, 0.7071, 0, 0.7071) in (x, y, z, w) order. A later SetAnimObjAdditionRotY(obj, 45) gives (0, 0.3827, 0, 0.9239), not the sum of the two.
- SetObjToMayaFrame(obj, 45): object time 1.5 s.
- Random(1, 3) returns 1, 2 or 3; Random(3, 1) the same; Random(5, 5) returns 5 and draws nothing.
- HasMoviePlayed(92) is false whatever was played.
- PlayStingStream(4, 1.0) at game time 100 s plays; a second call at 130 s plays nothing; a third at 141 s plays.
- PlayHeroAnim(a, WAIT 0, CTL 1) called on tick 100 returns on tick 101.
- PlayEntityAnim(e, a, WAIT 1, CTL 0) waits until the animation is done, then the entity returns to its idle animation in state 0x21. A WAIT value of 2 would not wait (no disc site passes 2).
- IsButtonPressed(7, ...) is false at every one of its 45 sites.
- BlockPlayerControl(1) while Tor is active: Tor plays animation 0, the call yields one tick, and Roberta is put in state 0x21.
- DoesHeroHaveTreasureMap(203) tests map entry 3, the same as DoesHeroHaveTreasureMap(3); DoesHeroHaveTreasureMap(200) tests entry 200, which is false unless the map table has more than 200 entries.
- QueryGameState(743) returns 0; SetGameState(743, true) stores nothing.
- LevelMsg(500, W, b, 1) on a world with a 100-line text table: returns at once, the hero's state is unchanged and nothing is shown. LevelPrompt(500, W, -1): returns false at once.
- SetConversationTargetWithOrient(id, 1, 0): the entity's partner is the hero, the hero has no partner and does not turn.
- DisplaySquareButtonContext(4) while the word at 0x35C9A0 is 1: nothing is shown and the script-prompt flag is unchanged.
- Blackboard, synthetic: Add(type 1, 10, world 109), then Get(type 1, world 109) returns 10; Get(type 2, world 109) returns -1; Add(type 1, -1, world 109) leaves the entry in use with value -1. With types 1 to 8 present in one world, Add(type 9, ...) stores nothing.
- Blackboard reset: after Add(type 2, 30, world 109), starting a new game or applying a saved game makes Get(type 2, world 109) return -1; an area change does not.
- Trigger block, synthetic: a movable block numbered 3 inside a volume that the selected hero's cell lists fires with +0x28 = 3, +0x20 = 0 and object 0; a hero shot inside a flag-bit-3 volume gives object 0, +0x20 = 1, +0x24 = 0 and +0x28 = -1; the selected hero's body inside it during a hit-window frame gives object 1, +0x24 = 1, +0x20 = 0 and +0x28 = -1.
- Update rule, synthetic: record idle; a shot fires on frame 10 (state 1, object 0, stamp 10); the hero's body fires on frame 11 (object 1 over object 0 with state 1: state 1 again, stamp 11); the hero stays to frame 14 (state 2); the hero leaves; the script step reads 2 on frame 15, 3 on frame 16 and 0 on frame 17.
- Object-0 firer that stays: an NPC through CollideEntityWithTriggers inside a box from frame 20 on, record idle: state after the script step 1 (20), 1 (21), 3 (22), 0 (23), 1 (24); a WaitTillLeaveTrigger on that record returns although the NPC is still inside.
- Trigger registration: RegisterScriptTrigger(W, a, 5) then RegisterScriptTrigger(W, b, 5): a fire of index 5 runs b.
- InitMortogBehavior(W, 3, 6, 7, 25.0, 30.0) with locator 6 at (0, 5, 0) and locator 7 at (3, 9, 4): zone centre (0, 5, 0), squared radius 25 (the height difference is ignored), hold 25 to 30 s.
- Movie return transition: exterior current, no return world set, SetReturnTransitionForFMV(131), PlayFMV(40): at the hand-off the transition buffer reloads pack 1220, lands on entry record 0 and becomes current.
- GrabEntityControl(e) on an NPC whose clip is at progress 0.995: on its next update it requests its default idle animation and is not a hit candidate; ReleaseEntityControl(e) makes its stored AI requests apply again.
- Hero swap: a swap starting on pass 10 and ending on pass 70: script steps 2 to 5 do not run on passes 11 to 70; the special threads of step 1 run on every pass.
- Pause: while the code is 9, no script step of either world runs, so no script thread of either world advances on those passes.
- Second world: a loaded second world with an Update entry never runs its Update thread while it is not current; its enter-interior sequence (thread 1), when alive, is stepped once per pass in which the world branch runs.
- Per-tick order: a script that calls Sleep(1) in a control thread on tick 100 resumes in step 4 of tick 101; a trigger fired by the hero's trigger pass on tick 100 starts its callback in step 2 of tick 100; a module whose Update never yields runs Update once per tick; its first run is in step 1 of the tick after the world's first script step.
- Act-1 pairs (rows 81, 82, 345): ResetTriggers; SetTriggerData(4, true, false) returns 0; SetTriggerData(9, false, false) returns 1; IsValidTrigger(4) = 1; SetTriggerData(4, false, true) returns 0; IsValidTrigger(4) = 0; SetTriggerData(7, true, true) returns -1; IsValidTrigger(7) = 0. After 16 appends a 17th returns -1 and stores nothing.
- GetObjMaxTime on a clip with key count F = 31 returns 2.0; F = 1 returns 0.0; a missing object returns -1.0 (and GetObjCurrentTime returns -1.0 for it too).
- AlignAxisToVectorToHero(o, 2, 1.0) sets the Z column to the unit vector toward the hero; with t = 0.0 the Z column keeps its direction (re-normalised) and the call still returns the dot product before the turn.
- HasHitEntity(e) for an id that names no entity, with the hero's hit list holding at least one empty slot (-1): true.
- IsDoingAnyMeleeAttack with Roberta selected: false in every state.
- CharacterDeactivate(W, 0, 3) with record 3 bound in slot 0: returns 1 and the record can be bound again by streaming; a second call returns 0. CharacterDeactivate(hero-pointer) on a streamed character also sets its record's kill bit, so streaming does not bind that record again.
- SetTimerColor(0) sets the timer colour to (128, 128, 128, 128). StopTimer then GetTimer on later ticks returns the same value.
- GetClosestNavMeshId (synthetic): a walking entity at (0.2, 5.0, 0.2). Mesh 0 (kind 1) has one triangle with vertices (0, 4.0, 0), (1, 4.0, 0) and (0, 4.0, 1); mesh 1 (kind 1) has the same triangle at y 6.5. The point is inside both (edge values -0.2, -0.6 and -0.2). abs(h - y) is 1.0 for mesh 0 and 1.5 for mesh 1, so the result is 0. With mesh 1 at y 6.0 the two are equal, and the result is still 0. A flying entity at the same point tests kind 2 meshes only and gets -1.
- ItemSetPriceMultiplier(4.0): a buy candidate priced 25 becomes 100, one priced 9000 becomes 32767, and a second call takes 100 to 400. A record with flag byte 0 keeps its price.
- SetWorldLoadOverRide(165) sets the pending slot to 165, and after the next pending step it is -1 again. SetWorldLoadOverRide(861) and SetWorldLoadOverRide(-1) leave it unchanged. Pack 1427: trigger index 1 fires, then index 0 in the same tick: the pending slot is 188 and the second call does nothing; after the next Update, a fire of index 0 sets 187.
- IsThatAFireMote(160) and IsThatAFireMote(161) are true; IsThatAFireMote(162), IsThatAFireMote(174) and IsThatAFireMote(-1) are false.
- LaunchFireMote with a valid start and end locator: bool false aims toward the end locator; bool true aims toward the active hero. With no end locator, bool false is overridden and the active hero is used. With no start locator, no projectile is created.
- SetAnimObjColorMultiplier(2, 128, 128, 128, 127): object 2 draws with colour factors 0.5, 0.5 and 0.5 and alpha 0.5. With alpha byte 255, alpha is 1.0.
- PlayCamera(4, WAIT 0, end mode 1, 1.0) on a 2.0 s track: IsAnimatedCameraActiveAndFinished(4) is false while the track runs, true from the update in which it ends, and false again after ReturnToNormalCamera. The same play with end mode 0 or 2 never makes it true. Any other track index gives false.
- OverRideMidiSettings(64, -1, -1, -1): bank 74 plays on every controller tick until OverRideMidiSettings(-1, -1, -1, -1).
- StartCameraShake(1.0, 0.5, 12.0, 0.3, 0.5, 1.5707964, 0.2): key times 0, 0.3, 0.5 and 1.0; start amplitude 0.5; wave rate 12.0 on every key; aim factor 0.2.
- AddAnimObjSound(W, 3, 40, 0.0, 1.0) on an object with F = 31 frames (D = 2.0 s), played forward from time 0: slot 0 gets start 0.00002 s and end 1.99998 s; cue 40 starts on the first advancing tick and is stopped on the tick the time reaches 2.0. With a = 1.0 the start is 2.0 s and never plays. A fourth call on the same object leaves the slots unchanged.
- GetObjCurrentFrame on an object at time 0.5 s: k = 7, fraction about 0.5, result about 7.5. Missing object: -1.0.
- OpenLetterBox(0.5, WAIT 1, colour) runs over 30 ticks and the caller resumes once the bars report done; OpenLetterBox(0.01, ...) runs over 1 tick (0.01 x 60 truncates to 0, raised to 1).
- SetMidiExciteLevel(99) stores 99; SetMidiExciteLevel(100) and SetMidiExciteLevel(-1) change nothing.
- IsEntityReactingToHit on an entity in mode 19 with sub-state 10: true; mode 19 with sub-state 3: false; mode 5 with sub-state 10: false.
- TriggerBaggus(9) in a world whose baggus list has 8 records does nothing. TriggerBaggus(3) with 8 records breaks baggus 3 and spawns one item 145 at its transform.

## Callbacks that the function index of the level modules does not list

Tools that index level-module code by function start miss 117 callback and thread entry points in 86 modules, because each of these is reached only through a code pointer that the module hands to a registration or thread-start call. In 62 of them the pointer is the first routine of the module. All 117 are decoded from the module code, and each one is described in its module's section of the level-script specs (SPEC-0032, SPEC-0037, SPEC-0039 and SPEC-0041 to SPEC-0045), as the table names. 74 of them are in modules that the main story passes through. Confidence: High for the registration kind and the calls of each entry; the module section carries the confidence of its details.

Reading the table: "trigger n" is a script trigger with index n; "proximity" is an entity proximity callback; "filter" is the activation filter that RegisterAIPlayerActivateCallback installs (it receives a record ordinal and returns nonzero to admit that record; SPEC-0006). A routine marked "empty" returns at once; it is still registered, so the trigger or breakable that names it fires with no effect. A "latch" is a module data word that a trigger routine writes when the hero's body enters its volume.

| Module | Spec | Entries and what each does in play |
|---|---|---|
| 685 | 0032 | Trigger 0: sets map-page transition region 54. No state test. |
| 699 | 0032 | Trigger 0: sets flag 259 once; with flag 220 set and 258 clear it also shows line 6. |
| 702 | 0032 | Trigger 1: Square prompt (context 3) while the hero stands in the volume; pressing it shows line 3. |
| 706 | 0039 | Entity 0 proximity: three-line rotation, lines 4 to 6 (flag 63 clear) or 7 to 9 (set). |
| 732 | 0039 | Entity 1 proximity: flag 562 set gives the rotation 0, 1, 2; clear gives line 4. |
| 735 | 0044 | Breaker thread (decoded in that section): every tick, for each of the level states 1 to 7 that is set, it plays the break scene of that group of breakables and clears the state. Filter: refuses record ordinal 73, admits all others. Shared breakable routine of breakables 4 to 31: empty. |
| 753 | 0039 | Entity 5 proximity: line chooser by flags 86, 76, 258, 256 and the active hero. Slot thread: loads character sets by flags 256, 258, 427 and 428. |
| 757 | 0039 | Trigger 1 (flag 664 clear): the three-enemy ambush scene. Control is blocked, enemies 14, 15 and 16 are set to attack, and when all three are dead the objects are lowered and flag 664 is set. |
| 758 | 0045 | Trigger 1: clears a latch when the hero's body enters it. |
| 784 | 0037 | Entity 0 proximity: flag 591 set shows line 0; clear sets flag 591 and shows line 3. |
| 790 | 0039 | Trigger 1: plays object animations 3 and 4 and camera 5, returns the camera and waits until the hero leaves. |
| 793 | 0039 | Entities 1 and 2 proximity: flags 417 to 410 pick the line set; the item-13 exchange removes item 13 and sets flag 412. |
| 796 | 0037 | Trigger 1: Square prompt (context 4); pressing it runs the walk, the animations, the script-group switches and cameras 5 and 4. |
| 797 | 0032 | World-swap callback: arriving from world 753 or from another world selects one of two sequences (camera 4 or 5, hero placement at locator 1 or 0, an animation and a walk). |
| 799 | 0037 | Trigger 0: the lever scene (item 16 removed, flag 663 set) when flag 416 is set, line 0 when it is clear. Trigger 1: empty. |
| 809 | 0037 | Thread: loads character set 4, 3, 2, 1 or 0 for the first set flag of 58, 57, 56, 55, 54. |
| 816 | 0032 | Pickup callback: item 76 sets flag 517. The thread entry beside it is empty. |
| 822 | 0041 | Trigger 3: empty. |
| 855 | 0037 | Thread: opening line by flags 463, 462 (with item 22), 59 and 56. |
| 856, 857, 858 | 0037 | Thread: opening line by flags 59 and 56. |
| 864 | 0044 | Ambient thread: scenery objects play animations and effects after random waits of 0 to 30 ticks. |
| 874 | 0039 | Entity 7 proximity: scene with line 0, camera 4, hero and entity 7 moves, fade out, flag 290 set, slot reset and fade in. |
| 880 | 0039 | Trigger 1: clears the "talked" word so that entity 3 speaks again. |
| 886 | 0039 | Slot thread: loads sets 3, 2, 1 or 0 for flags 52, 51, 50 and 251, sets 4 or 5 by flag 250, otherwise makes AIs 16, 17 and 18 visible and sets agro on 16 and 17. |
| 906 | 0037 | Thread: loads set 0. |
| 934 | 0045 | Hold thread of trigger 1: while the hero stays in the volume it loads sets 1 and 10 by flag 248, waits for the hero to leave and resets the slot. |
| 947 | 0037 | Teleport 0: empty. Trigger 2: latch. |
| 950 | 0032 | Thread: long fall, camera 4, then the normal camera (distance 10) and idle. |
| 984 | 0039 | On-enter callback 0: line chooser by flags 67, 202 and 150. |
| 985 | 0041 | Trigger 4: empty. |
| 990 | 0042 | Trigger 2: camera 7, long fall, normal camera. |
| 1023 | 0045 | Triggers 38 and 40 (one routine): clear the latch of the trigger that fired. |
| 1055 | 0032 | World-swap callback: arrival from world 1098 or from another world selects hero placement (locator 1 or 3), camera 4 or 5, 60-tick waits and a walk. |
| 1056 | 0044 | Eight trigger routines (indices 5, 6, 7, 9, 10, 11, 12, 14): each is a latch for its own module word. |
| 1074 | 0032 | Thread: loads set 1 (flag 481 set, 482 clear) or set 0. |
| 1077 | 0042 | Activate-world callback (blackboard value for world 109: 10 or 20 moves the hero, long fall, flag 305); thread B (hero height patch type 7: reads locator 24 and overrides the secondary height); trigger 22 (the lever scene that sets flags 38 and 308). |
| 1080 | 0032 | Trigger 2: the hole fall (15-tick sleep, look-at camera, hero animation 137, camera shake). |
| 1081 | 0042 | Breakable routine: empty. |
| 1096 | 0041 | Trigger 6: posts value 130 for world 126 and starts the exterior load of world 126. Activate-world callback (the one registered when flag 45 is set): initialises object 1 with the flag true. Filter: refuses record ordinal 0, admits all others. |
| 1098 | 0045 | Entity 39 proximity: with flag 196 clear the long cutscene (lines 30 to 41, cameras 20 to 24) and flag 196 set (flag 206 first when 205 is on); later talks show lines 0 to 3. |
| 1122 | 0043 | Trigger 0: activates the transition assets. |
| 1131 | 0039 | On-enter callback 0 and thread: line chooser by flags 301, 216, 202, 150 and item 124. |
| 1136, 1140, 1143, 1172 | 0039 | Thread: line chooser by flags 301, 216, 202, 150 and 7 plus the module's own flags. |
| 1139 | 0039 | Trigger 0: empty. |
| 1148 | 0039 | Entity 2 proximity: lines 0 to 2. |
| 1150 | 0042 | Entity 4 proximity: the item exchange (items 60 and 39) that sets flags 139 and 142. |
| 1152 | 0037 | Entity 0 proximity: flag chooser; sets flag 162 in one flag state. |
| 1156, 1159, 1170 | 0032 | Teleport 0: posts value 70, 60 or 10 for world 106, then leaves the interior to world 107. The door of the exterior that enters the interior again is registered with context 13 (Square), so the return does not re-enter by itself (R-0257). |
| 1171 | 0042 | Trigger 6: Square prompt; pressing it runs the button scene (walk to locator 57, hero animation 140, fades, camera 7, looping sound 11). |
| 1174 | 0044 | Per-tick object thread (floor tiles, breakables, mask change). Shared breakable routine and one trigger routine: empty. |
| 1217 | 0045 | Act 3 arrival thread (hero 1 moved to locator 1, camera 18, long fall, hero camera 8) and a thread wrapper that runs the shared sequence routine once. |
| 1220, 1225 | 0037 | Thread: loads set 3, 2, 1 or 0 for the first set flag of 84, 83, 82, 81. |
| 1230 | 0039 | Slot thread: sets chosen by flags 248, 20, 19, 18 and 52. Trigger 4: clears the "talked" word. |
| 1238 | 0037 | Pickup callback: item 83 sets flag 579. |
| 1244 | 0039 | Trigger 0: Roberta's entry scene; revives a fallen hero, then fades, letterbox, camera 4 and lines 0 to 4. |
| 1246 | 0039 | Trigger 1: first entry only; hero walk and move to locator 11, boss-fight tweak 14, movie 41 once. |
| 1248 | 0039 | Trigger 4: the object 43 scene (camera 11, sound 16, objects 25 and 26). Trigger 1: latch. |
| 1249 | 0041 | Trigger 1: look-at camera 7, object 12 off, object 10 on, camera returned. |
| 1251 | 0037 | Thread: loads set 0 and activates the item for the flag 460 state. |
| 1253 | 0037 | Trigger 0: posts value 40 for world 141 and starts its exterior load. |
| 1254 | 0032 | World-swap callback: arrival from world 1253 moves the hero to locator 0, camera 6, hero animation 136. |
| 1257 | 0039 | Thread: creates a 1-entry slot and loads set 0. |
| 1259 | 0037 | Trigger: when the trigger index differs from the stored set, resets the slot and loads the set named by the index. |
| 1299 | 0037 | Teleport 0: flag 248 set posts value 80 for world 154 and loads it; clear shows line 0. |
| 1327 | 0044 | Teleport 0: the Act 3 start (flags 245 to 247, 400, 223 and 397; movies 27, 29, 90 and 30; act start 3; return world 64) or the plain load of world 151. |
| 1335 | 0043 | Trigger 4: empty. |
| 1344 | 0039 | Entity 27 proximity: lines 0 to 2. |
| 1350 | 0042 | Hero-use-item callback for item 49: empty. Trigger 1: clears a latch. |
| 1358 | 0043 | Entity 20 proximity: the delivery-task giver (SPEC-0059). Trigger 2: clears a latch. |
| 1363 | 0041 | Entity 42 proximity: random draw between line sets by flags 101 and 103. |
| 1369 | 0037 | Trigger 1: starts the exterior load of world 157. |
| 1381 | 0042 | Block-puzzle arm-point routine (sets the armed word of station k while the block touches its arm point) and an empty trigger routine. |
| 1382 | 0041 | Thread of trigger 2: loads set 7. |
| 1391 | 0037 | Trigger 2: once, camera shake, animation 0 and a control grab, then idle. Trigger 0: empty. |
| 1393 | 0044 | Triggers 0 (opening sequence), 7, 13 and 19 (room entry reset) and 12 (sets the area-active word). |
| 1398 | 0043 | Thread: loads set 5. |
| 1429 | 0037 | Thread: opening line by flags 93 and 92. |
| 1432 | 0039 | Filter: refuses ordinals 7, 6 and 5 (the three Minion Grunts), admits all others. Thread B (SPEC-0054). |

## Open questions
- Resolved (R-0127): the path argument of rows 111 and 339 is a path number of the area's path list (SPEC-0006, Tagged tables, tag 0x0A), kept in the walker's placement (+0xB8); -1 in row 111 means the placement's own path.
- **R-0119 Baggus shared-model follow-up (partly resolved):** SPEC-0004 gives the type-29 second-table layout, 4-byte payload-relative handles and pack-735 candidate 3. The shared-model selection and trigger behavior are described in row 64. The per-record manager renderer submits the 32 subordinate components from each record, but does not submit the shared model pointer at each record transform. TriggerBaggus moves the selected shared model to the chosen record transform before invoking its generic breakable-object transition; it also resets the record components, starts a generic effect at position +2 world units, plays cue 178 and spawns item 145 at the record. This does not establish whether another global renderer draws the shared model before the trigger. Still open: exact node-tree/submodel identity named by the handle, pre-trigger visibility, the burst effect identity, and the pairwise roles of the type-40 trigger indices.
- Default if unresolved (rewrite choice): preserve unnamed flag, object, locator and asset-group values as numeric identifiers scoped to their existing table or module; do not infer global names or roles.

- Resolved: a script thread's end does not change the hero's state (the thread-free step writes no hero state; section 4, Control threads). A scene end does not either. The hold state is left only by the calls named in its row (High). Still open: the nine hero animation calls whose CTL value passes through a shared helper.

1. **Wait multiplier.** Resolved: the multiplier is not in the file; it is written at start-up as 60 from the boot rate table at data address 0x30C598 (SPEC-0007, Rate selection).
2. **Enter test.** Resolved: the trigger's fire step writes the callback state word (Layout, section 3): 0 idle, 1 first fire, 2 repeated fire (a later fire on a different frame), 3 decaying toward 0 (two frames after the last fire, then 0 one frame later). The per-tick volume test of the hero's trigger pass (SPEC-0028, section 13) only decides yes or no each tick; a hit runs the fire step for the volume carrying that trigger's index (High for the writes and initial values; Medium for which callers run in play).
3. **Per-tick caller.** Resolved: Init is a plain call made once when the entries are bound; Update runs as a one-shot script thread that the world's per-tick script step creates whenever none is alive, and each tick that step runs, after the object updates, the special threads, the trigger callbacks with the trigger state decay, the other callbacks, the control threads in slot order and then the Update creation, in that order; steps 2 to 5 are skipped while the hero-swap in-progress flag (game-global word 0x35C980 + 0x44) is set, and the whole script step is skipped under code 9 (High); Exit is called at script-manager teardown before the module is unloaded (Behavior section 1, Per-tick script step; High).
4. **Family slots.** Resolved: the family is chosen by the buffer: E (the exterior buffer) calls the Exterior entries, I (the interior buffer) the Interior entries and any other world (T) the Transition entries (Behavior section 1.1; High).
5. **Enum values.** Resolved: the values used on the disc and how the host reads each (WAIT 0 or 1, tested nonzero, exactly 1 for PlayEntityAnim; CTL 0 or 1, compared with 1; CAMW 0, 1 or 2 as the play end mode; hero selector -1, 0, 1; shake type 0 to 2; axis selector 2 only; blackboard type 1 to 3; world identifiers 5 to 192) are in Layout section 2.1 (High).
6. **Flag and id meanings. Resolved in part:** eWorlds arguments are world-slot ids. For example, slot 68 selects pack 909 (Ellis Family Farm), slot 90 selects pack 1023 (Meridell Village), and slot 106 selects pack 1098, whose script registers map areas 48 and 49 (Meridell Castle 1st and 2nd Floor). A progress flag can be named when it matches a quest row’s start or done field: flag 107 starts Herding Whinnies; flag 110 completes Herding Whinnies and starts Talk to Mum; flag 174 completes Chores and starts Crokabek Nests (SPEC-0049). Other flags have no title in the quest table. Object and locator arguments are per-world indices, not global names: object wrappers address the current world’s object records (out-of-range indices do nothing), while locator arguments address its type-11 locator table and use the selected record’s position and, where applicable, facing vector. For example, pack 909’s Whinny pen setup uses locators 5 and 8; pack 1248’s Exit checks objects 0, 1 and 2. The operation and local record determine their roles; names for flag, object and locator indices are developer labels that no data on the disc encodes, so they are unknowable and the rewrite keeps the numeric indices (proactive (open close-out)). [High for the lookup rules and listed mappings; Medium-High for the bounded caller examples.]
7. **Bool arguments.** The meaning of the bool arguments in the rows marked M or L. Resolved for StartEffect (row 6: with the bool set, the camera block is passed as the start frame), PlayFMV (row 138: the bool is not used), GiveItemToHero (row 190: the plain pickup popup), PlaySound, PlayMainSound and LoopSound (rows 223, 233 and 228: do not restart), VisitLoopSound (row 232: no visible effect) and LaunchFireMote (row 76: true selects the active hero as target, false selects the end locator; a missing end locator forces hero targeting) (High). Resolved (proactive (open close-out)): PlayHeroSound (row 235) passes its bool in the same argument as PlaySound's noRestart (do not restart); the remaining bool arguments of M, M-H and L rows are the ones already described in their rows (InitObjOn, ItemActivate, the buy and pickup callback flag words, SetItemBlockMode); no bool argument of an M or L row is left without a stated role (Medium-High).
8. **Hero-class providers.** Established. The boot name table has a bound function for each of the nine hero-class names (rows 52, 53, 54, 171, 178, 182, 192, 193, 195). Their bodies were confirmed and their effects are stated in section 2. Row 178's return value remains Medium (question 19).
9. **Asset group mapping.** Resolved in part: GetAssetGroupCharacterRange selects the run of entity-id entries whose signed slot byte matches the key (row 215; SPEC-0006, Tag 0x12). The key is local to the current pack; membership is given by that pack’s entity-id table, not by a global group-name table. Examples from live callers and their pack data: in pack 934, key 4 contains three Crokabeks and key 6 contains eleven Flower/Spore Monsters; in pack 1300, key 17 contains Seradar, King Hagen, King Skarl, Lucy, Patricia, Hubert and Queen Fyora, while key 21 contains Old Knight, Master Torak, Servant Faun, Gord the Merchant, Old Lady, Kell the Beggar and Queen Fyora. Queen Fyora also occurs in key 17. Pack 777’s caller passes -1, selecting slot byte 0xFF; that pack’s table has no entries for that key, so the host follows its documented no-match output rule. Other callers use computed keys or other module-local memberships. These examples identify table membership, not designer-facing group names. Resolved (proactive (open close-out)): the key is the region number itself, so there is no further name-to-number rule; designer-facing group names are not encoded on the disc (unknowable), and a rewrite keeps the pack and numeric key. [High for the slot-key rule and listed memberships; Medium-High for the bounded caller set.]
10. **Unindexed callbacks.** Resolved: the 117 callback and thread entry points in 86 modules that the function index omits are decoded. They are listed by module, with what each does in play, in the section on callbacks that the function index of the level modules does not list; each is described in its module section (High). One statement in a module section was wrong and is corrected there: the second activation filter of module 1096 admits every record ordinal except 0; it is not empty.
11. **Negative animation time.** Resolved: PlayAnim's float is a speed factor, not a time; a negative value first moves the object to the end of its animation and then plays it backwards (row 2; High).
12. Resolved: **Header hooks.** Corrected 2026-11-09 (High): the load hook (+0x18) is one routine the loader calls once, with no arguments, after registering the module, and the unload hook (+0x1C) one routine called before unlinking; the word at +0x24 counts an extra-word table at +0x20 and is zero in every sampled module (Layout section 1). The rewrite implements level modules natively and needs only the load-time constructors' effects, which are given per module. Answers: proactive (open close-out).
13. **Symbol link rule.** Resolved (High): an import is bound to the host function whose table key has the same low 16 bits as the import's link number (Layout 1.1); the rule holds for all 5,171 imports and 509 exports on disc.
14. **Exit semantics.** Resolved: Exit is called when the world's script manager is torn down, just before the module is unloaded (Behavior section 1; High). It runs at the start of the world reset, before the world's own tables are cleared, so the hero, the item and flag stores and the on-screen timer are as play left them. After it the module is unlinked and the rest of the reset runs (actor entries, render and effect state, world sounds; SPEC-0055 Open question 1). The teardown cancels no script thread or callback task by itself and runs no callback other than Exit; their storage goes with the rewound pool. Three modules rely on Exit running: 800 (item 9 for the hero if missing), 934 (on-screen timer cleared) and 1248 (flag 79 set when objects 0 and 2 are inactive and object 1 is active); 160 of the 173 modules have an empty one and 10 have none. A module streamed into a buffer of another family (packs 1429, 1430 and 1431 in the exterior buffer) has no Exit bound, so none runs for it (High). Test vector: leaving pack 1248 with objects 0 and 2 inactive and object 1 active sets flag 79; with object 0 active it does not.
15. **Family changes.** Resolved: no. The family is selected once per load, at load completion, from the buffer that holds the pack, and its Update and Exit are stored on the world then. Swapping which buffer is current does not call the family entries again, and the same load completion step holds all the call sites. A pack reached through a different buffer is a fresh load with its own family call (Behavior 1.1; High).
16. **Trigger capacity.** Resolved: n = the first u32 of the world's type-11 chunk payload (0 when the chunk is missing; 0 in 69 packs; at most 228), and every resolved RegisterScriptTrigger index on the disc is in range. The other per-world table sizes are in Behavior section 3 (High).
17. **PlayCamera arguments.** Resolved: the arguments are (track, WAIT, CAMW end mode, speed factor); the third argument is the end mode, not an entity index; the float is a speed factor, not seconds; the target is chosen by the call kind (Note on camera calls; High).
18. **Slot keep flag and cap.** Resolved: the per-tick update of the slot reads the keep flag (1: full streaming pass; 0: release of finished dead characters only; section 5). In that mode the per-tick update binds no character; the cap of 15 then matches the at most 15 character objects that the setup at read completion can fill (Medium for the role of the cap).
19. **Revive target (row 178).** Resolved: the named hero (0 or 1 by absolute id) only decides whether the revive helper runs: it must be in the death state. The helper then acts only when the swap mode word equals exactly 2 (modes 0, 1, 3, 4 and 5 all refuse). The hero revived is the partner of the selected hero, and only when that partner is in the death state; it is set to state 1 and its health is raised by (maximum - current), so it stands at full health. The host call hands back the helper's result without change: 1 when a revive happened, 0 when the named hero is not dead, the mode is not 2 or the partner is alive; none of the four disc call sites reads it (High for the gate and the target; Medium-High for the returned value). Test vectors: mode 2, Tor selected, Roberta dead: NeedHeroAlive(1) revives Roberta (state 1, full health) and returns 1; NeedHeroAlive(0) returns 0 and changes nothing because Tor is not dead. Mode 2, Tor selected and dead, Roberta alive: NeedHeroAlive(0) runs the helper, finds the partner alive and returns 0, so Tor stays dead. Mode 3 or 1: NeedHeroAlive(1) with Roberta dead returns 0 and changes nothing. The known script survey has four calls in packs 1244 and 1394: both pass fixed selectors 0 and 1, and none passes -1 (the active-hero selector). Pack 1244 calls selector 0 when hero 0 is dead, then selector 1 when both heroes are dead; pack 1394's initialization checks hero 0 and then hero 1 when both are dead. These scripts name absolute hero ids; either id can be the active hero at runtime, so the selector does not itself mean active or inactive. High for the known caller survey and selector interpretation; ordinary-play reachability of pack 1244's dormant recovery path is not implied.
20. **Hero index range.** Resolved: the disc passes only -1 (51 sites), 0 (31) and 1 (29) as a hero selector; -1 is the active hero, 0 Tor and 1 Roberta (Layout section 2.1; High). The revive gate is the swap mode word (row 167); value 2 is the free setting (row 167, row 178).
21. **Camera shake argument order.** Resolved: StartCameraShakeMode takes (preset, duration in seconds, strength), with amplitude 0.3 x strength; disc sites include (0, 0.5, 1.0) four times and (1, 5.0, 1.0) twice. Every preset, including 0, receives the shake record (row 36; High). This matches SPEC-0027 (Camera shake).
22. **SetToNormalCamera argument types (rows 33 and 43).** Resolved: row 33 is the (int, bool) record and behaves exactly like ReturnToNormalCamera; row 43 is the (float distance, float angle in radians, bool release) record. The site in module 1077 that passes 180.0 is read as radians (High).
23. **Overload matching by table order.** Names with several host records were matched to rows in table order, and each matched row's argument list was confirmed against its host name. These agree: rows 28, 32, 33, 43, 45, 57, 60, 66, 112, 198, 201, 202, 213, 220, 225 and 229. Names in rows 295 to 388 with several overloads (rows 351 and 361, 365 and 366, 374 and 378, 375 and 377) have different argument lists and were matched by those lists. Rows 210 and 218 (CharacterNavmeshAssign) each match one host record with their own argument list. Row 135 (MoveEntity, three-float form) and row 332 are the same host record; row 135 gives the effect. Rows 201 and 364 (AddItemToWorld) are two distinct host records. Rows 135, 331 and 332 are now High.
24. **Rows 295 to 388.** Effects marked M name the called routine but did not check it, for example the entity and camera routines, the hit-point routine, the item-spawn routine and the flag store. Rows marked L have an argument mapping or return value that is not established. Resolved as not needed: no module imports any of rows 295 to 388 except AddItemToWorld (row 364, two modules; question 30), so the M and L rows are never called in play; Default if unresolved (rewrite choice): implement only row 364 and leave the rest as no-ops. Answers: proactive (open close-out).
25. **DFS_Act1_IsTriggerHit (row 291).** Resolved: it returns false in every case, and no level module imports it or the validity test of the same family; pack 1174 is the only importer of the act-1 family and calls only the set-indices, set-trigger-data and reset-triggers routines. Whether the test was unfinished or intended does not matter in play: no script reads it (High). A per-level spec must not assume the test can succeed.
26. **Slot number remap (-1) and module slot-word survey. Partly resolved.** The runtime passes the slot number unchanged; no remap of -1 exists on the path from a level module to the slot body, and the body rejects -1 with no effect (High). The local survey covers 170 module-data words used by slot calls in 92 modules. Reachable Init stores exist for words in modules 722, 735, 745, 750, 842, 864, 906, 986, 1066, 1091, 1163 and 1327. Four words remain never stored in reached code: module 837, 851, 864 and 956; their loads do nothing when reached. Resolved (proactive (open close-out); Medium-High): the five module-1098 words are all stored by slot-creation results in its Init. Two alternative paths set them: when either of two initial tests (on ids 30 and 31) passes, the Init creates a slot table of 1 entry for the first word and of 2 entries for the second word; otherwise it creates tables of 6 entries (first word), 7 (third word) and 3 (fifth word), and a table of 1 entry for the fourth word only when four further tests (ids 501, 72, 504 and 503) allow it. So on the first path the third, fourth and fifth words are never stored, and on the second path the second word is never stored; a load through an unstored word does nothing (an unstored word reads as -1, which the slot body rejects). Of 12 slot-call sites whose argument is not a module-data word, eight pass a SlotCreate result directly, one passes slot 6, and three (modules 758, 991 and 1023) pass a handle read from a 24-byte zone-record table that Init fills from SlotCreate results (Resolved, proactive (open question sweep, slot -1); Medium). Play outcome of the -1 cases on the main story path (High unless marked): module 864 never creates its second slot, so the two character sets its instance-load worker routes there (Otto the Old Miner and Elspeth) never load in the mine; modules 722 and 745 create each store under the same flags that gate the loads into it, so none of their loads reaches an uncreated store (Medium-High for 745); module 956 never stores the word for its set-1 load (Illusen's earlier set), which does nothing, while Illusen still appears from set 11 loaded by its other thread; modules 837 and 851 sit in dormant code, so their loads are never reached. Init stores that precede the module's load-thread starts establish the ordering for those threads; store-before-first-load ordering for callback-root stores outside that pattern remains open (Medium; see also questions 2 and 28; Default if unresolved (rewrite choice): run Init stores first, then load threads and callbacks in the order the module lists them). A per-level spec should preserve these unresolved cases rather than infer that their loads are empty.
27. **Failed read and stale result.** Resolved: the release at the start of every SlotLoad and SlotReset sets the result to 0, and the completion step writes 1 only when the data starts with the IDM signature, so a failed read returns 0 and no earlier result survives (Edge cases; High).
28. **Slot below capacity with no buffer (partly resolved).** Resolved: such a slot's buffer handle is all zero (start, position and end 0), so the region read has no target buffer (High). Resolved for the result: it is 0 after the release and becomes 1 only when a read completes with data that starts with the IDM signature (High). Whether the archive layer fails such a read does not change the script-visible result, which stays 0 either way (needs a runtime check to see the engine's error handling). Default if unresolved (rewrite choice): return 0 and load nothing.
29. **Slot freeing.** There is no free call. A world-level reset (not named here) empties every created slot and sets the created count to 0, so SlotCreate numbering restarts at 0. A rewrite frees the table and buffers only at that world reset. Default (rewrite choice); no other freeing path is described.
30. **Module call sites for rows 295 to 388.** Resolved: a scan of every import of the 163 distinct modules shows that only AddItemToWorld (row 364) is imported, by 2 modules; no module imports any of the other 93 names (High).
31. **Argument-ignoring quirks.** Whether the original build intended the quirks listed under Edge cases to use their arguments, or whether they are unfinished: developer intent, unknowable. What matters is the behaviour: reproduce each quirk as listed under Edge cases (the argument is ignored as stored), which is what the shipped call sites exercise.
32. **Return values of wrappers.** Resolved: the query wrappers that end in a call (rows 45, 57, 60, 66, 142, 176, 181, 196, 200, 240, 354, 355 and 371) return the callee's result unchanged, and every module site of the imported ones reads that result. Their meanings are in the rows (High).
33. **Reanimate timing constants.** The reanimate timers use the tick rate and a seconds-to-tick step. Resolved: both are written once at start-up from the boot rate table at data address 0x30C598, as 60 ticks per second and 1/60 s per tick (SPEC-0007, Rate selection); they are not disc constants.
34. **Key-value table.** Resolved in part: section 9 describes a host-global 16-entry pair store. The script-facing wrapper discards the append/set result, so a script cannot observe a full-table or missing-key return through that call. Module 1174 uses key 1 for its Act 2 start record and keys 5–53 for hazard, phase/checkpoint, breakable, barrier-trigger and floor-tile records. Resolved (proactive (open close-out); High): only module 1174 imports this call family (SetIndices, SetTriggerData and ResetTriggers), and the read-side calls (IsTriggerHit and IsValidTrigger) are imported by no module, so no other script has a key set and no other consumer exists in shipped play; nothing reads a full-table result.
35. **Id registry.** Resolved in part: section 10 holds runtime DLL class-instance addresses, not item, script-module or entity ids. Registration and unregistration use it through object-lifecycle paths (High). Still open: which spec should own the broader DLL lifecycle description, the particular class represented by each caller, and whether any level-script API consumer exists.
36. **State numbers**: what the 0x3C-byte record and the global object states in section 11 mean, and which scripts or modules write them. The owner of the mode-byte routine is not confirmed.
37. Resolved: **Load by id: init on reuse and the init field (High).** There is no reuse. Registration never compares names or numbers, so it never matches an existing entry, and a module holder that still has a linked module is unlinked first (its unload hook runs and its manager slot is freed) before the next image is linked. The loader's init field is the module container's load hook at +0x18 (section 1): one field, which holds an offset in the file and an absolute address after linking. It is called once, with no arguments, right after the image is linked and just before it is registered, and it runs the load-time constructors. The family Init (section 1) is a different call made later by the world's load-completion step. In play: every load of a world into a buffer relinks its module from the pack bytes, runs the constructors and then runs the family Init again, so a revisit that reloads the world starts it from the disc values (all triggers, callbacks, threads and slots rebuilt by Init; only state kept outside the module, such as game flags, the blackboard and the save data, persists). A revisit where the world stays resident (leaving E for an interior and coming back, an E and T swap, a streaming step to an already resident pack) runs neither the constructors nor Init again (section 1.1). Test vector: enter exterior 956, go in through its door to interior 984 and back out: 956's Init runs once (on its first load), 984's Init runs on every entry; if a later load replaces E and 956 is loaded into E again, its constructors and Init run again. Default if unresolved (rewrite choice): none needed.
38. **Load by id: failure and source of the id.** Resolved for the known image-buffer path: world-data type-37 parsing and boot startup provide the module image; the key is the image-header name reference at +0x14 (an address of the module-name string, not a number). The call returns 6 when the module manager refuses registration because its 1,024-entry table is full. The load hook runs after linking and before registration, so it has run even when the full table gives 6; this path has no existing-record reuse branch (High). Other uses or failure causes, if any, remain unconfirmed.
39. **Load by id: second form.** The branch checks byte +3 of the SNR image header for ASCII '1' (format version 1), then calls the manager to unload that same image pointer. It proceeds only if the pointer is already linked; this is not a module-name test. No direct callsite for this form appears in the known boot, level-module, or DLL records, so no on-disc module name is shown reaching it. **[High for the branch and precheck; no live caller found]**
40. **MoveEffect forms and helper axis.** Resolved for the forms: row 32 is the point form; row 324, and row 28 when its face argument is set, use the direction form; row 28 with face clear uses the point form (High). Resolved: the two fixed fallback vectors are world +Z and +X; the 0.1 branch tests one extracted component of the normalized cross-product result, and fallback selection uses a separate component-magnitude comparison. Explicit point and direction constructors set their fourth lane to 0; the no-direction form selects point mode without a fallback axis. **[High for these static rules]** Still open (Medium-Low): exact edge behavior for zero or near-zero direction vectors.
41. **MoveHero bool and placement mode.** Resolved: the second argument turns on the probe (from y + 4.0), the control argument selects the state (1 for control 1, 33 otherwise), and the fourth argument selects a move with facing (nonzero) or position only (zero). The move is immediate and does not change the start pose (row 156; High).
42. **WalkEntity wait codes.** Resolved: the WAIT of WalkEntity yields while the entity's current requested class is 23 or 24 (row 106), and WalkEntityAndOrient's while it is 25 or 26 (row 108); neither polls request code 0x32 (High).
43. **Shared camera-play routine.** Resolved: the six camera-play calls (rows 34, 37, 38, 39, 42 and 318) share one core and pass (track, WAIT, CAMW end mode, speed factor), with the target chosen by the call kind; rows 37 and 39 behave identically, and rows 38 and 318 differ only in the object anchor (0x190 look-at, 0x180 track). Row 313 (SetAiHotspotDistCamera) is a separate host function, not one of the six, and no module imports it (High).
44. **CAMW argument of rows 37 and 39.** Resolved: the camera-wait value is read in both rows as the play's end mode (High).
45. **Query bodies with no visible return value.** Resolved: each of rows 45, 57, 60, 66, 142, 176, 181, 196, 200, 240, 354, 355 and 371 returns the callee's result unchanged; every module site of the imported ones tests or uses the result. The rows now state the answers (High).
46. **Seconds-to-tick factors (rows 104, 168, 239, 374, 375, 377 and 378).** Resolved: the factor that converts seconds to ticks is not a constant in the disc image; it is written once at start-up from the boot rate table at data address 0x30C598, which holds 60.0 (SPEC-0007, Rate selection). The rate is 60 ticks per second and the reanimate step is 1/60 second (High). A fade with the given seconds and a one-frame minimum (rows 136 and 137) depends on the same factor.
47. **Registry key (answered).** The low 16 bits of each host-table key are the link number that module imports and exports carry, and bits 16 to 23 are the kind (Layout, sections 1.1 and 1.5). The key is not a hash of the name and not an argument count. Confirmed on the disc: every import and export matches a table record by name, and its link number equals the key's low 16 bits.
48. **Family entry names.** Resolved: the Init, Update and Exit names of the three families are records 397 to 405 of the host table, with kind 1 and no bound function. When a module is attached, the host selects the family, resolves the family's three exports by name, stores them on the world and calls Init at once as a plain call; Update runs as a script thread created by the per-tick script step; Exit is called at script-manager teardown (Behavior section 1; High).
49. **Unnamed or inferred callees.** Resolved for DisableInvisibility (row 121): it sets the entity's invisibility-off word (High). The number formatter behind FloatToChar (row 98) is not named. The meanings of rows 333 (IsAI_Flying) and 347 (CloneHero) are inferred from names and callees. Rows 282, 384 to 386 are resolved as prior area-slot getters (SPEC-0006, Prior and active world slots). Row 283 (StartTransitionScene) is resolved: it stores the pack for a scene-driven swap and has no live caller (row 283).
50. **Empty bodies (rows 329, 330 and 373).** Resolved (High): the bodies are empty and no level module on disc imports any of the three names, so no script can reach them; there is nothing to implement. Test vector: a script that called row 373 would see no change to any item.
51. Resolved: **Camera record (rows 316, 319 and 320).** The camera profile record is the one static record the follow camera reads. OverRideCameraAttributes(distance, elevation, minimum height) turns on the record's override flag and writes the three floats into the script profile's entries of the profile tables (follow distance, default elevation, minimum eye height; SPEC-0027, Camera profile record); the other fields of that profile keep their fixed values (stiffness 0.085, spin gain 0.005, decay 0.125, damping 0.583095). The follow camera reads the flag on every normal-follow tick that has a ground patch under the focus: with the flag on it picks the script profile whatever the ground kind, and it reloads the live fields only when the picked index differs from the current one. A second call with new numbers while the script profile is already current therefore changes the stored table entries but not the live fields until the profile changes. RestoreCameraAttributes clears the flag; the next such tick picks the ground-kind profile again and reloads it. The written numbers stay in the table. SetHeroDistCamera passes its three floats straight into the fallback-pose entry as reach R, height h and yaw offset o (distance sqrt(R^2 + h^2), elevation asin(h / R)); SetAI_DistCamera maps its floats (first, second, third) to R, yaw offset and height, so the two calls take the last two numbers in opposite orders. No module calls any of the four. **[High]** Test vectors: (a) profile 0 current, OverRideCameraAttributes(12.0, 0.3, 2.0): on the next follow tick with a ground patch the live follow distance is 12.0, the default elevation 0.3 and the minimum eye height 2.0. (b) Then RestoreCameraAttributes while the hero stands on ground kind 4: the next tick loads profile 1 (distance 9.0, elevation 0.25, minimum height 1.0). (c) OverRideCameraAttributes(12.0, 0.3, 2.0), one tick, OverRideCameraAttributes(14.0, 0.4, 3.0), one tick: the live distance stays 12.0. (d) SetHeroDistCamera(8.0, 2.0, 0.0) with the hero at the origin facing +z and height value H: distance 8.2462, elevation 0.25268, eye at (0, H + 2.0616, 7.9844), looking at (0, H, 0); SetAI_DistCamera(id, 8.0, 0.0, 2.0) gives the same pose for an entity at the origin facing +z.
52. **Callee bodies.** Resolved for the camera shake (row 36): the host body gets the shake record and calls the preset selector, which multiplies the strength by 0.3 (data word at 0x2DB2F0) to give the amplitude and passes the duration unchanged, with the preset's other values, to the shake start. The start stores the duration as the end time and the amplitude as the start amplitude and in the two middle keys (SPEC-0027, Camera shake record; High). Test vectors: StartCameraShakeMode(0, 0.5, 1.0): end time 0.5, amplitude 0.3, key times 0, 0.3, 0.0 and 0.5, key amplitudes 0, 0.3, 0.3 and 0, wave rate 12.0 on every key. StartCameraShakeMode(2, 2.0, 3.0): end time 2.0, amplitude 0.9, key times 0, 2.0, 2.0 and 2.0. StartCameraShakeMode(7, 1.0, 1.0): no shake. Still not confirmed: the counter and timer effects (rows 246 to 249 and 369 to 372; rows 244 and 245 are resolved by SPEC-0023 section 31, High). The vibration routine behind row 325 is resolved: it is the rumble request of SPEC-0033 (preset, duration in seconds, magnitude), and no level module calls it (High). Until the counter and timer routines are confirmed, the effects in those rows stay as their confidence marks state.
53. **Angle use in the rotation rows.** Resolved for rows 19, 23 and 27: the angle is in degrees, converted with pi/180 (data word at 0x2DB480) into a half-angle unit quaternion about the chosen axis, which replaces the earlier additional rotation (High). Rows 306 to 309 follow the same rule with the axes given in their rows (Medium-High). Row 303 differs: its first float is a radian angle that picks the horizontal axis and only its second float is the rotation, in degrees (Medium-High). No module imports rows 303 or 306 to 309.
54. **Callback stores (rows 269 and 368).** Resolved for row 269: the use-item registration stores the callback and its flag in the world's script record, as the buy and pickup registrations do (row 269). Resolved in part for row 368: the wrapper and shared-record helper do not write the callback; the helper matches an existing record using the callback and item-proximity context, then increments its +0x30 word. Where that matching callback record is initially stored and what the counter means remain open.
55. **State value 10 (row 68).** Resolved: the word is the attack sub-state of the selected hero and the hero index word 0 is Tor, so the query is true only during Tor's power step, the charged Mote swing (SPEC-0036, State 13; Medium-High for the label, High for the test).
56. **Character index quirk (row 222).** Resolved in part (High for the use): the only caller on disc is pack 1398's trigger 34 routine, whose caller is a non-hero firer of that trigger (the routine refuses every fire by the hero, SPEC-0043), so the excluded ids 0 and 1 (the heroes) never matter in play; whether a character with id 100 or 101 can fire it is not known. The reason for excluding them is not known. Test vector: a character whose id is 0 gets -1; any other id gets its record ordinal (the grouped lookup returns the ordinal itself, row 222), and the pack 1398 routine uses that result plus 16: the Evil Harris (type 70, class 17) is not among the excluded ids.
57. **AddItemToWorld records (rows 201 and 364).** Resolved: two distinct host records carry the name: row 201 (int, bool, bool, float), a scripted add of a current-region placement, and row 364 (int locator, int item, bool, bool, float), the only one any module imports; its locator count is the type-11 locator count on the disc (High). Resolved: in both rows the first bool is "snap to ground", the second is "placed", and the float is the item's lifetime in seconds, forwarded to the item spawn (row 201 forwards it too). A negative lifetime means the default: 15.2 s for a charm, 11.2 s for any other item, and no expiry for a placed item. All 118 call sites on the disc pass snap true, placed true and lifetime -1, so every scripted item is a placed item that never expires (High). Still open: the value row 364 returns.
58. **Target colour (row 338).** Resolved in part: for a valid index the four float arguments are copied, without numeric conversion, into the first 16 bytes of the marker record; an invalid index does nothing. Vector: with two targets, index 1 and arguments (0.25, −0.5, 1.0, 0.0), record 1's first four float slots become those values and record 0 is untouched. The role of those four fields (including channel order and rendering use) remains open.
59. **Argument order and types.** Resolved for PlayStingStream (row 227): the arguments are (stinger id, float volume) (Medium-High). Resolved for SetHeroStatus (row 184): the arguments are (int status code, float seconds), sent as the record (code, seconds, -1, -1) to the current hero's status-apply method (High). Resolved for SetAI_DistCamera (row 315): the entity id is the first argument and the floats are reach, yaw offset and height (Open question 51; High). Still open for SetAiHotspotDistCamera (row 313; the entity id is taken from the third argument position) and CloneHero (row 347; the hero enum is not visible). Check each against the host signature before a per-level spec relies on it.
60. **Body-only meanings.** Resolved for RestorePlayerControl (row 166): the hero selector is used (row 166; High). WhichHeroIsActive (row 162) and IsHeroSwapInProgress (row 180) are stated in their rows (High). Resolved for IsThatAFireMote (row 72): its argument is a Mote item id, and it is true for 160 and 161 (High). Resolved for KnockbackHero (row 289): the host wrapper calls the current hero's knockback routine without forwarding its declared float; the script call does not use that value. The downstream routine's full effect is not established by this wrapper (High).
61. Resolved: **Switch-group prompt thread.** The 29/30 loop is the kind-4 switch group trigger-0 callback, entered after a repeated trigger fire; 29 and 30 are Square context prompts selected by the element-match test, not level-script host events or sound cues (section 4; SPEC-0048, Prompt thread). The exact callback dispatch binding route is described with the world interaction group.
62. **Debug messages on failure paths.** Resolved (High): this build never prints them. Each such print sits behind a shared debug gate routine whose whole body returns 0; 58 host routines, including ActivateEntity and the control-thread start, test it. A failed call therefore changes nothing except its documented return value. Test vector: ActivateEntity with an id outside the entity-id table does nothing and shows no message.
63. **ActivateEntity argument (row 116).** Resolved (High): the lookup and the (never printed) failure text use the same number, the entity id, which is the position in the world's entity-id table (SPEC-0006). An id below 0 or not below the table's count finds nothing and does nothing; otherwise the entry gives the pool slot and ordinal that the bind uses (a negative slot means the world's embedded block). Test vector: with a table of 40 entries, ids 0 and 39 are looked up and 40 and -1 do nothing.

64. Resolved: **Thread flag and head rule (High).** The 0x30-byte node with the flag at 0x14 is not a script thread; it is the compiler runtime's caught-exception record (Behavior, Caught-exception record). The flag is the caught flag, the counter is the handler count, and state 2 is the call that destroys the thrown object at the end of the last handler, not a wake-up. The head rule is the runtime's guard that keeps an exception alive while it is still in flight. A zero-flag newest record whose counter reaches zero stays (counter -1) and is never finished, but this affects no script thread and no other record, and the case cannot arise in the shipped game: nothing throws and no level module imports a throw. Exact scheduling rule of script threads: Behavior, Scheduling rule of script threads (each live thread runs once per tick in the fixed step order until it yields; no thread blocks another from finishing). Test vector: a script thread that calls Sleep(3) at tick 100 resumes at tick 103 at the same place in the step order; a thread stuck in a wait loop never ends but the other threads still run every tick. Default if unresolved (rewrite choice): none needed; implement no exception record.
65. **Trigger kind tables.** Resolved: the numbers 3, 5 and 8 are flag bit numbers (bit 3 for kinds 1, 2 and 3, bit 8 for a static rectangle, bit 5 for a moving rectangle); the handler tests are in section 3, Movable block pass, and SPEC-0028 (High).
66. **Trigger fire hook (Low).** Resolved in part: on the actor kind dispatcher a hit goes to the actor's trigger-hit method, which fires with object 1 for the hero, object 0 for the monster class and nothing for the base class (Behavior, section 3, Fire paths). Still open: whether the hook at offset 0x70 of the character is that method, what it does with the word at 0x98, and whether a null hook means the fire step has no effect on the character.
67. **SlotLoad cap after keep true.** Resolved: slot construction sets the cap to 15 and then runs the reset, which sets it to 6, so a fresh slot and a slot after SlotReset have cap 6. The release at the start of every load also sets the cap to 6, so a keep-true load always leaves 6 and a keep-false load sets 15 (row 205; High).
68. **Switch setter callers.** Resolved for keys below 200: those keys are collected-table rows of chests and element-sealed chests, read at setup and set when the chest opens, so they never needed the switch path (Behavior, section 2.1; SPEC-0048, Lock chests; High). Resolved for row 89: it is the Whinny pen-list setter and does not use the batch form (High).
69. **Approach radius (row 251).** Resolved: the registration index is the character descriptor id, and the table has one entry per character descriptor of the world. The table is the talk table read by the hero's world interaction chooser (SPEC-0022, Talk entry). The approach radius is the 20-unit talker list of the talk candidate plus the 5.8-unit range of the "Talk" caption and press; the 400-unit proximity test of section 3 does not apply (High).
70. **Price multiplier field (row 197).** Resolved (High): the scaled 16-bit field is the buy price of a buy-candidate record. The shop prompt's price caption and the purchase check both read that same field (SPEC-0050, sections 3 and 9), so a multiplied item shows and charges the multiplied price. The earlier reading as a plain placement count is withdrawn. Test vector: a flag-1 record priced 25 shows 25 before the call and 100 after ItemSetPriceMultiplier(4.0), and the purchase takes 100.
71. **Hero trigger-hit record lookup.** Resolved: the record-lookup argument is 0 on the cell list walks (a plain lookup of the trigger index), and on the walk over the inline volumes of an animated placed object it is that object, so the record is the object's own instance trigger (Medium-High).
72. **Hero trigger step input.** Resolved: cell lists A and B of the grid cell under the selected hero's feet point and the inline volumes of animated placed objects in reach (SPEC-0028, Behavior, Trigger pass; High).
73. **Activation filter store (row 259).** Resolved: the callback is stored in the world and passed on to the embedded block and to every slot created so far; each block's eligibility test calls it with the record ordinal. A slot created after the call has no filter (High).
74. **Init order against the post-load placement.** Resolved: Init is called inside the pack's load-completion step, right after the chunk walk, so before the arrival placement and before the loaded world is made current; the activate-world and on-enter callbacks start as threads when the world becomes current, so after the placement. A start pose stored by Init (module 777's SetHeroStartPos(8)) is overwritten by the placement, and calls in Init that act on the current world act on the world that is current during the load (section 6, Init order; High).
75. **Leave-interior bound with no interior list (Low).** The bound of row 278 when the interior slot has no entry list at all; the original reads the count through an empty reference.
76. **SlotCreate size term F.** Resolved: F = 384, the sum of two fixed pool sizes (16 x 8 + 16 and 16 x 14 + 16), so the buffer is R + 6388 x min(c, 15) + 20612 x c + 153 bytes (row 207; High).
77. **ActivateHero against SPEC-0053 (row 187).** Resolved: ActivateHero takes five arguments (h, locator, block, fade, cue). It moves the hero that is selected when the call is made (not h) at once onto the locator (probe on, CTL 1, face on), then requests a switch to h with fade and cue, then, when block is set, blocks h at once. Nothing in the call itself is deferred; if the switch runs, its placement step copies the outgoing hero's placement (now the locator) to h, so h arrives on the locator (swap mode 4: h's own stored spot). The two disc sites, both in module 1244, pass false for all three bools (row 187; High). Confirmed again from the body; SPEC-0053, section 3.3 and SPEC-0039 (module 1244) already agree, so no further change is needed.
78. Resolved: **Alignment words (rows 176, 181 and 354).** The getters return the active hero's three equipped Mote ids: code 0 is armour, code 1 shield and code 2 weapon. An empty slot returns -1; otherwise the value is the equipped Mote's item id (SPEC-0021, Equipment slots; High). Synthetic vector: equipment words (armour, shield, weapon) = (-1, 166, 161) produce armour = -1, shield = 166 and attack = 161.
79. Resolved: **Script step and camera update.** The camera mode update runs first (SPEC-0026, main loop step 7), then the world tick, the regions, the projectiles, the items, the selected hero and the petpet, then the script step (step 8.1.12), then the front end (step 14) (High).
80. **Yield inside Init.** Answered in part: Init is a plain call inside the loader's completion step, not a thread (section 6, Init order), so a yield there would yield the loader. The static Init-root host-call survey finds eight wait-capable call sites across six modules (nine path records because one site has two alternatives); all pass WAIT 0. It finds no Sleep, Wait or WaitTillLeaveTrigger call under an Init root. This supports that the known Init call paths do not request a host wait, but does not rule out an unindexed indirect yield or establish runtime path execution (Medium-High). Whether any Init body yields through another mechanism remains open.
81. **Module lifetime across visits.** Resolved: every load into a buffer rewinds its pool, reads the whole pack, relinks the module, runs its load-time constructors and calls Init, so Init sees the file values apart from the words a constructor writes (in play this matters only in pack 1324, whose constructor copies flag 239 into the word that splits its Init; SPEC-0041). The data words last until that buffer is released: they survive interior trips, swaps between E and T, streaming steps whose pack is already resident and a movie with no return world, and they are reset by any driver load of E, streaming to a different exterior and a new game. Interiors and transitions are fresh on every entry (Behavior section 1.1; High).
82. **Applied walk during script hold.** Resolved: the walk freezes (its routine is not stepped), the clip already playing carries the entity along its facing until the clip's progress reaches 0.99, the entity then stands in its idle, and the walk resumes toward the same target on release (section 4, Script hold during a walk; High for the frozen routine and the idle request, Medium-High for the drift).
83. Resolved: **AddAnimObjSound slot test (row 224).** The first empty slot (cue -1) is taken; the two floats are the start and end of the sound window as fractions of the object's duration, clamped to 0.00001 and 0.99999 (High).
84. **Trigger word +0x20 and hero shots.** Resolved: +0x20 = 1 marks a fire by the hero shot pass, which was earlier taken for a per-tick contact scan; no body-contact path sets it (High).
85. **Selected hero's three lists (Medium).** Partly resolved: slot teardown empties the selected hero's 15-entry target list at +0xA70 (SPEC-0053, section 2.1) and 15-entry character-candidate list at +0xDC0 (SPEC-0040, Behavior 20), as well as a third 15-record list at +0xC18 (28-byte records). That third list is the talker list of SPEC-0022 (Candidate lists), not bookkeeping. It is filled each hero tick from the global character list, taking only characters whose townsfolk flag (character word +0xC54) is set: a character not yet listed is added to the first free record with its character id, a zeroed word, one float derived from the hero-to-character offset, a zeroed word and a flag of 1, and the owner's count (+0xDBC) is raised. It feeds the talk candidate: the hero's two target-query methods return the lock-on leader, the talker leader, or the nearer of the two (the talker leader wins only when strictly nearer), and the talk step of SPEC-0022 uses the second (High for the list and its use; the per-record refresh rules are in SPEC-0022). The teardown runs for whichever slot is released; it does not establish that the non-selected hero's lists are cleared. Example: after any slot release, the selected hero's target and candidate counts are 0 and every entry id is -1; the middle-list count is 0 and its entry ids are -1. High for these writes and the two cross-referenced roles; Medium for the broader effects.
86. **Plain pickup popup (row 190).** Resolved (High): the script popup and the auto-collect popup are the same popup object, set up by the same item-effect steps (the popup shows no item name). The popup starts in its ordinary state (state 1, with the timing words 5.0, 0.5 and 0.6) whenever the script's bool is set, whatever the item. The auto-collect path uses a different start state (the full showcase) only when the item has a text line, the pickup is not quiet and the hero is not busy: that state starts the engine's showcase task and requests a stinger that is out of the player's range, so that request is silent; the task's own pose stinger (6, 7 or 8) does play (High; SPEC-0021, Item showcase). The ordinary state plays nothing, and the count of finds is raised in both cases. Every other auto-collect starts the ordinary state. So the script popup looks like an auto-collect popup of an item with no text line. Test vector: GiveItemToHero(item 36, 1, true) shows the ordinary popup with no voice and no showcase task, even if item 36 was never found before.
87. **Drift under script hold.** The drift of a held walk (section 4, Script hold during a walk) is not confirmed on the running game (Medium-High).
88. **Computed fade and letterbox durations.** Resolved: one live FadeScreenOut call in module 678 uses 1.0 s; a second live FadeScreenOut call in module 678 uses 0.8 s with WAIT 1; a different live FadeScreenOut call in module 800 uses 0.5 s; one live FadeScreenOut call in module 722 uses 0.7 s with WAIT 1; one live FadeScreenOut call in module 893 uses 0.7 s with WAIT 1; two live FadeScreenOut calls in module 1371 each use 1.0 s with WAIT 1; four FadeScreenOut calls in module 1394 use 0.5 s with WAIT 0; and two live FadeScreenIn calls in module 800 use 0.4 s and 0.5 s (rows 136 and 137; High). One live FadeScreenOut helper call in module 1023 has a caller-selected duration: its two observed callers supply values producing 0.05 s (3 frames) and 0.03333334 s (2 frames), both with WAIT 1. This variable-duration site stays outside the fixed-duration histogram. The module 1023 call is present on the survey's trigger-17 and Init routes; that does not prove both paths run in a particular playthrough. The second module 678 call is in the script-trigger-1 routine; its survey lists the direct object, flag and module-word gates and an Init route. The module 722 call is in the control-thread routine, with the survey also listing helper/script-trigger and ungated Init routes. The module 893 call is in a control-thread routine; survey routes retain the recorded hit-point, module-word and flag gates, alongside another control-thread route and an Init route. The two module 1371 calls are in an Update helper; their survey routes preserve the listed state and flag conditions. The module 800 0.5-second FadeScreenOut call and second FadeScreenIn call are listed under control-thread helpers; their survey routes retain their recorded control-thread, state and word conditions, alongside helper/Init routes where listed. The four module 1394 FadeScreenOut sites are in the helper called by script trigger 13; the direct trigger route includes the recorded object, state and module-word checks, while the survey separately lists an ungated Init route. The 0.4-second FadeScreenIn call is reachable under script trigger 1 and from Init in the module survey; the trigger route itself retains its listed state and flag gates. One live-survey FadeScreenIn call in module 777 uses 0.5 s with WAIT 0; its route includes the listed trigger, state, module-word and game-state conditions, with an Init route also listed for the helper. This does not prove the route occurs in a particular playthrough. The remaining FadeScreenIn and LetterBoxOff sites are fixed values too. FadeScreenIn: module 678 at 0.5 s, 0.8 s (48 frames) and 0.7 s (one site each), module 722 at 0.7 s (two sites), module 893 at 0.7 s (two), module 1023 at 1.0 s with WAIT 1 (two), module 1371 at 1.0 s with WAIT 1 (two), module 1394 at 1.0 s with WAIT 1 and module 1398 at 0.5 s; the other sites listed above have WAIT 0. LetterBoxOff: module 1023 at 0 s (1 frame), module 1174 at 0.5 s and at 0.2 s (12 frames) and module 1217 at 1.0 s. In each case the duration is a constant held in a saved floating-point register of its routine and passed straight to the call, so none depends on a caller (High; the routes are on the survey with the gates listed). No fade or letterbox site on the disc takes a computed duration, apart from the caller-selected FadeScreenOut helper of module 1023 described above. The corrected histograms are in rows 137 and 139 (rows 136 to 140).
89. **Script-group kind set-ups (row 271).** Resolved: each implemented kind returns 0 or 1 under its own setup checks; kinds 0, 7 and 12 return 0. The per-kind conditions are stated in row 271. This is not a uniform success/failure contract. **[High for the observed return rules]**
90. **ItemActivate region argument (row 198).** Resolved: the first integer is a slot number of the world, checked against the created-slot count and the slot's result word. The items come from the item placements of the character region loaded into that slot; modules 1056 and 1096 pass slot 0 in play (row 198 and the note after the Category 21 table).

91. **Resolved (R-0104): Rope aim scan outputs.** Return 0 means no selection, 1 a character aim candidate, and 2 a Rope Gun anchor; the anchor pass can replace a character result. The scan stores a world-space shot aim point independently of anchor selection: selected character aim point plus half its motion height, or camera position plus 150 units along camera-forward when no character passes. The point is not the raw right-stick coordinate.
92. **Special-thread names (added 2026-11-09).** The per-tick step runs eleven special threads in the order given in Behavior section 1 (the Update thread is slot 4, High). The spec calls slots 1 and 2 the enter-interior and leave-interior sequences; the code-side description of the same slots names them only by their positions in the manager (a world callback and a further hook thread). The slot order is confirmed; the two names are not independently confirmed (Medium).
93. **Load-override word (added 2026-11-09).** The tick saves and restores the world's load-override word (world +0x57C4) around the trigger pass, and restores it during a hero swap. What the word selects for the load is not described here (open).
94. **IsHeroDead method (added 2026-11-09).** The call forwards to the hero object's own "is dead" method; that the method is the state-16 test has not been separately verified (Medium).
95. **Eye w lane of SetToNormalCamera (distance, angle) (added 2026-11-09).** The eye's w is the hero's w plus the direction length times the distance; whether the pose setter uses that lane is unknown.

## Confidence

| Section | Confidence |
| --- | --- |
| Container header fields, sizes and counts (173 modules, 163 distinct, 24,368 fixups, 5,171 import and 509 export symbol records) | High (confirmed on disc) |
| Export names and family counts (Exterior 87, Interior 50, Transition 36 packs; Exit missing in 10) | High (confirmed on disc); distinct counts Medium |
| Compressed fixup stream | Medium (not needed by the clean side) |
| Name encoding of host calls | High |
| Enum values used by the level modules (Layout, section 2.1) | High (every call site evaluated) |
| Callback information record | High for the writes and initial values; Medium for which callers run in play |
| Family selection rule (by buffer) and module lifetime | High |
| Init, Update and Exit roles, per-tick script step | High, including the flag that skips steps 2 to 5 (the hero-swap in-progress flag) and the non-current world's thread table |
| Host API catalogue, rows 1 to 294 | Per row: High, Medium or Low as marked |
| Host API catalogue, rows 295 to 388 | Per row, as marked in the table; the argument-ignoring quirks in Edge cases are Medium-Low |
| Hero-class rows 52, 53, 54, 171, 182, 192, 193, 195 | High (bodies confirmed) |
| Hero-class row 178 (revive) | High for the gate (swap mode 2, named hero dead) and the revived hero (partner of the active hero); Medium for the return value (question 19) |
| Hero-class rows 348, 349, 350, 357, 361 | Medium per row (no level-module call is known; the call walker may miss calls) |
| Camera calls (play family, StartCameraShake, StartCameraShakeMode, return to normal) | High for the play arguments, targets, end modes and speed factor, the StartCameraShakeMode order and the SetToNormalCamera overloads; Medium for StartCameraShake |
| Triggers, enter test, per-tick firing | High for the locator table, the trigger capacity and the other per-world table sizes; Medium for registration, callbacks and the enter test's per-tick overlap test; High for the state word's writer (the fire step, Layout section 3); Medium-High for the fire gates by volume kind and flag bit, and High for which paths pass object 1; Low for the firing predicate |
| Threads and waits | High for Sleep, thread slots, the 50-thread limit and the enum reading of WAIT and CTL; High for Wait (trunc(s x 60) in single precision), the control-thread table (no return value, entry freed on the tick its thread ends), the script hold of base state 33 and the frozen walk under it; Medium-High for the clip drift of a held walk; Medium for the waiting wrappers |
| Saving (no host call opens the save screen or sets a save-point flag; a running thread hides the save prompt) | High |
| MoveEffect point and direction forms, WalkEntity wait codes | Medium for the MoveEffect helper axis (Open question 40); High for the WalkEntity wait codes (classes 23 and 24, Open question 42 resolved) |
| MoveHero (row 156) and SetHeroStartPos (row 281) | High (Open question 41 resolved); Medium-High that SetHeroStartPos has no visible effect |
| Slot loading, numbering and records (section 4 and rows 205 to 216) | High for region numbers, slot numbering, the first SlotCreate returning 0 at capacity 1, the no-remap rule for -1, the wait and extra yield, the keep flag effect on the per-tick update and the cap values, SlotCreate loading nothing and SlotLoad as the only region reader, the SlotCreate size (R, c and F = 384), the release, the result value, the character setup at read completion and the item-placement spawn of the activate step; Medium-High for the module slot-word pattern; Medium for a read into a slot with no buffer |
| Entity activation rows 62, 114, 116 and 259 and the wave rule (section 5) | High |
| Host-call binding (Layout, section 1.5) | High for the direct binding and unchanged arguments; Medium-High for link-time order |
| Transitions | Medium; High for the plane index rule, the interior landing rules, the current-world gates of rows 277 to 279 and 284, the dead StartTransitionScene callers, the world-taking call-site survey, the Init order against the placement and the make-current word |
| Clean-side approach (section 7) | High for the rule, including module data words as live state and the reachability rule; the schema is a plan |
| Rows 286 to 294 | Medium per row; High for row 286 (Draik readers) and row 291 (DFS_Act1_IsTriggerHit) |
| Rows 33 and 43 (SetToNormalCamera) | High (Open question 22 resolved) |
| Load by id (section 12 of Behavior) | High for the known image-buffer path, id source, capacity failure and init order; no path reuses a loaded record (question 37); Low for the name-based form |
| Uncatalogued host names | Complete: every host record with a body now has a row (section 2) |

## Appendix: Uncatalogued host names

None. Every host name in the boot program's host table now has a catalogue row. The rows outside the first group are rows 286 to 294 (Category 15) and rows 295 to 388 (Categories 16 to 23). The nine family entry names (Init, Update and Exit) are records 397 to 405 of the host table, but they have no bound function and are not catalogue rows (Layout, section 1.2; Open question 48). Row 242 (memset) is host record 1 and is a host record like the others; it has no game-state effect.

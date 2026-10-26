# SPEC-0023: In-game HUD

- Answers: proactive, none requested (in-game HUD); R-0018 (in part: point 1 is open question 137 in SPEC-0035; points 2 to 4 answered and point 5 open, Open question 74); R-0020 (answered: open question 75 resolved, item-id child rule and flag modes)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per section. On-screen countdown timer (section 31): High for the limit and freeze rules, the clamp order, the running gate on the clamp (count-up included), the step size (the start-up rate word 1/60), the default colour and the counter element. Section 24 is a pointer: the Mote panel belongs to SPEC-0021 (High). Canvas size and bar scale: High (values from the boot program's data; the canvas is always 512 by 448). HUD screen, node fields and art chain (resource 532, screen record 9): High (every sheet asset resolves on disc). Visibility timers and pulse timing: High for the rules and the tick rate (once per front-end pass, once per game tick). Petpet meter (section 7) arithmetic and identity: High; its warning has no visible effect on disc (High). Journal-notification icon (section 8): High, including its record table (design table 2). D-pad hint icon (section 9): High. Monster health bar arithmetic, timer, fade, tick order and tuning words: High (from the boot program's data and fixed constants); which monster kinds show it: High (every region character through the take-hit, plus the boss arms of section 10.1); that no shipped monster has a zero maximum: Medium. Tint target and byte rounding (section 2): High. The whole HUD is not drawn while a dialogue box is open (section 4.1): Medium-High. Health and pool bars: fill rule, frame pieces, dividers and loss effect High; the wrap mode of a fill wider than its texture: clamp from start-up, High; the in-play wrap state Medium (HUD screen and art). Item-keyed centred scale (section 3.1; answers open question 41): arithmetic, gates, item sources and sprite identities High; exact design meaning of the percentage-scaled icon emblems Medium-Low. The health and pool bars do not use it. Low-health warning (section 32): High. Button prompts (section 33): High for the six slots and the single live slot. Tint start colour (neutral 128 per channel from boot, written by the text-window init, a start-up table entry): High. Quick-item icons and counts: High. The four icons are the four entries of the quick-item panel (SPEC-0021), with a fixed direction-to-entry mapping, and the three equipped-slot counts are value texts in the three equipment panels (section 11). HUD node draw path shared with the front end: High. Hero-switch portrait swing (section 12): High for the start rule, the constants, the geometry and the gates; Medium-High for the exact tick counts (31 increments in both directions under float32 with truncation, SPEC-0008). Countdown timer display (section 13): High. Group hide and element alpha push: High. Number captions: High, including the overflow and negative quirks; there is one number format (section 26). Overlay cell canvas margin: Medium-High. Percent gauges (arc and bar, section 16): geometry High (table and angle rule); the bar as a quad emitter with optional label Medium; the gauge caller and the input scale Low (Open questions 34 and 35). Screen-to-fixed-point conversion (section 17): High; it has no caller. Canvas (Canvas mapping): High; the canvas words are never rewritten. Ping-pong oscillator (section 18): arithmetic and bounds High (boot data values); owner High (the front end's save and load picker). HUD id pairs (section 19): table High, meaning Low. Book-load sequence and page angle (section 20): High for the book load/complete path and page-angle draw; Medium-High for the callback's resource-parsing role. Six-slot effect pool (section 21): High (the icon tweens of section 11.1). Event posts (section 22): Medium for the writes, Low for the meaning. Section 23 is a pointer: the equipped weapon and shield model loader belongs to SPEC-0021 (Medium-High). Section 25 is a pointer: the two routines once read as an element placement start and stop Tor's weapon and shield Mote glows, owned by SPEC-0046 (Medium-High). HUD icon tweens (section 11): High for the triggers and speeds, Medium for the motion. Prompt caption draw (section 30): High. Three-option cycler (section 27): High for the rule (Left steps down, Right steps up, on the rising edge); it belongs to an unused option word and never runs. Mote charge re-arm of the equipment-panel timers (section 4): High for which timer each charge use sets; Medium-Low for the intent.

## Summary

The in-game HUD is a 2D overlay drawn on a 512 by 448 canvas centred on the screen. It is one screen of the front-end UI database in resource 532 (SPEC-0035): screen record 9 (state code 0xB4, front-end state 11), whose page node is 322. Its art, layout and colours are all in that resource, and it is drawn by the same node draw path as the menus. It shows the hero's health bar and magic-pool bar, each framed from its maximum and filled from its current value (2/3 pixel per unit), with upgrade dividers, a selected-quick-item panel, three equipment panels with item counts, and a set of groups that fade out 240 ticks (4 s) after the last change. Losing health starts a red tint pulse that lasts 40 ticks, and a short screen-anchored effect at the old end of the bar. While the hero is poisoned, a green tint pulse plays whenever no other pulse is running. Every change to a health or pool value re-arms the visibility timers. While a dialogue box is open, the whole HUD is not drawn (section 4.1).

Further in-play elements: the active petpet's meter (60 pixels wide at full), a journal-notification icon that pulses after a new journal-linked progress flag is set, a D-pad hint icon shown after L3, the countdown timer and counter of section 31, the hero-switch portrait swing, a monster health bar at the bottom left for the monster hit most recently (section 10), the quick-item cross (four icons with counts, only while the quick panel is open), and six button-prompt slots of which only the Square slot is used (section 33). A low-health warning plays a sound cue only (section 32). There is no NeoPoints counter in play; the balance is shown on the item screens. The HUD has no damage numbers and no pickup notices in the shipped build, as far as is known.

Captions use a nine-character number format with thousands separators, and value-text nodes format bound game words as integers, strings or times.

The overlay also has a ping-pong oscillator that moves a value between 1.2 and 1.8 (section 18; it belongs to the front end's save and load picker, not the HUD), two logical id pairs chosen by a small switch (section 19), a one-slot effect sequence and a six-slot effect pool (sections 20 and 21), and a set of event posts (section 22). Sections 23 and 24 point to SPEC-0021, which holds the equipped weapon and shield model loader and the Mote panel, and section 25 points to SPEC-0046 for Tor's weapon and shield Mote glow. The canvas is fixed at 512 by 448 (Canvas mapping).

A percent gauge is a dial (arc) or a bar that shows a value from 0 to 100 percent. Its arc is drawn as a polyline of at most 16 steps around the circle, clockwise from 12 o'clock, using a table of sine and cosine pairs in the boot data. The same drawing routine, with a four-corner quad, serves the bar variant. A gauge can carry a percent label. Nothing in the program calls the gauge update, so no gauge is ever drawn in the shipped game (section 16).

The clean side can build the overlay from the HUD screen's node tree in resource 532 (sprites, value texts, positions and colours) plus the rules below for the bars, the pulse, the timers, the gauges and the elements.

## Where it lives on the disc

- Canvas and colour defaults are 32-bit words and bytes in the boot program's data, read at runtime by virtual address (SPEC-0001 gives the fingerprint). Values and addresses are in the Layout table. The file-backed part of the program image starts at the executable's load address (the lowest virtual address of its loadable segment, given in the ELF program header). The file offset of a virtual address is that address minus the load address, plus the segment's file offset (0x80 in this build).
- The HUD's per-frame state (timers, tint ramp, bar caches, element counters, phases, and the monster bar's target and timer) is zero or set at run time. It is not on the disc. The quick-item and flash timers, the arc-swing state and the tick-rate and frame-step words are in the same position: zero-filled at load (their addresses are beyond the file-backed part of the program), set at run time. The tick-rate words are written once at start-up from row 0 of the boot rate table at 0x30C598 (SPEC-0008, Display mode): ticks per second 0x368FA0 = 60.0 and frame step 0x368FA8 = 1/60. The tint start colour is also zero-filled at load, but the text-window init, a start-up table entry, sets it to 128.0 per channel at boot, before the first frame.
- The gauge's own record values (current value, rate, centre, radii, mode) are run-time values set by the owner of the gauge. They are not on disc.
- The monster bar's tuning values are boot-program data words, listed in the second table of the Layout section.
- The HUD is screen record 9 of the front-end database in resource 532 (state code 0xB4, front-end state 11, page node 322; SPEC-0035). Its node tree holds every HUD sprite and value text, with positions, sizes and colours, and its art is the texture chunk of the same resource (Layout, HUD screen and art). The HUD has no text labels of its own, only sprites and value-text nodes.
- The hero divider table (per-hero base and step of the upgrade dividers) is eight 32-bit words of boot data at 0x319BC0 (Layout).
- The value-text variable table (pointers to the game words that value-text nodes show) is in boot data at 0x2DD200.
- The hero's values (health, pool, quick-item slots) come from the character's run-time state. They are not HUD data.

## Layout

### Boot-program data read by the HUD

| Name | Address | Type | Value at boot | Use |
| --- | --- | --- | --- | --- |
| Canvas scale | 0x2EC900 | float | 1.0 | Vertical scale of the 2D canvas (a further 1.0 sits at 0x2EC8FC). The only writer of the canvas words is a setter that is never called, so they keep their boot values (Canvas mapping). |
| Canvas height | 0x2EC904 | int32 | 448 | Canvas height in pixels. |
| Canvas width | 0x2EC908 | int32 | 512 | Canvas width in pixels. |
| Draw offset (x, y) | 0x2EC8F4, 0x2EC8F8 | int32 pair | 0, 0 | Offset applied at the top of each HUD node draw. The item screens set it to (60, 23) while they draw the bar group 0x31 and clear it afterwards; in normal play it stays (0, 0) (section 34). |
| Pulse half-length | 0x2DCDDC | float | 20.0 | Ticks for the rise of a tint pulse, and again for its fall. |
| Red tint | 0x2DCDE8 | 4 bytes (R, G, B, A) | 200, 0, 0, 128 | Tint for health loss. |
| Green tint | 0x2DCDE0 | 4 bytes (R, G, B, A) | 0, 200, 0, 128 | Tint for poison. |
| Neutral colour | 0x2DCDD8 | 4 bytes | 128, 128, 128, 128 | Neutral colour value. The D-pad hint (section 9) uses it as its colour. The health and pool tint pulses do not read this word; they start from the run-time start colour below. |
| Bar offset table A | 0x2DCE00 | int16, entries 0 and 1 used | -9, -9 (entries 2 and 3 are 0 and not used; entries 4 and 5 are the two words of table B) | Right-cap offset, entry 0 health and entry 1 pool, in pixels (section 3). |
| Bar offset table B | 0x2DCE08 | int16 x 2 | 6, 6 (the words after them are the section 3.1 scale constants, below) | Frame inset, entry 0 health and entry 1 pool, in pixels: middle width = L - B, right cap x = L + B + A (section 3). |
| Item scale constants, rule A | 0x2DCE0C, 0x2DCE10, 0x2DCE14, 0x2DCE18 | float32 x 4 | 0.8, 1.8, 8.0, 100.0 | Section 3.1: scale at the low end, scale at the high end, value at the low clamp, value at the high clamp. |
| Item scale constants, rule B | 0x2DCE1C, 0x2DCE20, 0x2DCE24, 0x2DCE28 | float32 x 4 | 0.8, 1.8, 8.0, 100.0 | Same roles as rule A, for the second rule. The word after them, 0x2DCE2C (800.0), is not used by these routines; it is the fly-in speed of the HUD icon tweens (section 11). |
| Icon tween fly-out speed | 0x2DCED0 | float32 | 100.0 | Canvas units per second of an icon that flies out of an equipment slot (section 11). |
| Tint start colour | 0x37BF20 | 4 floats | 128.0 per channel (neutral) from boot. Zero-filled at load (beyond the file-backed part; no file bytes), then written at boot by the text-window init, which is a start-up table entry (SPEC-0026) | Start colour of a tint pulse. The text-window init (SPEC-0022, Text window) writes four floats of 128.0 here before the first frame. A run-time value. |
| Timer-flash colour start | 0x2DD040 | 4 bytes (word 0x80808080) | 128, 128, 128, 128 | Starting colour word of the two timer-flashed elements (section 13). It is also the default colour of the countdown timer's Activate, the counter's Show and the colour setters when no colour is given (section 31). |
| Quick-item icon list | 0x2DCCD0 | 4 int32 | -1, -1, -1, -1 | Item ids for the four quick-item icon elements (section 11): the four entries of the quick-item panel (SPEC-0021), written each tick while the panel is open. Entries 1 to 3 are the item index words at 0x2DCCD4, 0x2DCCD8 and 0x2DCCDC of section 3.1. |
| Arc-swing constants | 0x2DCEE8, 0x2DCEF0, 0x2DCEF4, 0x2DCEF8, 0x2DCEFC, 0x2DCF00 | float32 | 0.5, -0.5236 (-pi/6), 2.6180 (5pi/6), 0.5, 2.0, 4.0 | Section 12: swing duration in seconds, the start angle for side 0 and for side 1 in radians, the low end of the scale blend, and the hold lengths in seconds after an accepted swap (2.0) and after a refused one (4.0). |
| Gauge sine and cosine table | 0x2ECEA0 | 17 pairs of float32 (sine, then cosine), 8 bytes per pair | Pair k holds the sine and cosine of k x 2 pi / 16, for k = 0 to 16 (see section 16) | Angle table of the percent arc (section 16). |
| Label font block | 0x2ECE18 | block of words | First word 0 | Font data of the percent label (section 16). Read only by the gauge routines, which never run, so it has no visible effect (Open question 37, resolved). |
| Oscillator lower bound | 0x2DCDB4 | float32 | 1.2 | Lower bound of the ping-pong oscillator (section 18). |
| Oscillator upper bound | 0x2DCDB8 | float32 | 1.8 | Upper bound of the ping-pong oscillator (section 18). |
| Oscillator step | 0x2DCDBC | float32 | 0.02 | Added to the oscillator value on each call; its sign is reversed at each bound (section 18). |
| Oscillator value | 0x37BF14 | float32 | 0 at load (zero-filled, beyond the file-backed part); run-time value | Current value of the oscillator (section 18). |
| Book page angle start | 0x2DB2E0 | float32 | 1.2 radians | Start angle for the two-page turn animation (section 20; SPEC-0035 section 6.1). |
| Label words | 0x2ECE30, 0x2ECE34, 0x2ECE38, 0x2ECE3C, 0x2ECE40, 0x2ECE44 | float32, float32, float32, float32, float32, float32 | 1.0, 1.0, 640.0, 512.0, 640.0, 512.0 | Initial values of six words that the label draw overwrites (section 16). Read and written only by the gauge label draw, which never runs, so they keep these values and have no visible effect (Open question 37, resolved). |
| Hero divider table | 0x319BC0 | 8 int32 | health base Tor 150, Roberta 120; pool base Tor 90, Roberta 150; health step Tor 25, Roberta 20; pool step Tor 15, Roberta 25 (in that order) | Upgrade dividers of the two bars (section 3). Hero = the active hero's index (game object word +0x14C: 0 Tor, 1 Roberta). |
| Value-text variable table | 0x2DD200 | table of pointers | pointers to game words | Value-text nodes (kind 6) pick their bound word by index into this table (Layout, HUD screen and art). |
| Journal pulse step | 0x2DCF18 | float32 | 0.15 | Phase step per tick of the journal-notification icon (section 8). |
| D-pad hint alpha words | 0x2DCF1C, 0x2DCF20 | int32 x 2 | 128, 32 | Full and floor alpha of the D-pad hint ramp (section 9). |
| Hero available words | 0x2DCF08, 0x2DCF0C | int32 x 2 | 1, 1 | Hero 0 and hero 1 available; 0 shows that hero's switch portrait dimmed (section 12). |
| Petpet warning latch | 0x2DCF10 | int32 | run-time latch | Keeps the last petpet-meter warning state (section 7.2). |
| Ticks per second | 0x368FA0 | float32 | 0 at load (zero-filled); 60.0 from start-up | Written once at start-up from the rate table (SPEC-0008). Used by the monster bar timer (section 10) and the portrait swing (section 12). |
| Frame step | 0x368FA8 | float32 | 0 at load (zero-filled); 1/60 (0.016666668) from start-up | Written once at start-up as 1 / ticks per second. Step of the countdown timer and of the counter duration (section 31). |
| Rate table | 0x30C598 | 4 float32 | 60.0, 30.0, 50.0, 25.0 | Row 0 (60.0, 30.0) is always used; the second row is the PAL row, which is unreachable (SPEC-0008). |
| Prompt caption colour | 0x315A38 | 4 bytes (R, G, B, A) | 160, 160, 90, 127 | Default colour of the button-prompt captions (section 30). |
| Prompt caption positions | 0x315A78 (x), 0x315A88 (y) | 6 16-bit values each | x 207, 207, 327, 327, 327, 327; y 395, 418, 395, 418, 395, 418 | Caption cell of each of the six prompt slots (section 30). |

Monster health bar tuning words (boot-program data, read at boot; section 10):

| Name | Address | Type | Value at boot | Use |
| --- | --- | --- | --- | --- |
| Frame divisor | 0x2DD01C | float32 | 1.5 | Frame width = maximum health / this value, plus 1, truncated. |
| Frame maximum | 0x2DD020 | int32 | 250 | Upper clamp on the frame width in pixels. |
| Frame minimum | 0x2DD024 | int32 | 64 | Lower clamp on the frame width in pixels. |
| Name margin | 0x2DD028 | int32 | 24 | The frame is at least the name width plus this value (section 10.3). It takes effect for names wider than 40 px. |
| Piece offset A | 0x2DD02C | int32 | -5 | Frame right cap x = frame middle x + frame width + this value (section 10.5). |
| Piece offset B | 0x2DD030 | int32 | -5 | Name plate right piece x = name plate centre piece x + name width + this value (section 10.5). |

Fixed constants, not boot data (values as used by the HUD): 240 ticks (visibility timers), 30 ticks (fade length), 0.6666667 (float32) px per unit (bar scale, frame and fill), 1/512 and 1/448 (loss-effect point normalisation, section 3), 4.266667 (alpha per fade tick, 256 / 60), 60 (full width of the petpet meter in pixels), 1792 and 1824 (canvas origin for the overlay cell margin, Canvas mapping), 9 (width of a number caption, section 15), 128 (full alpha byte), 45 and 14 (warning thresholds of section 7), 64 (journal icon pulse, section 8), 0.033333335 (one thirtieth, D-pad hint ramp, section 9), 0.15 (low-health threshold, section 32), 14 (dividers per bar), 600 ticks (monster bar timer after an arm, trunc(ticks per second x 10.0)), 60 ticks (monster bar timer cut at zero health, trunc(ticks per second)), 10 px (name width used when the monster name is empty), 1.0 (fill scale, full), 31 characters (monster name length), 2048 (screen centre, Screen-to-fixed-point conversion), 16 (units per pixel in the fixed-point screen format), 16 steps per full circle (percent arc, section 16).

### Percent gauge record (run-time, one per gauge)

| Offset | Type | Meaning |
| --- | --- | --- |
| +0x00 | float | Current value, in percent. Starts at 0. |
| +0x04 | float | Rate. The value is multiplied by the input (see section 16). |
| +0x08 | float | Centre x (for the bar, the left end). |
| +0x0C | float | Centre y. |
| +0x10 | u32 | Colour word. |
| +0x14 | float | Horizontal radius (for the bar, the length of 100 percent). |
| +0x18 | float | Vertical radius (for the bar, the half height). |
| +0x1C | integer | Mode: 1 = arc; 3 = arc with label; 0 = bar; 2 = bar with label. Read as an integer bit pattern. |
| +0x20 | 8 bytes | Fill state word (run-time). |
| +0x28 | 8 bytes | Label state word (run-time; used when the label is on). |
| +0x30, +0x34 | float | Label position words (run-time). |

### HUD elements

| Element | Count | Size or source | Remarks |
| --- | --- | --- | --- |
| Health bar group | 1 | Inside group 0x31. Frame middle 0x07, right cap 0x06, fill 0x01, plus a left cap, a gloss strip and 14 dividers per hero | Frame from the maximum, fill from the current value (section 3). |
| Pool bar group | 1 | Inside group 0x31. Frame middle 0x09, right cap 0x08, fill 0x02, same other pieces | Shown only while the pool maximum is above 0. |
| Monster health bar | 1 | Group 0x24 at the bottom left: name plate 0x21 (centre) and 0x22 (right), frame middle 0x1F, right cap 0x20, fill 0x23 | Shows the most recently hit monster (section 10). |
| Selected quick item panel | 1 | Group 0x30: ring, item icon 0xA0, count text 0x15, effect icons | Section 11. |
| Equipment panels | 3 | Groups 0x1C, 0x1D and 0x1E, each with a ring, a hero-specific icon and a count text | Equipped-slot counts (section 11). |
| Quick-item cross icon | 4 | Element ids 0x9C (right), 0x9B (up), 0x9E (left) and 0x9D (down) | Entries 0, 1, 2 and 3 of the four-entry list (section 11). Shown only while the quick panel is open. |
| Tint element | 1 | Element 0x3B (node 649): the health frame group, holding the left cap, the right cap 0x06 and the frame middle 0x07 (nodes 650 to 652) | Its colour word is replaced each tick while a pulse runs (section 2). |
| Petpet meter | 1 | Element id 0x12. Full width 60 px | Section 7. |
| Petpet portraits | 5 | Element ids 0x3C to 0x3F and 0x03 | The warning of section 7.2 targets them; it has no visible effect. |
| Fade groups | 5 | Element ids 0x31 (bars), 0x30 (item panel), 0x1C, 0x1D and 0x1E (equipment panels) | Each has its own 240-tick timer (section 4). |
| D-pad hint icon | 1 | Element id 0xBB | Section 9. |
| Countdown timer texts | 2 | Element ids 0xD8 (timer) and 0xD9 (counter) | Section 13. |
| Group-hide elements | 3 | Element ids 0x31, 0x30 and 0x0C | Set to alpha 0 by the post-load reset (section 14). |
| Journal-notification icon | 1 | Element id 0xDF | Pulsing alpha (section 8). |
| Hero-switch portraits | 2 | Element ids 0x2A (Tor) and 0x2B (Roberta) around pivot 0x29 | Section 12. |
| Button-prompt slots | 6 | L1, L2, R1, R2, Square and Cross slots | Only the Square slot is live (sections 30 and 33). |
| Reticle | 1 | Element id 0x0C | Its group predicate is always false, so it is never drawn. |
| Item-screen extras | 2 | Element 0xBF (NeoPoints balance) and 0xBE (equipped-item icons) | Drawn only on the item screens (section 34). |
| Percent gauge (arc or bar) | 1 or more | One gauge record (section 16) | Arc or bar, with or without a label. Never drawn: the gauge update has no caller (section 16). |

Element ids are the section C keys of the front-end database (Layout, HUD screen and art); the clean side finds each element's node through that map. It may give the elements its own names, as long as the rules below apply to the same elements.

Hero values read each tick (logical names): health current, health maximum, pool current, pool maximum, and for each equipped quick-item slot (slots 0, 1 and 2) the item id (-1 when empty) and the owned count of that item. The count is 0 when the slot is -1 or the item is not valid.

### Canvas mapping

- Canvas pixel (x, y) is measured from the top-left corner of the 512 by 448 canvas, with y increasing downward.
- The canvas is centred on the screen point (2048, 2048), in the screen's pixel space. Its top-left corner sits 256 pixels left of and 224 pixels above that centre.
- In the screen's fixed-point units (12.4 format) each pixel is 16 units. The conversion of a screen coordinate to these units is in section 17.
- Sprite scale is 4.12 fixed point: a stored value of 4096 means 1.0.
- Menu overlay cells (SPEC-0035 section 15) are drawn only when their canvas position satisfies -10 < x < 522 and -10 < y < 458. That is a 10-pixel margin outside the 512 by 448 canvas. The projection subtracts the canvas origin values 1792 and 1824 (the screen-space origin above: 2048 minus 256, and 2048 minus 224). Confidence: Medium-High.
- The HUD's node tree is drawn by the same node draw dispatcher as the front-end node tree (SPEC-0035, section 12). Node kinds, visibility conditions, draw-offset values and the canvas rule are shared, so the HUD's layout is not changed by that path. Confidence: High for the shared dispatcher.
- The canvas is always 512 by 448 with scale 1.0, for the HUD and for every front-end page. The canvas words (vertical scale 0x2EC900 = 1.0, height 0x2EC904 = 448, width 0x2EC908 = 512, and a further 1.0 at 0x2EC8FC) have exactly one writer, a four-word setter, and that setter is never called: no call reaches it and no pointer to it exists in the program. Confidence: High.
- Canvas to the screen's fixed-point units: x_screen = (x + 2048 - 512 / 2) x 16 and y_screen = (((y + 2048 - 448 / 2) - 2048) x s + 2048) x 16, with the vertical scale s = 1.0. The vertical scale acts about the screen centre; the horizontal axis has no scale. Confidence: High.
- The two routines that pass 512 and 448 together with 32, 32 or 256, 256 are not canvas set-ups. They are the screen-effect object's init (SPEC-0052): width, height, letterbox bar heights of 32 or 256, and a platform word. Confidence: High.
- The record that holds height 448, or 512, is the video-mode record of the display set-up and belongs to SPEC-0008 (Display mode). Its 512-high branch needs the PAL flag, which is never set because the region detector always reports NTSC in this build. So no 512-high canvas or screen occurs. Confidence: High.
- **Canvas setup helpers (Medium).** The screen/canvas initialiser prepares a four-corner quad in 1/16-unit coordinates with extent 128; a flag adds an 8-unit nudge. The meaning of that flag and the nudge's purpose are not known. The same initialiser calls a clip-rectangle helper that converts four object fields to 1/64 units, truncating toward zero, and writes clip value 16,380. The exact source fields, the conditions for its 0 or -1 result, and what the clip value limits remain open. These are screen/canvas setup details, not camera behavior.

### HUD screen and art (resource 532)

Confidence: High (every HUD element id below resolves to a node of the HUD tree on disc, and all 244 sheet assets resolve to the texture chunk).

- The HUD is section D record 9 of the front-end database (SPEC-0035): state code 0xB4, front-end state 11, page node 322. Element ids in this spec are section C keys; section C maps each key to a section J node. Examples (key -> node): 0x01 -> 617 (health fill), 0x02 -> 654 (pool fill), 0x06 -> 651, 0x07 -> 652, 0x08 -> 656, 0x09 -> 657, 0x31 -> 615 (bars group), 0x3B -> 649, 0x12 -> 578 (petpet meter fill). The clean side looks each element up through section C at run time rather than hard-coding node numbers.
- Art chain of a sprite node (kind 1): node -> its section E sprite record -> the asset index at E +0x08 -> the section O record. For a sheet asset (kind word 2), bytes +0x00 to +0x07 of the O record are a GS texture register value (TEX0): base block, buffer width 8, pixel format 8-bit or 4-bit indexed, texture width and height exponents, and palette base block. The texture is the sub-entry of the resource-532 texture chunk whose base block and pixel format equal the asset's; the palette is the 32-bit sub-entry whose base block equals the palette base. The chunk's pixel block is uploaded at GS block 0x2A00 (256 by 511 pixels, 32-bit), and decoding follows SPEC-0003. All 244 sheet asset records on disc resolve this way (207 at 8 bits, 37 at 4 bits).
- A sprite is a quad from the node's local (0, 0) to (width, height), after the node transform, with width at J +0x22 and height at J +0x20 (signed 16-bit). The texture is sampled one texel per pixel from the top-left of the sprite's frame cell, so a sprite wider than its art shows more texture; it is not stretched. Colour = the E record's colour bytes (R, G, B, A at E +0x04), modulating the texture (128 = 1.0).
- Texture wrap of a sprite wider or taller than its art. The sprite draw sends only the texture register (TEX0) with each quad. It sets no wrap mode: it draws in GS context 1 and uses the context-1 wrap register (CLAMP) as it stands. Start-up sets that register to clamp on both axes, and no front-end or HUD draw writes it afterwards. The only later writers are world materials with a second texture context (500 of the 15,592 material groups on disc, in 121 resources; of their 1,000 context-1 wrap words, 882 are repeat on both axes and 118 are horizontal clamp with vertical repeat) and the pass-1 texture animator (region repeat). So on screens shown before any world is drawn (card check, logo, attract, title) a wide sprite clamps: every column past the art's right edge shows the last texel column. In play the mode is whatever the last such material drawn earlier in the frame left, most often repeat. The clean side should clamp on both axes (the start-up value); the 64-texel bar strips are almost uniform across, so either mode looks close. Confidence: High for the missing wrap write and the start-up value; Medium for the in-play state, which depends on draw order.
- Node fields used for placement: x = J +0x14 and y = J +0x16 (signed, relative to the parent). Alignment J +0x2E: 0 x is the left edge, 1 x is the centre (x minus width x own scale / 2), 2 x is the right edge. Row spacing: the parent's byte J +0x2A times n is added to y, where n is 0 for the parent's first child and otherwise the number of visible siblings from the second child up to and including this node. J +0x2D nonzero means absolute positioning (the parent's transform is discarded). After translation, the node's scale (J +0x18, +0x1A, 4.12) multiplies everything drawn under it, about the node's own origin. J +0x1C and +0x1E are an authoring pivot and are not used for placement. J +0x30 nonzero hides the node. Mode J +0x2C: 0 always drawn, 2 drawn while its predicate (J +0x24, argument J +0x2F) is true, 3 drawn while it is false.
- Kind 6 is a value-text node. Its section K record holds a variable index (K +0x08) into the boot table at 0x2DD200 (pointers to game words), a font id (K +0x0A) and a format (K +0x0F): 0 string, 1 integer, 3 time (M:SS:CC, or HH:MM when K +0x10 is 2). The integer format is plain decimal with no padding.

### Normal-play layout (canvas pixels, top-left origin, no draw offset)

Confidence: High (laid out from the disc tree with the node rules above). The clean side reads these from resource 532; the values are given as a check.

- Health bar: bar origin (71, 19). Left cap at (69, 19), 16 by 20. Frame middle 0x07 at (80, 19), (L - 6) by 20. Right cap 0x06 at (68 + L, 19), 16 by 20. Fill 0x01 at (76, 24), F by 9, colour (135, 30, 30). Gloss strip at (81, 26), 32 by 4. Dividers 5 by 6 at y 26. L and F are given in section 3.
- Pool bar: the same pieces 17 px lower (frame y 36, fill at (76, 41), gloss at (81, 43), dividers at y 43), with frame middle 0x09, right cap 0x08 and fill 0x02, colour (30, 30, 200). The fill sprites' disc alpha (100) is replaced by the fade alpha (128 when fully visible).
- Selected quick item panel 0x30: ring at (15, 21), about 57.6 px; item icon 0xA0 at about (22, 25), 41.6 px (absolute); count text 0x15 centred at (60, 72).
- Equipment panels 0x1C, 0x1D and 0x1E at y = 106, 166 and 226: ring at (434, y), hero icon at (461, y), count text centred at (451, y + 60).
- Petpet panel (while a petpet is active): portrait 32 by 32 at (78, 59); meter frame from (108, 56) to the right cap at (176, 56); meter fill 0x12 at (114, 61), height 9, colour (200, 10, 10).
- Monster bar 0x24: name plate at y 387 (left piece at x 60, centre piece 0x21 at x 77), frame at y 408 (left cap at x 51, middle 0x1F at x 62), fill 0x23 at (58, 413), height 9, colour (135, 30, 30).
- Hero-switch portraits around pivot 0x29 at (437, 17), 56 by 64. D-pad hint 0xBB, 128 by 128 at (16, 304). Journal icon 0xDF, 36 by 36 at (99, 349). Timer text 0xD8 right-aligned at x 436, y 77; counter text 0xD9 right-aligned at x 437, y 77. Save spinner 25 by 30 at (256, 371).
- Quick-item cross (only while the quick panel is open): rings about 57.6 px at right (135, 174), up (85, 114), left (35, 174) and down (89, 234); count texts centred at (178, 226), (129, 169), (80, 226) and (134, 286).

### HUD record defaults (record of 0x2A0 bytes; Open question 62)

Confidence: High for the stores; Medium for the meaning of the fields. Offsets are from the start of the record. The base defaults are applied first; the record init then overwrites some of them.

| Offset | Size | Value | Meaning |
| --- | --- | --- | --- |
| +0x004 | 1 | 160 | Byte; not changed by the record init |
| +0x005 | 1 | 160 | Colour byte 1 |
| +0x006 | 1 | 90 | Colour byte 2 |
| +0x007 | 1 | 127 | Colour byte 3 |
| +0x008 | 4 | 0 | Default (role not established) |
| +0x00C | 4 | -1 | Default (sentinel) |
| +0x010 | 4 | 3 | Default (role not established) |
| +0x014 | 4 | 10.0 (float) | Default (role not established) |
| +0x018 | 4 | 0 | Default (role not established) |
| +0x01C | 4 | 0 | Default (role not established) |
| +0x020 | 1 | 0 | Default (role not established) |
| +0x220 | 4 | 0 | Default (role not established) |
| +0x224 | 4 | 0 | Default (role not established) |
| +0x228 | 4 | 512 | Default (role not established) |
| +0x22C | 4 | 0 | Default (role not established) |
| +0x000 | 4 | 0 | Set last by the record init |
| +0x230 to +0x24C | 8 words | two 16-byte vectors | Copied from boot data at fixed places (values not given here) |
| +0x250 to +0x260 | 5 words | 0 | Cleared |
| +0x270 | 4 | 0 | Cleared |
| +0x274 | 1 | 255 | Set by the record init |
| +0x278 | 4 | -2 | State word |
| +0x27C to +0x288 | 4 words | 0 | Cleared |
| +0x294 | 4 | 0 | Cleared (last offset written; the record is at least 0x298 bytes) |

Confidence for the state word and the byte at +0x274: High. The field names are not established.

## Behavior

Time is in game ticks. The HUD screen (front-end state 11) runs once per front-end pass, which is once per game tick (60 ticks per second; SPEC-0007, SPEC-0035 Behavior 1).

### 1. Per-tick order

The bar update (steps 1 to 9) runs on an in-play tick only while the change-detector timer C is not 0 (section 4). It does, in this order:

1. Compare the four watched values (health current, health maximum, pool current, pool maximum) with the values stored last time. If any differ, set C and the timers G0 and G1 to 240 (section 4).
2. If the pool current fell, start the pool loss effect (section 3).
3. Store the four values for the next comparison.
4. If the health current fell, store it, start the red pulse with the red tint and a half-length of 20 ticks, and start the health loss effect (section 3).
5. Otherwise, if the poison status is active and no pulse is running, start the green pulse.
6. If a pulse is running, advance it (section 2) and set the tint element's colour from it.
7. Lay out the health bar: frame and fill from its maximum and current value (section 3), and its dividers.
8. If the pool maximum is at least 1, lay out the pool bar the same way.
9. Push alpha(C) to the subtree of group 0x31 (section 4), then lower C by 1 if it is above 0.

Steps 1 and 9 together mean that a change in this tick is counted once: C shows 239 after the update that re-armed it.

After the bar update, the in-play state runs the five group fades (section 4), the petpet meter (section 7), the journal icon (section 8), the D-pad hint (section 9), the quick-item cross (section 11), the portrait swing (section 12), the countdown timer texts (section 13) and, after them, the monster health bar (section 10), under the gates of section 4.

All of these, and the on-screen countdown timer of section 31, step only on passes when the in-play front-end state runs (SPEC-0026, main loop step 14). They do not step under any other page, so they freeze under game-state code 9 (the pause page, the item screens, the map and the other freezing screens of SPEC-0026) and also while a dialogue box is up, although the world keeps running during a dialogue box (Medium-High). While a dialogue box is up the HUD is also not drawn (section 4.1).

Draw order and the one-pass lag (Medium-High). In each front-end pass the current page's node tree is drawn first, and the state handler (here the in-play state) runs afterwards (SPEC-0035 Behavior 1b). So every change the in-play state makes to node widths, positions, colours or alpha reaches the screen on the next pass. Caption texts that the handler draws itself (the prompt captions of section 30 and the price caption) appear in the same pass.

### 2. Tint pulse

- A pulse has a half-length D (20 ticks), a colour target, and a ramp value t that starts at 0.
- Each tick while a pulse runs, t increases by 1/D. When t reaches 1, the direction reverses and t is held at 1. When t falls to 0 on the way down, the pulse stops.
- Total length is about 2D = 40 ticks, with the peak at tick 20 from the start.
- The displayed colour is the start colour plus t times (target minus start), per channel. Each channel is rounded half away from zero to a whole byte (add 0.5 and truncate, for the non-negative values that occur). The start colour is neutral, 128 per channel, from boot (Layout, tint start row). So at t = 0 the tint is neutral (no tint), and at t = 1 it is the target.
- Target (High). The colour goes to element 0x3B, the health bar's frame group: the left cap, the right cap 0x06 and the frame middle 0x07. Each tick while a pulse runs, the pulse colour replaces the whole colour word (R, G, B and A) of these three sprites. Sprite colour modulates the texture with 128 = 1.0, so the tint acts as a per-channel multiplier of the frame art. The fill, the gloss strip, the dividers and the whole pool bar keep their own colours.
- After the last pulse tick the ramp is 0, so the frame is left at the start colour (128 per channel), which equals its colour on disc.
- Starting a new pulse (for example a second health loss during a pulse) restarts the ramp from 0 with the new target. The start call also advances the ramp once in the same tick (Medium).
- Red has priority over green in the same tick. Green never starts while a pulse runs.

### 3. Bars

Confidence: High (fill, frame, dividers and loss effect); Low for the drawing of a negative width.

- Scale K = 0.6666667 (float32). For each bar, L = trunc(maximum x K) and F = trunc(current x K): the float32 product, truncated toward zero. The product itself is a single-precision result truncated toward zero, and the conversion to an integer truncates (SPEC-0008, Platform arithmetic). Since K is stored slightly above 2/3, the product of a whole value never falls below its exact value, so the widths are the same under either rounding of the product. Both bars use the same scale. Example: health maximum 150 gives L = 100; pool maximum 90 gives L = 60.
- Frame: the middle piece is L - B wide and the right cap sits at x = L + B + A, relative to the bar origin, with A = -9 (table A, 0x2DCE00) and B = 6 (table B, 0x2DCE08) for both bars. So the middle is L - 6 wide and the right cap is at x = L - 3. The left cap and the gloss strip do not move.
- Fill: the fill sprite (0x01 health, 0x02 pool) gets width F each time the bar update runs. There is no animation, no clamp and no rounding: the width is set directly. A current value above the maximum gives a fill longer than the frame.
- The pool bar is laid out only when the pool maximum is at least 1, and drawn only while the pool maximum is above 0 (section 4).
- Upgrade dividers: each bar has 14 divider sprites per hero (only the active hero's set is drawn). Divider k (0 to 13) is shown when k < (maximum - base) / step, with integer division truncating toward zero, using the active hero's base and step from the hero divider table at 0x319BC0 (Layout): health base Tor 150, Roberta 120, step Tor 25, Roberta 20; pool base Tor 90, Roberta 150, step Tor 15, Roberta 25. A maximum at or below the base shows no divider. The divider x positions come from the HUD tree.
- Hero switch: the bars, dividers and panels read the selected hero every tick, so they show the incoming hero from the tick it is selected (SPEC-0053).
- Loss effect: when the health (pool) current falls and that bar's effect slot is idle, the right end point of the fill as drawn on the previous tick (fill x + width, fill y + height / 2) is divided by (512, 448), turned into a world point in front of the camera, and effect 0xFA2 (pool: 0xFA3) is started there. Each frame the point is projected again so that the effect stays on the bar. A third slot holds effect 0xFA1, with no HUD use. The effects themselves belong to SPEC-0034.

#### 3.1 Item-keyed centred scale (identified; answers open question 41)

Confidence: arithmetic, gates and sprite identities High; the design meaning of the percentage-scaled icons Medium-Low. The scaled nodes are quick-item effect emblems using the Health_Icon or Magic_Add_Icon art (see the element list below). This rule is not a health or pool fill; the bars use the rule of section 3.

Two variants share one form. Rule A reads the item's pool-A percentage and rule B its pool-B percentage. Both are signed 16-bit fields of the item record, at record offset 0x28 (rule A) and 0x2A (rule B). If the item's type code (signed 16-bit at record offset 0x0E) is 1 to 6 or 17, the value is 0.

Rule, for a value v:

1. If v is not above 0, the element is left unchanged.
2. Otherwise clamp v to the range 8 to 100, and compute the scale s = 0.8 + (clamp(v) - 8) x (1.8 - 0.8) / (100 - 8). The four constants are the float words of the Layout table for the rule in use.
3. Store the scale as 4.12 fixed point, trunc(s x 4096). The range is 3276 to 7372. Both axes get the same scale.
4. Centre the scaled element on its unscaled centre. The x offset is trunc((W / 2) x (1 - s)) and the y offset is trunc((H / 2) x (1 - s)), where W is the element's width and H its height in pixels. There is one offset per axis for the whole element. It is not applied to each end of a bar.

Element fields written (node offsets, section J; Medium): x offset at +0x14 (from the width at +0x22), y offset at +0x16 (from the height at +0x20), x scale at +0x18, y scale at +0x1A. These are the node's position and scale fields of the Layout section.

Which elements (High for the ids, item sources, art and tree ownership; Medium-Low for design intent):

- Outer HUD gate (the block runs when the outer gate returns zero): rule A on elements 0x16 and 0x27; rule B on element 0x28. The item id is the active hero's effective quick-item selection marker (SPEC-0021, Quick-item selection marker, rule 2): the marker's id while the party holds 1 or more of it, otherwise -1, and then nothing is written (High). Test: the marker holds an item whose pool-A percent is 54 and the party holds 2: s = 0.8 + 46 x 1.0 / 92 = 1.3, both scales trunc(1.3 x 4096) = 5324 on elements 0x16 and 0x27. With the count at 0, both elements keep their old values.
- In-play gate (the gate returns its flag only when the menu word is zero): rule A on element 0x26 with the item index in boot word 0x2DCCD4; rule B on element 0x25 with the index in boot word 0x2DCCDC; rule A on element 0xBC and rule B on element 0xBD, both with the index in boot word 0x2DCCD8.
- The three index words are entries 1, 2 and 3 of the four-entry quick-item list at 0x2DCCD0 (section 11): -1 in the file image and written each tick while the quick panel is open. While an entry is -1 its lookup fails and nothing is written.
- The art identifies the scaled nodes as item-effect emblems: 0x16 and 0x27 use Health_Icon, while 0x28 uses Magic_Add_Icon in the selected-item panel; 0x26 and 0xBD use Health_Icon, while 0x25 and 0xBC use Magic_Add_Icon in the quick-item cross. The two panel trees place them beside or over the corresponding quick-item visuals. Rule A therefore scales the health icon art and rule B the magic-add icon art. The exact gameplay meaning behind the item-flag modes and the design reason for scaling these emblems remain uncertain.

Test vectors (synthetic, from the stored constants):

- v = 5 is clamped to 8, s = 0.8, scale 3276.
- v = 54 gives s = 1.3, scale 5324.
- v = 100 gives s = 1.8, scale 7372.
- v = 0 leaves the element unchanged.
- An element 64 px wide at scale 1.3 has x offset trunc(32 x (1 - 1.3)) = trunc(-9.6) = -9.

### 4. Visibility timers, fade and hiding

Confidence: High.

Timers (integers; 240 = re-armed): C, the change detector; G0, the bars group 0x31; G1, the selected quick item panel 0x30; G2, G3 and G4, the equipment panels 0x1C, 0x1D and 0x1E; D, the D-pad hint (section 9). All are 0 at program start. The four watched-value caches of the bar update start at -1.

Each in-play tick (front-end state 11):

1. If C is not 0, run the bar update (Behavior 1). It ends by pushing alpha(C) to the subtree of 0x31 and lowering C by 1 if it is above 0.
2. For (0x30, G1), (0x1C, G2), (0x1D, G3), (0x1E, G4) and (0x31, G0), in that order: if the timer is above 0, push alpha(timer) to that element's subtree, then lower the timer by 1.

alpha(t) = 128 when t < 0 or t >= 30; otherwise trunc(t x 4.266667). The push sets the alpha byte of every sprite and text under the element. So after the last re-arm a group stays at full alpha for 210 ticks, fades over the last 30 ticks, and is hidden when its timer reaches 0: 240 ticks (4 s) in all.

Draw gates (node predicates): group 0x31 is drawn only while G0 is not 0; its bar group only while C is not 0; the pool bar only while C is not 0 and the pool maximum is above 0. Group 0x30 is drawn only while G1 is not 0, and 0x1D only while G3 is not 0. 0x1C is drawn only while G2 is not 0 and the hero has an unconsumed item of category 1; 0x1E only while G4 is not 0 and the hero has one of category 2.

Re-arms (set to 240):
- Any hero health or pool change through the character accessors (increase, decrease, reaching zero, a pool maximum change; SPEC-0040): C.
- The bar update, on any difference in the four watched values: G0, G1 and C.
- Equipping slot 0, 1 or 2: G3, G4 or G2.
- Spending Mote charge (SPEC-0021, Mote charge): each use sets the timer of the equipment panel of the equipment word whose Mote it spends to 240, the same timer as equipping that slot: word 2 (weapon Mote) G2, word 1 (shield Mote) G4, word 0 (armour Mote) G3. So a landed swing, a release, a quick-shot hit, a shield block or an armour-Mote hit that uses charge shows the matching panel again for 4 s, under that panel's draw gate. C, G0 and G1 are not set by it. (High for which timer each use sets; Medium-Low for the design intent, a charge display.)
- L3 pressed: C, G0 to G4 and D.
- A hero switch made with the extra flag (the L1+R1 chord, a hero-down switch and the Potion of Revival; SPEC-0053) sets all six of C and G0 to G4 at the end of the swap. D is not set.
- Selecting, using or holding a quick item: G0, G1 and C. While the quick panel is open, G0, G1 and C are held at 240.
- The low-health warning: C only (section 32). C alone does not keep the bars drawn, since group 0x31 also needs G0.

In normal play the in-play state does not re-arm any timer on its own. Only the item-screen tab states (front-end states 8, 9 and 10) set G0, G1 and C to 240 on every tick (section 34).

#### 4.1 Hiding

- Menus and other screens are separate front-end states, so the HUD tree is not drawn while they run.
- Opening the pause page or the item screen sets G0 to G4, C and D to 0, and clears the prompt flags, the countdown counter and the monster bar timer. The HUD stays hidden on return until the next re-arm. The FrontendHUDReset host call does the same (one level module calls it).
- After a load, a full reset runs: all timers 0 and C = -1, so the bar update runs every tick at full alpha and C does not count down until it is re-armed; groups 0x31, 0x30 and 0x0C get alpha 0 (section 14).
- The monster bar, the D-pad hint and the journal icon are hidden while the letterbox or the screen fade is active or animating (SPEC-0046), and while the quick panel is open.
- Dialogue hides the whole HUD (Medium-High). While a dialogue box is open, the in-play state hands the front end to the box host state (SPEC-0035, table B state 23, record 19). The box host's page (node 924) holds only the box, its name and its prompt icons, and the front end draws only the current screen's page. So the whole HUD (bars, panels, monster bar, portraits and timer texts) is not drawn while a box is up. The box host also raises letterbox bars over 10 ticks on entry and lowers them when the box closes. The HUD timers do not step meanwhile (section 1), so on return every group resumes at the alpha it had.
- A scripted cutscene that shows no dialogue box does not hide the bars. It hides only the monster bar, the D-pad hint and the journal icon, through the screen-effect gate above. Outside these cases the bars fade 240 ticks after the last re-arm.

### 5. Quick-item slots

- Three equipped slots, in order 0, 1, 2, held in the hero record at offsets +0x1544, +0x1548 and +0x154C (SPEC-0053, section 2.1). Each count is the owned count of the item in that slot, read each tick; it is 0 when the slot is -1 or the item is not valid.
- The counts are value-text nodes in the three equipment panels: slot 0 in panel 0x1D (armour icon for Tor, robe for Roberta), slot 1 in panel 0x1E (shield, magic shield) and slot 2 in panel 0x1C (weapon, wand). Each count text is shown while its count is above 0 (section 11).
- The four-entry quick-item panel is a separate control, specified in SPEC-0021; its four entries are the quick-item cross of section 11.

### 6. Map icons

The map icon list is not part of the in-game HUD overlay. It belongs to the map screen, which is a front-end page (it is not drawn from the HUD's own element records). The list holds up to 32 icons. Each icon record holds a position pair and a type or index. The icon positions and the map scale are not fully described.

### 7. Petpet meter (element 0x12)

Confidence: arithmetic and identity High.

- The meter shows the active petpet's meter (SPEC-0047): A = the petpet's current meter value and B = its meter maximum (SPEC-0047 calls them score A and score B). It runs in play, once per tick, while a petpet is active; the petpet panel shows the portrait group whose predicate argument matches the active petpet's kind index.
- Width in pixels W = round half away from zero (A x (60 / B)). Full width is 60 px. The fill node's x scale (about 1.11) widens it on screen.
- A maximum of zero (Medium-High): B is set together with A when the petpet is fed or restored, and A only counts down from it, so B = 0 occurs only with A = 0. Then W = 0 and the fill is hidden. A rewrite should treat B = 0 as W = 0 rather than divide by zero. Test: A = 0, B = 0 gives W = 0, fill alpha 0.
- Visibility: if W is 1 or more, the fill is shown at alpha 128. If W is below 1, the fill gets alpha 0.

#### 7.2 Warning

- When A is below 45.0 and W is 14 or less, the warning starts the node tracks of class mask 4 on the five petpet portraits (0x3C to 0x3F and 0x03); otherwise it stops them. A latch (boot data word 0x2DCF10) keeps the last state, so the start and stop run only when the state changes.
- No visible effect (High). A node track plays only when a start call's mask has the bit of the track's class. Resource 532 has 20 node tracks: 18 of class 0 and 2 of class 4, on portraits 0x3C and 0x3E. Mask 4 selects class 2, which no track has. The class-4 tracks would need mask 16, which nothing uses (the only masks used are 3, every page on entry; 4, this warning; and 8, the focused item). So the two class-4 tracks never play, and the warning changes nothing on screen. Reproduce it as a no-op, or omit it.

### 8. Journal-notification icon (element 0xDF)

Confidence: High.

- Setting a progress flag for the first time (SPEC-0022), when that flag has a journal record, sets the journal-notification flag and resets the phase to 0.
- The journal record table is design table 2 of resource 533 (SPEC-0024 Table 2, progress-flag cues), loaded at start-up by the design-table loader. It is not built at run time. A flag has a journal record when its row's word +0 (journal kind) is nonzero. Opening the quest journal (SPEC-0035, section 14.3) clears the flag.
- Each tick while the flag is set: phase = phase + 0.15 (boot data 0x2DCF18). Let y = -|phase| / (2 pi) and t = |(round(y) - y) - 0.5| - 0.25. The alpha byte = round half away from zero (64 x (1 + P(t))), where P is the same truncated sine series as SPEC-0034 pulse kind 3 (constants 6.283185, 41.34167, 81.60223, 76.57496 and 39.71066).
- Quirk: t reaches 0.75, outside the range where the series is accurate, so near the bottom of each cycle the alpha runs 25, 18, 15, 14, 17 instead of falling to 0.
- The icon is hidden while the letterbox or screen fade is busy and while the quick panel is open (section 4.1).

### 9. D-pad hint icon (element 0xBB)

Confidence: High.

- D is the hint's timer. L3 (pressed) sets D = 240, together with the other HUD timers (section 4). Each tick D is lowered by 1 if it is above 0.
- Alpha = 128 when D >= 30; otherwise round half away from zero (D x (128 - 32) x 0.033333335 + 32), with 128 and 32 from boot data 0x2DCF1C and 0x2DCF20. The colour (128, 128, 128, alpha) is set on the icon's subtree each tick.
- So the hint is at full alpha for 210 ticks after L3 and then ramps down to alpha 32 at D = 0.
- D is set to 0 when the pause page or item screen opens. The icon is hidden while the letterbox or screen fade is busy and while the quick panel is open.

### 10. Monster health bar

Confidence: High (arithmetic, timer, fade, tick order, tuning words and the set of users); Medium that no shipped monster has a maximum health of 0.

One bar shows one monster at a time: the monster that was armed most recently. Its target is replaced when another monster is armed.

10.1 Who arms the bar
- Every non-hero character in a world region is the same kind of game object. Its take-hit step (SPEC-0029, SPEC-0038 Behavior 37) arms the bar on every hit it receives, before damage is applied. Two cases skip it: the target's dead flag is set, or the target is in base state 16 (death). So every common enemy and every boss that receives the hero's hits through the take-hit shows the bar: melee swings, shots, Mote objects and petpet hits. The hit does not have to deal damage; a hit-locked target still arms the bar.
- These bosses also arm the bar from their own hit code, at the point where their own damage rule applies (each is the same arm step):
  - Black Knight: in the four enraged stun states (charge, high swing, floor strike, sword spin), after a part loses HP.
  - Cloud Beast: on the first tick of the special-reply state.
  - Undead Draik: hit reactions 1, 3 and 5.
  - Faerie Sisters: the dodge reaction, when the hit does not take the sister below 0 (Medium-High).
  - Gelert Assassin: when its hit detection applies damage, and on its flinch.
  - Gnarfas: its own hit detection.
  - Ixi Chieftain: incoming hits.
  - Juppie Monster: slam, stun and dazed hits.
  - Plague Serpent: the bite and mesmerize state.
  - Spyder nest: a valve in valve state 5. The bar then shows the valve, with the valve's own name, health and maximum.
- The name is the target's spawn-record name (SPEC-0029 4c). A record with an empty name shows no name plate.
- No roster row has a maximum health of 0: the smallest maximum is 5 (a boss row), and common enemies have at least 16. So the divide by zero of the Edge cases does not occur on disc (Medium: spawn health is the row health, SPEC-0029 Scaling).

10.2 Arm and tick
- Arm: target = the monster; name = its first 31 characters; timer T = trunc(tps x 10.0) = 600, where tps is the ticks-per-second word (60.0, Layout); then the layout of 10.3 to 10.5 runs once. Arming again restarts T at 600 and may change the target.
- Each in-play tick, after the countdown timer texts, and only while T is not 0: T = T - 1, then the layout runs. The layout ends with the zero-health cut: if the target's current health is 0 and float(T) is above tps, T = trunc(tps) = 60. Then alpha(T) of section 4 is pushed to group 0x24.
- So the bar is at alpha 128 while T is 30 or more, fades as trunc(T x 4.266667) for T = 29 down to 1, and is hidden when T reaches 0. The draw predicate is: T not 0, the quick panel closed and the screen effect idle.
- The take-hit arms before it applies damage, so a killing hit through the take-hit first arms the bar with the health before the hit. The cut does not happen at that arm; it happens in the layout of the next in-play tick, after the kill. A boss that arms after its own damage can be cut at the arm itself, since the arm's own layout already sees zero health.
- T does not step while the in-play state is not running: under pause and every other page, and while a dialogue box is open. Opening the pause page or the item screen, the HUD reset, and every world load set T to 0.

10.3 Frame width, in pixels
- Let M be the monster's maximum health. Compute M divided by 1.5 (the tuning word in Layout), add 1.0, and truncate to a whole number.
- Limit that result to the range 64 to 250 (the frame minimum and maximum words in Layout).
- Then raise it to at least N + 24, where N is the measured width of the monster's name in the name node's font (10 when the name is empty) and 24 is the name margin word at 0x2DD028. This final value is the frame width used below.
- Example: M = 150 gives 101 px. M = 60 gives 64 px (41 before the limit). M = 600 gives 250 px (401 before the limit). M = 60 with a name 60 px wide gives 84 px.

10.4 Fill width, in pixels
- Let C be the monster's current health and B the frame width. Compute v = truncate(C x (B / M) + 1.0), in float32. B / M is computed first, then multiplied by C, then 1.0 is added. With the platform arithmetic of SPEC-0008 (a divide rounds to nearest; a multiply or add result is truncated toward zero), a product whose exact value is a whole number can land just below it, so the fill is then 1 px less than exact arithmetic gives (see the test vectors).
- The fill width is the smaller of v and the frame width.
- The +1.0 means a monster at zero health still shows a 1-pixel fill while the bar is visible (until the timer runs out, 60 ticks after the health reached zero if the timer was longer).

10.5 Pieces (High)
- The bar group 0x24 sits at the bottom left of the canvas (Layout, normal-play layout).
- Frame: a left cap, the middle 0x1F with the frame width B, and the right cap 0x20 at x = middle x + B - 5 (piece offset A, 0x2DD02C).
- Name plate: a left piece, the centre piece 0x21 with the name width N, and the right piece 0x22 at x = centre x + N - 5 (piece offset B, 0x2DD030).
- The fill 0x23 is drawn with its x scale reset to 1.0 and the fill width of 10.4.
- The bar is hidden while the letterbox or screen fade is busy and while the quick panel is open (section 4.1), and its timer is cleared when the pause page or item screen opens.

10.6 Name panel
- The name panel is drawn only when the first character of the name buffer is not zero.

10.7 Reads
- The monster's current and maximum health are read through the same character accessors the hero HUD uses.

### 11. Quick-item cross, counts and equipped icons

Confidence: High.

- The quick-item cross has four icon elements, one per D-pad direction, each showing one entry of the four-entry quick-item list at 0x2DCCD0 (the quick-item panel of SPEC-0021): right 0x9C = entry 0, up 0x9B = entry 1, left 0x9E = entry 2, down 0x9D = entry 3. The list is -1 in the file image and is written each tick while the panel is open. Each icon is shown when its entry's count is above 0.
- The cross, its rings and its four count texts are shown only while the quick panel is open. The panel's candidate lists, cycling, selection, marker and use are specified in SPEC-0021.
- Item icons are found from the item id: the icon is the child at zero-based index equal to the item id, in the item-icon node of the HUD tree (the node reached by section C key 0x9F; the disc gives it 160 children). The art is in resource 532 like every other HUD sprite. Ids with no child draw nothing, and the element keeps its previous art.
- The selected quick item panel 0x30 shows the selected item's icon (0xA0), its count (0x15) and effect icons for what the item restores (health, magic, cure or both; SPEC-0021).
- The three equipped-slot counts are value texts in the equipment panels: slot 0 -> panel 0x1D, slot 1 -> panel 0x1E, slot 2 -> panel 0x1C (section 5). Each count text is shown while its count is above 0.
- Element 0xBE is not a count. Its children hold the item-screen equipped-item icons (categories 1, 5 and 2, the last for hero 0 only), with alpha 0 when the slot is empty. It is drawn only on the item screens (section 34).

11.1 Icon tweens (High for the triggers and the speeds; High for the motion, the colours and the step)
- The HUD has a pool of six icon tweens. A tween moves an icon node from a start point to an end point at a fixed speed in canvas units per second. Starting a tween on a node that already has one cancels the old one first.
- Fly-in speed: 800 canvas units per second (0x2DCE2C). Fly-out speed: 100 canvas units per second (0x2DCED0).
- Quick-item marker: when the quick-item panel (SPEC-0021) moves its marker to a chosen entry, that item's icon flies in at 800 from the chosen cross icon (0x9C, 0x9B, 0x9E or 0x9D) to the selected-item icon element 0xA1.
- Mote equip from the Mote panel (SPEC-0021): the new Mote's icon flies in at 800 from its panel cell to the slot's equipment icon. When the slot held a Mote with a count above 0 before the press (it is replaced or cleared), the old icon flies out from the slot by a fixed start-up offset at 100. The equipment icons are refreshed after every equip or clear.
- These tweens are not the hero-switch portrait swing (section 12).
- Tween set-up (High). The start point and end point are the canvas positions of two nodes: each node's x and y plus those of its parents, up to and including the first node with the absolute flag. The duration is T = distance / speed seconds (straight-line distance). Each tween also has a start colour and an end colour (R, G, B, A, 128 = 1.0), and the colour moves linearly from the first to the second over T.
- Tween step (High). On each tick of the in-play state and of the item screen's Charms tab (front-end state 9, where the Mote picker runs), every live tween adds speed x frame step (the frame-step word, 1/60) to its position along the line, adds the colour change for one frame step, and adds the frame step to its elapsed time. When the elapsed time reaches T, the position and the colour are set exactly to the end values and the tween ends after that tick. The position is rounded half away from zero and written as the moved node's x and y. Each colour channel is limited to 0 to 255 for R, G and B and to 0 to 128 for A, rounded half away from zero, and pushed to the node and everything under it.
- Colours and offset (High; start-up values). Quick-item fly-in: start (128, 128, 128, 0), end (128, 128, 128, 128), so the icon fades in while it flies. Mote fly-in: start and end (128, 128, 128, 128), no fade. Mote fly-out: the end point is the slot icon's position plus (0, -30), so the old icon rises 30 canvas units (y grows downward), in 30 / 100 = 0.3 s; its colour runs from (128, 128, 128, 200) to (128, 128, 128, 0). The alpha limit of 128 keeps it at full alpha for the first 0.108 s (until 200 - 200 t / 0.3 falls below 128), then it fades to 0 at 0.3 s.

### 12. Hero-switch portrait swing

Confidence: High for the rules, the constants, the geometry and the gates; Medium-High for the exact tick counts (float32 accumulation with the truncating arithmetic of SPEC-0008, Platform arithmetic).

The swing is the hero-switch portrait animation: the Tor portrait group 0x2A and the Roberta portrait group 0x2B move along an ellipse around the pivot element 0x29.

Start.
- Every hero-swap request (SPEC-0053 3.2) starts the swing with three inputs: side s = 1 when Tor is the selected hero (going to Roberta) and 0 when Roberta is (going to Tor); accepted (the request passed the swap-mode lock); blocked (the target hero is dead). A forced request (the script form) is always accepted and never blocked.
- Available words (boot words 0x2DCF08 for Tor and 0x2DCF0C for Roberta, both 1 in the file image): the word of hero (1 - s), the current hero, is set to 1. The word of hero s, the target (Roberta when s = 1), is set to 1 when the request is accepted and not blocked, otherwise to 0. A portrait whose available word is 0 is drawn dimmed (colour 20, 20, 20); so after a refused request the target's portrait shows dimmed. A dead icon is drawn while that hero slot's death test is true, that is while that hero's action state is 16, the death state (SPEC-0053).
- The active flag is set only when the in-play controls gate passes (SPEC-0053 section 3.13: the hero is not in state 25, the second hero test is false, and the game-state words +0x20 and +0x5C are both 0).
- Accepted and not blocked: angle a = the start angle of side s (side 0: -0.5236, side 1: 2.617993; boot words 0x2DCEF0 and 0x2DCEF4); end = a + 3.141593; angle step = ((end - a) / 0.5) / tps per tick (pi / 30 at 60 ticks per second). Blend b starts at 0.5 and rises for side 0, or starts at 1.0 and falls for side 1, with a step of ((1 - 0.5) / 0.5) / tps per tick (+1/60 for side 0, -1/60 for side 1). Hold H = 2.0 x tps = 120 ticks.
- Refused (swap mode 3 or 5, the lock mode) or blocked: no new swing; H = 4.0 x tps = 240 ticks. So a refused swap does not move the portraits; it only dims the target and shows the group for 4 s.

Tick (in-play state; it does not run during a dialogue box).
- While the angle step is not 0, place the portraits (below). Then, if a equals end, set the step to 0. Otherwise add the steps to a and b, limit a to at most end and b to at most 1.0 (b has no lower limit).
- Increment count: in both directions the angle equals the end only after 31 increments (float32 with the truncating arithmetic of SPEC-0008), so the portraits are placed on 32 ticks, the last one at the end angle, and the step then becomes 0. Under round-to-nearest arithmetic side 0 would need 30 increments and side 1 still 31. Side 0's blend reaches the 1.0 limit. Side 1's blend has no lower limit and ends at 0.48333 (0.483332 with truncation), so the last placement has Tor at scale word 1980 and Roberta at 4164 (1.016668), not at 0.5 and 1.0.
- While the step is 0 and H is above 0: H = H - 1. When H reaches 0, the active flag clears.
- Draw gate: the group is drawn while the active flag is set and the SPEC-0053 swap mode (game-state word +8) is 2. In swap modes 0, 1, 3, 4 and 5 the portraits are not drawn.

Placement.
- R = half the pivot element 0x29's width, truncated (56 gives 28). (Px, Py) = the pivot element's authoring pivot fields (J +0x1C and +0x1E: 28, 32).
- Tor's group 0x2A is at angle a with scale b. Roberta's group 0x2B is at angle a + 3.141593 with scale b' = 0.5 + (1 - (b - 0.5) / 0.5) x 0.5, which is 1.5 - b.
- For a group with its own pivot (qx, qy) (Tor 28, 29; Roberta 29, 32), angle t and scale k: x = round(Px + 0.8 x R x cos t - qx x k) and y = round(Py + R x sin t - qy x k), where round goes half away from zero. Both are written to the group's position, relative to the parent at canvas (437, 17). The x and y scale are both round(k x 4096) (4.12). The horizontal radius is 0.8 R, so the path is an ellipse.
- cos and sin are the truncated series of SPEC-0034 pulse kind 3: one cosine function c, built on a 9th-order odd polynomial for sin(2 pi x) after a range fold, gives cos t = c(t) and sin t = c(t - pi/2). The clean side may use exact cos and sin; at R = 28 the difference is below one pixel (Medium).
- End state: the incoming hero is at 150 degrees (left of and below the pivot) with scale 1.0, and the outgoing hero is at -30 degrees (right of and above the pivot) with scale 0.5.

### 13. Countdown timer display (elements 0xD8 and 0xD9)

Confidence: High.

- Elements 0xD8 and 0xD9 display the on-screen countdown timer of section 31. 0xD8 is drawn while the timer is visible and shows its value as M:SS:CC: hundredths h = round half away from zero (value x 100), taken as unsigned and clamped to 59 999, then M = h / 6000, SS = (h / 100) mod 60, CC = h mod 100. A negative value therefore shows 9:59:99.
- 0xD9 is drawn while the secondary element of section 31 is visible and its stored duration is not 0, and shows its counter value as a plain integer.
- Each tick the stored timer colour and counter colour (set by Activate, SetTimerColor and Show) are written to the two elements' colour words. Both colour words start as neutral 128 per channel (word 0x80808080, Layout).
- The counter is cleared when the pause page or item screen opens (section 4.1).

### 14. Element alpha push and group hide

Confidence: High.

- An alpha value given to an element is passed down its tree: it sets (does not multiply) the alpha byte of every sprite and text under the element. An alpha of 0 hides the element and everything under it.
- The full reset that runs after a load pushes alpha 0 to three elements (ids 0x31, 0x30 and 0x0C), together with the timer reset of section 4.1 (all timers 0, C = -1). They stay at alpha 0 until a re-arm makes the fade push a new alpha.
- The visibility rule of section 4 sets the group alphas from their timers. This hide gives the three elements alpha 0 directly.

### 15. Number captions (fixed width)

Confidence: High.

- There is one number formatter for captions. It writes an integer into a text field of a fixed width of N cells. The price caption of SPEC-0035 and the NeoPoints balance use N = 9.
- Digits are right-aligned. A comma goes before each group of three digits, counted from the right.
- Leading zeros are blanked: their cells are spaces. The last digit is always written, so the value 0 shows a single 0 at the right.
- Cells to the left of the number are spaces.
- Exact rule. Cell i (0 to N - 1, left to right) has rank r = N - 1 - i. A cell with r mod 4 = 3 is a separator cell: ',' when a nonzero digit has already been written to its left, otherwise a space. Every other cell is a digit cell. The digit cells take the decimal digits of the value from the highest power down, starting at p = 10^(N - 1 - trunc(N / 4)). In each digit cell: d = trunc(v / p), v = v - d x p, p = p / 10 (integer division toward zero). If d is not 0, the cell is the character with code 48 + d, and digits have now started. If d is 0, the cell is '0' when digits have started or when it is the last cell, otherwise a space.
- N = 9 has 7 digit cells, so the largest value shown correctly is 9,999,999.
- Overflow (quirk, reproduce): a value of 10,000,000 or more gives a first digit d of 10 or more, and that cell holds the character with code 48 + d, which is not a digit. 12,345,678 shows "<,345,678".
- Negative values (quirk): division truncates toward zero, so every nonzero d is negative and its cell holds a character code below 48. For example -5 in 9 cells is eight spaces and then '+'. On disc the formatter is used only for the shop price and the NeoPoints balance, which are never negative (Medium).
- Users: the buy-price caption (SPEC-0050) and the NeoPoints balance on the item screens (section 34).
- Value-text nodes are separate: format 1 (counts, the counter) is plain decimal with no padding and no separators, and format 3 is the M:SS:CC timer of section 13.

### 16. Percent gauges (arc and bar)

Confidence: High that no gauge is drawn in the shipped game (the update has no caller). Arc geometry and angle table High (from the boot data); bar and quad emitter Medium; label Medium. The rules below describe the dormant routine only.

Reach (High). The gauge update is not called from anywhere in the boot program: no call, jump, stored routine address or address load names it, and the arc and bar draws are reached only from it. The label font block and label words are used only by these routines. So no arc gauge, bar gauge or percent label appears on any screen, and the clean side need not draw one. A gauge holds a percent value p from 0 to 100 (Layout, gauge record).

Update. Given an input k, the value advances by k x rate: new value = old value + k x rate (the record's rate at +0x04). With no caller, the input has no source and its scale has no effect. When the old value is 0, the track is drawn first (the full arc, or the full bar, from 0 to 100 percent). Then the fill is drawn from the old value to the new value, with the label when the mode has one. Then the new value is stored. Input percents are clamped to 100 before the arc or bar is drawn.

Mode. Mode 1 is an arc, mode 3 is an arc with label, mode 0 is a bar, and mode 2 is a bar with label. Any other mode stores the new value and draws nothing (High). The bar routine has its own label path, reading the same font block (Medium-High).

Arc angle. A percent p maps to the angle theta = p x 2 pi / 100, measured clockwise from 12 o'clock. A point at theta is (cx + Rx sin theta, cy - Ry cos theta), where (cx, cy) is the centre, Rx the horizontal radius and Ry the vertical radius. Percent 0 is the top point, 25 is the right point, 50 is the bottom point, and 75 is the left point (screen y increases downward).

Arc polyline. The arc is drawn with at most 16 steps around the full circle, one step per 6.25 percent. The points of the arc from p1 to p2 are: the point at p1; the point at each multiple of 6.25 percent strictly between p1 and p2; and the point at p2. For each multiple of 6.25 percent the sine and cosine come from the boot-data table of 17 pairs at 0x2ECEA0 (Layout). Pair k is the sine and cosine of k x 2 pi / 16, so the table index for a sample is floor(p x 16 / 100). The table is 17 entries of 8 bytes each (two float32 values per entry, sine then cosine, at boot data 0x2ECEA0); entry 16 repeats entry 0. The stored values are rounded to four decimal places (the largest difference from the exact sine or cosine is about 2e-5). The clean side should compute the exact sine and cosine rather than reproduce the table. If p2 - p1 is not more than 0.0625 percent, only the two end points are drawn.

Bar. A bar from p1 to p2 is a band from x = cx + p1 Rx / 100 to x = cx + p2 Rx / 100, and from y = cy - Ry to y = cy + Ry, in the colour word (+0x10). Rx is the length of 100 percent. This band is drawn as one quad from four corners around the centre point, with a half-width and half-height. The four corners are written as vertices with the same colour and a flag (Medium; the quad emitter is the same routine as the bar, so the two are one rule).

Label. When the label is on, the integer part of the percent is formatted as the integer followed by a percent sign, and drawn at the arc end point (for the bar, at its end). The label uses the font block at boot data 0x2ECE18 and the label words at 0x2ECE30 to 0x2ECE44 (Layout). The label words are overwritten by each label draw, so their initial values are not a constant of the rule. The label position is the record's label position words (run-time).

### 17. Screen-to-fixed-point conversion (12.4)

Confidence: High for the rule; it has no caller in the boot program.

The routine takes a rectangle (corner x, y, width w, height h) in pixels, four texture coordinates, and a size pair (Sx, Sy) in pixels. It writes two corner vertices of a sprite in the 12.4 fixed-point screen format:

1. For each axis, the origin is 2048 minus half the size: Ox = 2048 - Sx / 2 and Oy = 2048 - Sy / 2. With the 512 by 448 canvas as the size pair, the origins are 1792 and 1824, the canvas origin of Canvas mapping.
2. The first corner is (x + Ox) x 16 and (y + Oy) x 16; the second is (x + w + Ox) x 16 and (y + h + Oy) x 16. The origin is added in pixels before the multiply by 16.
3. Each value, and each texture coordinate times 16, is rounded to the nearest integer with halves going away from zero: for a positive value add one half and truncate; for a negative value subtract one half and truncate.
4. Both vertices get the fixed depth 0xFFFFFE.

No routine of the boot program calls it and no data word points to it, so no HUD element is placed through it. The live HUD draw uses the canvas origin of Canvas mapping. The rounding is the same rule used for the 4.12 positions of the arc swing (section 12).

Worked vectors (from the rule): with Sx = 512, x = 10.03 gives (10.03 + 1792) x 16 = 28832.48, which rounds to 28832. With Sy = 448, y = -10.03 gives (1813.97) x 16 = 29023.52, which rounds to 29024. With a size of 0, x = 0.03125 gives 2048.03125 x 16 = 32768.5, which rounds to 32769 (a tie goes away from zero).

### 18. Ping-pong oscillator

Confidence: arithmetic and bounds High (boot data at the Layout addresses); owner High.

- The oscillator keeps one run-time float value v. Its step and two bounds are boot-program data words.
- Each call adds the step to v. Then, if v is above the upper bound (1.8) or below the lower bound (1.2), v is set to the bound it passed and the step's sign is reversed.
- So v sweeps between 1.2 and 1.8 in steps of 0.02, reversing at each bound; one sweep is about 30 calls (0.6 divided by 0.02). The exact count depends on float rounding.
- Owner (High): it is not a HUD element. It is the highlight pulse of the front end's save and load slot picker (SPEC-0035). The picker calls it once at the end of each of its passes, which is once per front-end pass while that picker is open.
- Use (Medium-High): the focused slot row's colour is multiplied by v on all four channels, then limited to 0 to 255 for R, G and B, with alpha held at 128. So the focused row pulses brighter and back.
- Reset: each time the picker cursor moves to another slot (Up or Down accepted), v is set to the upper bound 1.8. The sweep then starts downward on the next call.
- Start: v is zero in memory before the picker first moves its cursor. From zero, the first call sets v to 1.2 and makes the step negative; the next call clamps back to 1.2 and makes the step positive. This quirk shows only until the first cursor move (Medium-High).

Test vectors: v = 1.8 after a cursor move, step +0.02 as stored: the next call gives 1.82, which is above 1.8, so v = 1.8 and the step becomes -0.02; the call after gives 1.78. A row colour of (100, 200, 50, 128) at v = 1.5 shows (150, 255, 75, 128).

### 19. Darkest Faerie orb visual pairs (cross-reference SPEC-0054)

Confidence: High for the first dispatch's orb flight/impact role and per-kind mapping; Low for the separate second routine and the meaning of its sentinel argument.

- A small effect record's integer at +0x34 selects a pair of logical effect ids. The mapping is: 1 gives (0x643, 0x3DF); 2 gives (0x645, 0x3E1); 3 gives (0x648, 0x3DC); 4 gives (0x644, 0x3E0); 5 gives (0x646, 0x3DD); 6 gives (0x647, 0x3DB); any other value gives (0x84F, 0x19C).
- For the six elemental orb kinds, the first dispatch selects the orb's flight effect and impact effect in that order (SPEC-0054, Sounds and effects / Orb visuals). These are projectile visuals, not HUD icons, text, or the Square prompt.
- The pair is passed with the effect record's node at +0x40 and a separate argument of -1. The visible role of that argument is not established.
- A second routine uses id 0x84F, or 0x45F when the integer at +0x30 is not 2, with 0x3DE as the second id, and the same dispatch helper. Its owning visual and full caller set remain open.
- This is an effect-resource cross-reference; keep the pairings above when reproducing the orb visuals. See SPEC-0054 for the boss attack behavior.

### 20. Book-load sequence and page angle

Confidence: High for the load/complete state and the page-angle draw; Medium-High for the callback's resource parsing role.

- The sequence record holds a busy flag (+0x240), a requested book id (+0x238), a completion flag (+0x23C), a lifecycle gate (+0x274), and a page angle (+0x278). The 1.2 start value belongs to the page angle, not the lifecycle gate (Layout).
- The completion query is true only when the requested id matches the stored id and the completion flag is set.
- Starting is accepted only when the record is idle. A request for the already-completed id is left alone; starting while busy does nothing. A new request sets the lifecycle gate to 0, clears completion, sets busy, stores the id and sets the page angle to 1.2 radians, then starts an asynchronous book-resource operation with a completion callback.
- The callback parses the loaded book data into its page records when the lifecycle gate still permits it, then clears busy and sets completion. If the book has been closed/reset during the operation, it runs the reset path instead.
- While the book is complete, the frame-loop page driver draws the two pages using the page angle and lowers that angle once per frame-loop pass. It clamps the angle to 0.12 radians. The step is (1.2 - 0.12) / 75 = 0.0144 radians per pass, so the transition takes 75 passes at 60 passes per second (1.25 s). This is the book page-turn animation described in SPEC-0035 section 6.1, not the HUD's arc-swing or timer-flash elements.

(Open question 45 resolved for the shipped book path; the behavior of other possible callers of this helper is not established.)

### 21. Six-slot effect pool

Confidence: High. This pool is the six icon tweens of section 11.1.

- The pool has six slots of 0x60 bytes each. Each slot has an active flag (word at +0x240 of the slot) and a key (word at +0x50 of the slot).
- Allocation: the first slot whose active flag is clear is taken. Its flag is set to 1 and the slot is set up as a fly-in or a fly-out tween (section 11.1); the two allocators differ only in that set-up. When all six slots are active, an error is logged and no tween starts.
- Release by key: the first slot that is active and whose key equals the argument has its active flag cleared. The key is not cleared. If no slot matches, nothing happens.

### 22. Event posts

Confidence: writes Medium; task-system mapping High; record meaning Low.

- Several one-line routines start by getting a record (its owner is still open, Open question 47). Four set the word at +0xD04 to 1. Two set the word at +0xD14 to 1. Each queues a task with selector 3 on the shared global task system (SPEC-0026, Main loop, global task step).
- One routine first runs a lookup. If it returns nothing, no field changes and no task is queued. If it succeeds, the routine clears +0xD00, sets +0xD04 to 1, stores the lookup output at +0xD10, and queues the task.
- The shared task system runs these selector-3 entries from the main-loop task step. The callbacks are distinct per caller; this is deferred work, not evidence of a HUD draw or a single common event handler. The owner of the returned record, the lookup's meaning, and the individual callbacks' full user-facing effects remain open.
- Getter routines for the same record read +0xD00, +0xD04, +0xD08, +0xD14, +0xD18 and +0xD1C (High).

### 23. Equipped weapon and shield models (not HUD; see SPEC-0021)

Confidence: Medium-High. This is not a HUD element and not a prompt. It is the hero's equipped weapon and shield model loader, which runs each hero update tick and picks the held models from the equipped items. SPEC-0021 (equipment models) holds its rules. Nothing in the HUD depends on it.

### 24. Mote panel (see SPEC-0021)

Confidence: High. This is the Mote panel of SPEC-0021 (the second quick panel). SPEC-0021 holds its cursor table, directions, label, assign rules and timer re-arms. In the HUD tree the panel is group node 392, drawn while the Mote panel is open, with its label text at canvas (30, 404); Mote k's icon shows while it is owned, and its highlight shows while the selection is k.

### 25. Tor's weapon and shield Mote glow (not HUD; see SPEC-0046)

Confidence: Medium-High. The draw itself belongs to SPEC-0046. This is not a HUD element and not a placement.

- The two routines once listed here start Tor's weapon Mote glow and shield Mote glow. They run when a Mote is put into or taken out of equipment word 2 (weapon) or word 1 (shield), and only while the active hero is hero 0 (Tor).
- Inputs: the Mote index m = Mote id - 160, and the record index = weapon id - 98 (weapon glow) or shield id - 106 (shield glow). A negative Mote index (no Mote) stops the glow.
- The glow uses registry model 299 + weapon index (weapon) or 307 + shield index (shield). All thirteen models 299 to 311 are registered at boot from the boot item stream (resource 9; 13 models in the 283 to 322 id range), so the glow always has a model to draw (High).
- Colour: row m of the 4-byte RGB rows at 0x2DCA60, one row per Mote id from 160 to 173; the two Motes of each element pair share a colour. Alpha: the byte at 0x2DCA58 (64).
- Test vector: Fire Mote (160) in Tor's weapon word with weapon id 98 starts glow model 299 with colour (255, 0, 0) and alpha 64; clearing the word stops it.

### 26. No second number format (High)

- The HUD has one caption number format, the formatter of section 15. There is no zero-padded integer format. The only float-to-text routine in the program is a debug print that nothing calls (no call and no pointer to it exists), so it never shows on screen. The clean side needs neither.

### 27. Three-option cycler (High for the rule; not used in the shipped game)

- The cycler is the front-end handler of the options block word +0x0C (SPEC-0035, the unlabelled three-value option, default 1). It reads the rising-edge menu word, so one press makes one step.
- Order in one pass: first Left (mask 0x8000, D-pad or stick Left): v becomes 2 if v is below 1, else v - 1. Then Right (mask 0x2000): v becomes (v + 1) mod 3. With both pressed in one pass the two steps cancel.
- No table entry, focus item or pointer reaches the handler, so the shipped game never runs it, and the word keeps its default.

### 28. Six-channel update

Confidence: Medium-Low for channel/source semantics; Medium-High for the observed call rate and gate.

The object holds six channels in a row, starting at offset 0x600 of the object and 0x1C0 bytes apart. In the main-loop active-hero branch, the updater is called once for the selected hero during that pass, after the hero's regular update and before the global task step. The source file has one direct call site in the inspected boot program. The branch requires the selected hero record and its character/model state to be present and active; this does not establish an update on every frozen screen or outside that branch. Each update uses the game tick length, 1/60 second (SPEC-0007).

For each channel in order 0 to 5:
1. If a source lookup finds no source, or the channel's own check fails, the channel is reset to value 0.
2. Otherwise the source record is copied, a scalar is derived from one component of the source's vector multiplied by the tick length (1/60 second), and the channel is updated with the record and the scalar.

### 29. Placing a 2D element by canvas pixel

Confidence: Medium. The normalising factors are High (fixed constants).

1. Given a canvas position (x, y) in pixels, measured from the top-left of the 512 by 448 canvas, first check the element's slot. If it is busy, nothing is done.
2. Otherwise store the normalised position u = x / 512 and v = y / 448 in two float fields of the element record. The original multiplies by the reciprocals; the result is the same to rounding.
3. The position helper builds a transform from the normalized pair. The translation is centered: (u, v) = (0.5, 0.5) adds no translation, and offsets from that center shift along the element's existing transform basis, scaled using the owning element's dimensions and display terms. The normalized pair is therefore not itself a final pixel coordinate; final placement depends on the parent transform. The helper then writes a zero vector through a separate element setter. What that vector controls remains unknown (Open question 59; High for the centered placement rule, Low for the zero-vector field).

Examples: (256, 224) gives (0.5, 0.5); (512, 448) gives (1, 1); (0, 0) gives (0, 0).

### 30. Prompt caption draw

Confidence: High.

This is the per-slot caption draw of the button prompts. Section 33 gives the slots, their writers and when they show; this section gives the draw of one caption.

- Positions. The six slots have fixed caption cells at x (207, 207, 327, 327, 327, 327) and y (395, 418, 395, 418, 395, 418), for slots 0 to 5 (boot data 0x315A78 and 0x315A88). The Square slot (slot 4) is at (327, 395).
- Colour. The caption uses the colour given, or the default caption colour (160, 160, 90, 127) at 0x315A38 when none is given.
- Font and alignment (High). The six prompt captions are drawn with font 1 of the shared font table, the same font that the dialogue box uses for its prompt label (SPEC-0022); the font number is not a HUD record field. The two words at +0x208 and +0x20C of the HUD record are draw-queue context words that the text draw passes through, not font ids. The caption is left-aligned: its cell x is the left edge of the first glyph (alignment mode 0, no centring and no right alignment), and its cell y is the baseline of the line (the glyph quads lie above it). Every text draw then raises an x below 16 to 16, which does not affect the cells above. The price captions of the buy screen use font 0 with right alignment and are a separate draw (SPEC-0050).
- Shown counter. Each slot has its own shown counter. A caption is skipped when its counter is 0 or less. Otherwise its text is drawn at the slot's cell (depth -1.0), the number the text draw returns is added to a running word of the HUD record (+0x20C), and the counter is lowered by 1, even when nothing was drawn.
- The chooser and the other writers of section 33 set the Square counter to 1 (0 to clear) on every tick that an interaction applies. So the caption shows on exactly the ticks it is set. The slot's icon in the HUD tree is drawn while the counter is not 0; since the tree draw comes before the in-play state's caption draw in the same pass (section 1), the icon and the caption show on the same pass.
- Caption and icon are both skipped while the quick panel or the Mote panel is open.

### 31. On-screen countdown timer (High)

A countdown timer shows a time value in seconds. Level scripts use it for timed hero swaps (SPEC-0039 and SPEC-0041). It has these fields: a visible flag, a running flag, a value (a decimal number of seconds), a limit (a decimal number), a direction (0 counts down; any other value counts up) and a colour.

Host calls:

- Activate(start, limit, direction, colour): value = start; visible on; running on; limit and direction stored. The colour is the one given, or the default colour when none is given: the word at 0x2DD040, (128, 128, 128, 128). The colour setters and the counter's Show use the same default (High).
- Deactivate: visible off. Running and the value are kept.
- Start and Stop: running on and running off. Visibility and the value are kept.
- GetTimer: returns the value.
- SetTimerColor: sets the colour.

Limit rule: a limit of 0 or more is a floor when counting down and a cap when counting up. A negative limit means there is no limit.

Per tick, only while visible:

1. If running, step by the frame-step word at 0x368FA8, which start-up sets to 1 / ticks per second = 1/60 (float32 0.016666668; High). Counting down: value = value - step. Counting up: value = value + step.
2. If running and the limit is 0 or more, clamp: counting down, a value below the limit is set to the limit; counting up, a value above the limit is set to the limit.

The clamp runs only on ticks when the timer is running: a stopped timer is neither stepped nor clamped. Activate does not clamp, so a start value beyond the limit is kept until the first running tick. Counting up with a limit of 0 pins the value at 0.0 from the first running tick (High for the clamp order and the running gate).

While hidden the update does nothing, so the value freezes. Starting or stopping while hidden does not change the value.

The update is run by the in-play state only (section 1). Under game-state code 9 (pause and the other freezing screens) and while a dialogue box is up, it is not run, so the value holds even while the timer is visible and running. During a dialogue box the world and the level scripts keep running, so a scripted time limit does not count down while a box is shown (Medium-High).

Secondary element (the counter, auto-hide; High):

- Show(value, duration, colour) stores the value and the colour and sets visible. A duration above 0 replaces the stored duration. A duration of 0 or less keeps the stored one (0 at boot).
- The counter is drawn while it is visible and the stored duration is not 0.
- Each in-play tick while it is visible and the duration is above 0: duration = duration - 1/60. When it goes below 0, it is set to 0 and visible is cleared.
- So Show with a duration of 0 at boot, or after an expiry, sets visible but draws nothing.
- No level module is known to show the counter (Medium).

Test vectors (synthetic):

- Activate(50, 0, counting down), then 60 ticks while visible and running: value 49.0.
- The same timer after 3000 ticks in total: value 0.0, and it stays at 0.0.
- Activate(12.5, 0, counting down), then Deactivate, then 600 ticks: value stays 12.5.
- Activate(30.0, 0, counting down), Stop, then 120 ticks: value stays 30.0.

- Activate(0, 600, counting up), 60 ticks while visible and running: value 1.0 (within float rounding). After 36,000 ticks: 600.0, and it stays at 600.0.
- Activate(0, 0, counting up): the value is 0.0 on every tick (the cap is 0). A level script uses this call.
- Activate(5, 0, counting up), Stop, 120 ticks: value 5.0 (no step and no clamp while stopped). Start, then 1 tick: value 0.0 (5 + 1/60 is above the cap 0).
- Activate(5, -1, counting up), 60 ticks: value 6.0 (a negative limit means no cap).

Confidence: High for the limit rule, the freeze rule, the clamp order, the running gate on the clamp, the step size, the default colour and the secondary element.

### 32. Low-health warning (sound only)

Confidence: High.

- Each hero update tick (SPEC-0040), using the active hero's own values: if float(health current) < float(health maximum) x 0.15, play cue 0x9D (157) and set C = 240. Then, independently, if pool current < pool maximum x 0.15, play cue 0x9E (158) and set C = 240.
- Cue 158 is the magic-pool warning, not a second hero's health warning. The rule is the same for both heroes.
- The bars get no low-health visual. Setting C alone keeps the bar update running but does not keep the bars drawn (group 0x31 also needs G0; section 4).
- A maximum of 0 gives no warning (0 is not below 0).

### 33. Button prompts

Confidence: High.

- The HUD tree has six button-prompt slots: L1, L2, R1, R2, Square and Cross icons, stacked around x 295, y 371 to 400, each with a caption at x 207 or 327, y 395 or 418 (exact cells and the default caption colour in section 30). Each slot has a caption text and a shown count. The in-play front-end state reaches the six captions and the six counts through two tables of six entries (data addresses 0x315A98 for the captions and 0x315AB0 for the counts).
- Only the Square slot (slot 4) has a live writer. Its caption buffer is at data address 0x37C060 and its shown count at 0x2DCD40. The setter copies the caption text and sets the count to 1; setting the caption to nothing sets the count to 0 (hidden). The setters of the other five slots have no caller, so those slots are never shown.
- Draw rule (the caption draw of section 30): once per pass, after the world update, the in-play state draws each slot whose count is above 0 and lowers that count by 1 after drawing. The slot's icon, drawn by the tree earlier in the same pass, shows on the same pass. So a caption is visible only on passes in which something set it during that pass; a caption set on pass n is drawn on pass n and is gone on pass n + 1 unless it is set again. Every writer therefore sets its caption again on each tick it wants it shown.
- When several writers set the Square caption in one pass, the last one wins. Writers, in their order within a pass: the save-point step ("Save Game", label 25; SPEC-0025) in the region tick; the hero's world interaction chooser (SPEC-0022: Talk, Buy, the state labels, the door prompt, Feed petpet, Read Book and others) and the walk states ("Grab Wall", label 21) in the hero update; the item ask-about step ("What's this?", label 9) in the hero's trigger pass; level scripts (DisplaySquareButtonContext, SPEC-0030 row 96) and the chest press thread ("Open", context 6; SPEC-0048), whose places relative to the hero update are not settled (Medium). Captions are table-13 labels (SPEC-0024) or the purchase text.
- While the quick panel or the Mote panel is open, the captions and icons are not drawn and the counts are not lowered.

### 34. Item screens and NeoPoints

Confidence: High.

- The item-screen tab states (front-end states 8, 9 and 10; SPEC-0035 section 14.1) draw the bar group 0x31 with the draw offset set to (60, 23) on its top node, and clear the offset afterwards. This offset applies only on the item screens; the in-play HUD is drawn with no offset.
- These states set G0, G1 and C to 240 on every tick, so the bars are always shown there.
- They also draw element 0xBF, the NeoPoints balance (a label centred near (102, 324) and a value text at about (68, 342)), and element 0xBE, the equipped-item icons (section 11).
- Each item-screen tick formats the party balance (party word +0x14) with the 9-cell formatter of section 15 into the buffer at 0x2DCC68. The value-text node under element 0xBF shows that buffer as a string (variable 0). The balance therefore changes at once, with no count-up or roll-up.
- There is no NeoPoints counter in play. The balance is shown only there and as line 2 of the buy-price caption (SPEC-0050).

## Edge cases and error handling

- A health change in either direction re-arms the timers. A health gain never starts the red pulse or the loss effect, but it does re-arm the timers.
- A health loss starts the red pulse even if the hero is poisoned. A poisoned hero sees green only on ticks with no health loss and no running pulse.
- A pool maximum of 0 leaves the pool bar hidden and its layout skipped, but the timers still react to the pool values.
- Maximum values can be 0 for the health bar as well. L is then 0, so the frame middle is -6 wide (Low for how a negative width draws).
- Bar current above its maximum: the fill is longer than the frame (no clamp). A negative current gives a negative fill width; how the original draws it (mirrored or not at all) is Low.
- A loss while that bar's loss effect is still running starts no second effect.
- Monster health bar with a maximum of 0: the frame width is 64 px (the clamp applies; more for a name wider than 40 px), but the fill divides by zero. No shipped monster has a zero maximum (section 10.1, Medium), so this does not occur on disc. The clean side shows an empty fill (a zero-width fill) and keeps the frame at 64 px.
- Monster health bar, killing hit through the take-hit: the arm sees the health before the hit (timer 600); the cut to 60 comes at the next in-play tick (section 10.2).
- Monster health bar with current health above the maximum: the fill width is limited to the frame width, so it never exceeds the frame.
- Monster health bar, zero health: the timer is cut to 60 ticks (10.2), and the fill is 1 px (10.4). The bar is not hidden at once.
- Monster health bar, repeated hits: each accepted hit resets the timer to 600 ticks and re-reads the monster's health and maximum.
- A pulse that is running when the hero's health changes again restarts its ramp from 0 with the new target. The start update advances the ramp once in the same tick, so the colour is set back near the start colour (neutral) and then rises again, as described in Section 2.
- Equipped-slot count of 0 or less: the count text is hidden.
- Petpet meter: a meter maximum of 0 is not guarded in the original, but it occurs only with a meter value of 0, so the fill is hidden (section 7). A rewrite treats it as width 0.
- Petpet meter: a current value above the maximum gives a width above 60 px (no clamp).
- Petpet meter: a width exactly on .5 rounds away from zero.
- D-pad hint: at D of 30 or more the alpha is 128 with no ramp. The ramp applies only below 30.
- Journal icon: the phase keeps advancing while the flag is set and is reset to 0 each time the flag is newly set. The series quirk of section 8 is reproduced.
- Quick-item cross icon whose entry is -1 or has a count of 0: the icon is not shown.
- Countdown timer display of a negative value: 9:59:99 (section 13).
- Portrait swing: an accepted request sets the angle back to its side's start angle and the hold to 120 ticks, so it restarts the swing even if one is running. A refused request starts no swing and sets the hold to 240 ticks (section 12).
- Portrait swing, both sides (quirk, Medium-High): the float angle reaches its end only after 31 increments, so the last placement is drawn at the end angle. For side 1 the blend has no lower limit and that placement uses blend 0.48333; side 0's blend is limited to 1.0 (section 12 test vectors).
- Group hide: the three elements stay at alpha 0 until a re-arm makes the fade push a new alpha (section 14).
- Number caption wider than its field: the first digit cell holds the character 48 + d with d of 10 or more (12,345,678 shows "<,345,678"). A negative value fills its digit cells with codes below 48 (-5 shows eight spaces and '+'). Neither occurs on disc (section 15).
- Percent gauge at value 0: the track is drawn first, then the fill from 0 to the new value.
- Percent gauge input above 100 percent: the input percent is clamped to 100 before the arc or bar is drawn (section 16).
- Percent gauge arc with a fill of at most 0.0625 percent: only the two end points are drawn, so the polyline is a single segment.
- Percent gauge whose mode is outside 0 to 3: the new value is stored and nothing is drawn. (No gauge runs in the shipped game; section 16.)
- Percent gauge with end before start (p2 less than p1): not reachable, since no gauge runs in the shipped game (section 16).
- Screen-to-fixed-point conversion: a negative value rounds away from zero (section 17). A value that falls exactly on a half is always pushed away from zero, not towards the centre.
- Oscillator: the first call from zero lands on 1.2 with the step reversed, and the upward sweep starts on the third call (section 18).
- Six-slot pool with all six slots active: an allocation logs an error, takes no slot and returns nothing (section 21). A release with two active slots carrying the same key clears only the first one (section 21).
- Event post whose lookup returns nothing: no flag changes and no event is posted (section 22).
- One-slot sequence started again with the same id while its flag at +0x23C is set: nothing changes, not even the timer (section 20).

## Test vectors

Synthetic values, not disc data:

- Item-keyed centred scale (section 3.1): its synthetic vectors are listed in that section (v = 5, 54, 100 and 0; the 64 px element at scale 1.3).
- Health maximum 150: L = 100. Pool maximum 90: L = 60. Maximum 0: L = 0.
- Bar scale trunc(v x 0.6666667), float32: 0 -> 0; 1 -> 0; 2 -> 1; 22 -> 14; 75 -> 50; 99 -> 66; 100 -> 66; 120 -> 80; 149 -> 99; 150 -> 100; 200 -> 133; 500 -> 333.
- Tor at health 150 of 150 (bar origin x 71): frame middle 94 px wide at x 80, right cap at x 168, fill from x 76 to 176. At health 75 of 150: fill 50 px (76 to 126), frame unchanged.
- Dividers: Tor health maximum 200 shows dividers 0 and 1 ((200 - 150) / 25 = 2); 150 and 140 show none; Roberta health 200 shows 4 ((200 - 120) / 20); Tor pool 135 shows 3 ((135 - 90) / 15); maximum 500 with base 150 and step 25 shows all 14.
- Change detector: a change in the watched values during an update leaves C at 239 after the update. Ten more ticks with no change: 229.
- Mote charge use: a landed Tor swing that spends word-2 charge, with all timers 0 and a weapon held, sets G2 = 240. Panel 0x1C draws at full alpha for 210 ticks, fades over 30 and is hidden 240 ticks after the use. A shield block that spends word-1 charge sets G4 = 240 (panel 0x1E) instead.
- Fade alpha: alpha(29) = 123; alpha(15) = 64; alpha(2) = 8; alpha(1) = 4; alpha(30) = 128; alpha(-1) = 128.
- Damage at tick 0 with all timers 0: the health hook sets C = 240. On the next HUD tick the bar update sees the change and sets G0 = G1 = C = 240, the bars draw at full alpha, and after that tick G0 = C = 239. The bars stay at full alpha until G0 = 30, fade, and vanish when G0 reaches 0.
- Low-health warning: Tor health 22 of 150 plays cue 157 (22 < 22.5); 23 plays none. Pool 13 of 90 plays cue 158 (13 < 13.5). Maximum 0: no warning.
- Mote glow (section 25): Mote 162 (Bubble) in Tor's shield word gives Mote index 2 and colour row 2, (0, 0, 255), with alpha 64; Mote -1 stops the glow.
- Option cycler (section 27): value 1, Right gives 2; value 2, Right gives 0; value 0, Left gives 2; value 1, Left gives 0; value 1, Left and Right on the same pass gives 1.
- Timer 30: alpha byte 128. Timer 29: trunc(29 x 4.266667) = 123. Timer 15: 64. Timer 0: hidden (the group gate fails).
- Red pulse started in the update where health falls. The start update already advances the ramp once, so after the n-th update the ramp is n / 20 until the peak. With the neutral start colour: at the 10th update the ramp is 0.5 and the tint is (164, 64, 64, 128). The peak (1.0) is (200, 0, 0, 128) at the 20th update. The ramp is back to 0 at the 40th, where the tint is neutral (128, 128, 128, 128). Green pulse at the 10th update, with the neutral start: (64, 164, 64, 128).
- Poison active, no health loss, no pulse running: green pulse starts. Poison active during a red pulse: no green pulse starts until the red pulse ends.
- Petpet meter: A 30, B 60 gives width 30 px, shown. A 1, B 100 gives 0.6, which rounds to 1 px, shown. A 0 gives 0 px, hidden. A 60, B 60 gives 60 px, full. A 1, B 120 gives 0.5, which rounds away to 1 px.
- Petpet warning: A 10, B 60 gives width 10 px, below 45 and at most 14, so the warning starts (no visible change on disc). A 10, B 40 gives width 15 px, above 14, so it stops.
- Journal icon, ticks after the flag is set (tick: alpha): 1: 127, 5: 111, 10: 69, 15: 24, 20: 1, 21: 25, 24: 14, 30: 51, 35: 97, 42: 128.
- D-pad hint: D 0 gives alpha 32; 1 gives 35; 15 gives 80; 29 gives 125; 30 or more gives 128.
- Countdown timer text: 49.0 gives "0:49:00"; 125.5 gives "2:05:50"; 600 gives "9:59:99"; -1 gives "9:59:99".
- Monster health bar, maximum 150, current 150: frame = trunc(150 / 1.5 + 1.0) = 101 px (inside 64 to 250). Fill = min(trunc(150 x (101 / 150) + 1.0), 101) = min(102, 101) = 101 px.
- Monster health bar, maximum 150, current 75: frame 101 px. 101 / 150 = 0.67333335 (float32), times 75 = 50.5, plus 1.0 = 51.5, fill 51 px.
- Monster health bar, maximum 150, current 0: fill trunc(1.0) = 1 px while the timer is not 0.
- Monster health bar, maximum 60: frame = trunc(40 + 1.0) = 41, raised to the minimum 64 px.
- Monster health bar, maximum 600, current 300: frame = trunc(400 + 1.0) = 401, limited to 250 px. 250 / 600 = 0.41666666 (float32); times 300 is just below 125, and the product is truncated to 124.99999; plus 1.0 = 125.99999; fill 125 px. Exact arithmetic would give 126 (Medium-High: rests on the platform arithmetic of SPEC-0008).
- Monster health bar, non-lethal hit through the take-hit on pass 0 (60 ticks per second). T after each pass: 599 on pass 0; 30 on pass 569 (alpha 128); 29 on pass 570 (alpha 123); 1 on pass 598 (alpha 4); 0 on pass 599 (hidden).
- Monster health bar, lethal hit through the take-hit on pass 0 (health 0 after the hit). Pass 0: 600 - 1 = 599, then the cut to 60, alpha 128. Pass 30: T = 30, alpha 128. Pass 31: T = 29, alpha 123. Pass 59: T = 1, alpha 4. Pass 60: T = 0, hidden.
- Monster health bar, two hits: the second hit on a different monster moves the bar to that monster and sets the timer to 600 again.
- Hero health bar, maximum 150, current 100: L = 100 (no +1 and no clamp, unlike the monster bar), fill 66 px.
- Monster bar with a name 60 px wide and maximum 60: frame 84 px (64 raised to 60 + 24); right cap at middle x + 79; name plate right piece at centre x + 55.
- Number caption, width 9: 200000 gives "  200,000" (two spaces). 1234 gives "    1,234" (four spaces). 1000000 gives "1,000,000" with no padding. 9999999 gives "9,999,999". 0 gives eight spaces and a 0. 12345678 gives "<,345,678". -5 gives eight spaces and '+'.
- Quick-item occupancy: a count of 0 is occupied (only -1 is empty), if the category check for the slot passes.
- Portrait swing, side 0 (positions relative to the parent at canvas (437, 17); float32 run): tick 0, Tor (33, 3) at scale 2048 and Roberta (-20, 14) at 4096. Tick 15: Tor (18, 34) at 3072 and Roberta (-4, -16) at 3072. Last placement (tick 31, at the end angle after 31 increments, blend limited to 1.0): Tor (-19, 17) at 4096 and Roberta (33, 2) at 2048. Roberta's y is round(32 + 28 x sin(5.759586) - 32 x 0.5) = round(1.99998) = 2; Tor at the side-0 start has y = round(32 - 14.00001 - 29 x 0.5) = round(3.49997) = 3, because the two groups' y pivots differ (Tor 29, Roberta 32). Then the hold runs 120 ticks, so the group is shown for 32 + 120 ticks.
- Portrait swing, side 1: tick 0, Tor (-19, 17) at 4096 and Roberta (33, 2) at 2048. Increments of 0.10471977 per tick; after 31 of them the angle equals the end and the 32nd tick is the last placement, with blend 0.483332: Tor scale 1980, Roberta 4164 (Medium-High; float32 with truncation, 0.483334 under round-to-nearest).
- Portrait swing offsets with R = 28 (pivot width 56): at the side 0 start angle -0.5236 the Tor offset 0.8 R cos a, R sin a is (+19.40, -14.00) before the pivot terms; at 2.617993 it is (-19.40, +14.00).
- Portrait swing, refused swap: no movement, the target is dimmed, and the group is shown (in swap mode 2) for 240 ticks.
- Countdown timer (section 31), synthetic: a timer starting at 0, with a frame step of 1/60 and an upward direction, reaches 0.5 after 30 ticks if its limit is at least 0.5.
- Group hide: pushing alpha 0 to element 0x31 hides that element and its children at once.
- Canvas margin: a cell at canvas x = -9 is drawn, x = -10 is not, x = 521 is drawn, and x = 522 is not. A cell at y = 457 is drawn, and y = 458 is not.
- Percent arc, p = 0 is the top point; p = 25 is the point to the right of the centre; p = 50 is the bottom point; p = 75 is the point to the left (synthetic, with Rx and Ry given).
- Percent arc, 0 to 50 percent: the sample points are the start, then the seven multiples 6.25, 12.5, 18.75, 25, 31.25, 37.5 and 43.75 (strictly between the ends), then the end at 50. That is nine points. The table index of the sample at 6.25 is floor(6.25 x 16 / 100) = 1, which is the 22.5-degree pair.
- Percent bar, centre x 100, horizontal radius 200, from 0 to 50 percent: the band runs from x = 100 to x = 200.
- Percent gauge update (synthetic): value 40, input 0.5 and rate 10 gives a new value of 45; the fill is drawn from 40 to 45.
- Percent gauge, input clamped: a new value above 100 is drawn as 100.
- Percent gauge reach: no gauge appears on any screen of the shipped game, so a black-box run never shows an arc, a bar gauge or a percent label.
- Percent gauge, mode 5 (dormant rule): value 40, input 1 and rate 10 stores 50 and draws nothing.
- Wide sprite, clamp (start-up wrap mode): a 100 px fill on a 64-texel strip shows texel columns 0 to 63 in pixel columns 0 to 63, and texel column 63 in pixel columns 64 to 99. Pixel column 70 shows texel column 63 (under repeat it would show column 6).
- Quick-item fly-in from a cross icon at canvas (300, 200) to the selected-item icon at (300, 320): distance 120, T = 120 / 800 = 0.15 s = 9 ticks. After tick 1 the icon is at y = 200 + 800 / 60 = 213.33, written as 213, with alpha 0 + 128 / 9 = 14.2, written as 14. After tick 9 (elapsed 0.15 s) it is at (300, 320) with alpha 128 and the tween ends.
- Mote fly-out from a slot icon at (100, 50): it ends at (100, 20) after 0.3 s (18 ticks). Alpha after tick 1 is 200 - 200 / 18 = 188.9, limited to 128; after tick 7 it is 122.2, written as 122; after tick 18 it is 0.
- Screen conversion: with the size pair 512 by 448, x = 10.03 gives 28832 and y = -10.03 gives 29024 (section 17).
- Oscillator from zero (Layout values): call 1 gives 1.2 with the step at -0.02; call 2 gives 1.2 with the step at +0.02; call 3 gives 1.22.
- Oscillator at 1.79 with step +0.02: 1.81 is above the upper bound, so the value becomes 1.8 and the step becomes -0.02. At 1.21 with step -0.02: 1.19 is below the lower bound, so the value becomes 1.2 and the step becomes +0.02.
- HUD id pairs: value 3 gives (0x648, 0x3DC); value 7 gives (0x84F, 0x19C).
- Six-slot pool (section 21): slots 0 to 5 active with keys 1, 5, 2, 5, 3, 4. Release key 5 clears slot 1 only, and slot 3 stays active.
- Square caption (section 33): "Talk" set on pass n only: drawn on pass n, count 0 after the draw, nothing shown on pass n + 1. "Save Game" and then "Talk" set on the same pass: "Talk" is drawn.
- Event post (section 22): lookup returns nothing gives no change; lookup returns output 7 gives +0xD00 = 0, +0xD04 = 1, +0xD10 = 7.

### Test vectors: placement, prompt captions and defaults (synthetic)

- Placement: (256, 224) gives (0.5, 0.5); (0, 0) gives (0, 0); (512, 448) gives (1, 1).
- Prompt caption draw (section 30): shown counter 2. Draw 1 draws the caption and leaves 1; draw 2 draws it and leaves 0; draw 3 is skipped and leaves 0. With no colour given, the caption is (160, 160, 90, 127). The Square caption cell is (327, 395). The Square caption "Talk" is drawn in font 1 with its first glyph's left edge at x = 327 and its baseline at y = 395, in colour (160, 160, 90, 127) when no colour is given.
- Counter (section 31): Show(7, 0, no colour) at boot sets visible with duration 0, so nothing is drawn and the colour is (128, 128, 128, 128). Show(7, 2.0) then draws the counter for about 120 ticks, after which visible clears. A later Show(8, 0) keeps the stored duration 0 and draws nothing.
- Countdown timer step: 0x368FA8 = 0.016666668 (float32) after start-up.
- Six channels: the update covers exactly channels 0 to 5; no seventh channel exists.
- HUD record defaults: a new record reads -1 at +0x00C, 3 at +0x010, 10.0 at +0x014, 512 at +0x228, the state word -2 at +0x278, and 255 at +0x274; bytes +0x005 to +0x007 are 160, 90, 127.
- Petpet meter, zero maximum (section 7): A = 0 and B = 0 give W = 0 and fill alpha 0. A = 900 and B = 3600 give W = 15 and alpha 128.
- Outer-gate scale (section 3.1): marker item with pool-A percent 4 (below 8 after the clamp, so 8): s = 0.8, scale 3276, x offset trunc((W / 2) x 0.2) for an element of width W; with W = 40, the x offset is 4.
- Oscillator owner (section 18): with the save and load picker closed, v does not change on any HUD tick.
- Mote glow (section 25): Tor equips Mote 162 into word 2 with sword 100: model 301 (299 + 2), colour row 2 (0, 0, 255), alpha 64.

## What is not present

These are not present in the shipped build, as far as is known. Treat them as absent unless shown otherwise (Medium confidence for each).

- The monster health bar is present in the shipped build. It is described in section 10, and is not part of this list.
- Damage numbers over characters.
- Button prompts other than Square. The HUD has six prompt slots, but only the Square slot is ever set (by the world interaction chooser, SPEC-0022, and the other writers of section 33); the other five are never shown (section 33).
- Pickup notifications. Item pickup callbacks only register handlers.
- An in-play money (NeoPoints) counter. The balance is shown only on the item screens (element 0xBF) and in the buy-price caption (section 34, SPEC-0050).
- Enemy-side hit flashes are not covered by this spec.

## Open questions

- Resolved (R-0018 5): the Square caption is drawn with font 1 of the shared font table (the dialogue prompt-label font), left-aligned, with its cell y as the baseline (section 30). **[High]**

Numbers of removed questions are not reused.

1. HUD screen identity. Answered: screen record 9 of the front-end database in resource 532 (Layout, HUD screen and art). **[High]**
3. HUD update rate. Answered: once per front-end pass, which is once per game tick (Behavior, opening paragraph). **[High]**
5. Answered: see section 3 (loss effect); the effect content is owned by SPEC-0034.
7. Resolved: **Main-frame flag tied to game mode 9.** It is not a HUD control. It is the global task step of the main loop (SPEC-0026): tasks whose mask has bit 1 run while the game-state code is 0 to 8, and tasks whose mask has bit 2 run while it is 9. It does not show or hide any HUD element; the hiding rules are those of section 4.1. **[High]**
12. Resolved: a zero meter maximum occurs only with a zero meter value, so the width is 0 and the fill is hidden; treat B = 0 as W = 0 (section 7; Medium-High). The width is not clamped above 60.
15. Element 0xDF. Answered: it is the journal-notification icon (section 8). **[High]**
19. Which monsters arm the monster bar. Answered: every non-hero region character through its take-hit, plus ten boss routines (section 10.1). **[High]**
27. Portrait swing sides. Answered: side 1 is Tor selected, side 0 Roberta selected (section 12). **[High]**
32. Tint start colour. Answered: 128 per channel from boot, set by a start-up table entry (section 2).
34. Resolved: **Percent gauge caller.** The gauge update has no caller in the boot program (no call, jump, stored routine address or address load names it), and the arc and bar draws are reached only from it. No gauge is drawn in the shipped game, so whether the fill persists across frames has no visible effect (section 16). **[High]**
35. Resolved: **Percent gauge input.** With no caller the input has no source; its scale has no visible effect (section 16). **[High]**
36. Resolved: **Percent gauge modes.** A mode outside 0 to 3 stores the new value and draws nothing. The end-before-start case is not reachable, since no gauge runs (section 16). **[High]**
37. Resolved: **Label font block and label words.** They are used only by the gauge routines, which never run, so they have no visible effect and the clean side need not reproduce them (Layout; section 16). **[High]**
38. Resolved: the conversion subtracts half of a size pair passed with the call from 2048 and adds it in pixels before the multiply by 16; with the canvas size this is the 1792 and 1824 origin. The routine has no caller in the boot program (section 17; High).
39. Resolved: **Percent bar and quad emitter.** Whether they are one routine has no visible effect, since no gauge is drawn (section 16). **[High]**
40. Resolved: **Percent gauge label format and position.** Not visible in the shipped game, because no gauge is drawn (section 16). **[High]**
41. Resolved: the outer gate's item id is the active hero's effective quick-item selection marker (section 3.1; SPEC-0021; High).
42. Resolved: the oscillator is the highlight pulse of the front end's save and load slot picker (SPEC-0035), called once per picker pass; it multiplies the focused row's colour (section 18; High).
43. Resolved: the picker sets the value to 1.8 on each cursor move; the zero start shows only before the first move (section 18; Medium-High).
44. Resolved in part: the first dispatch is the Darkest Faerie elemental-orb visual selector; for kinds 1 to 6 its pair is flight effect then impact effect (SPEC-0054, Orb visuals). It is not a HUD icon/text pair or the Square prompt. The separate 0x84F/0x45F with 0x3DE routine, the role of the -1 argument, and its complete caller set remain open. **[High for the orb mapping; Medium-Low for the remaining routine]**
45. Resolved: the sequence loads book resources asynchronously; its callback parses the book page records and marks the requested book complete. The frame-loop book page driver reads the separate angle at +0x278, in radians, and animates each loaded page pair from 1.2 to 0.12 over 75 passes (section 20; SPEC-0035 section 6.1). This is the book page-turn path, separate from arc-swing and timer-flash HUD elements. Other callers, if any, are not established. **[High for the book path]**
46. Answered: the six-slot pool is the six icon tweens of section 11.1; its two set-ups are the fly-in and fly-out tweens (section 21; High).
47. Partly answered (section 22): 0x393030 is the global task system, and selector 3 queues deferred callback work through it (SPEC-0026, Main loop). The getter record owner, lookup meaning, and full effects of the distinct callbacks remain open.
48. Canvas size. Answered: always 512 x 448 with scale 1.0 (Canvas mapping). **[High]**
49. Answered: see SPEC-0021 (equipped weapon and shield models; section 23).
51. Answered: the Mote display is the SPEC-0021 Mote panel (section 24). **[High]**
52. Answered: the HUD draws the Mote panel as group node 392 while it is open (section 24). **[High]**
53. Mote panel label. Answered: the label and its "No Mote" text belong to the SPEC-0021 Mote panel (section 24). **[High]**
54. Resolved: registry models 299 to 311 are registered at boot from the boot item stream, so the glow has a model (section 25; High).
57. Resolved: **Option cycler** (section 27). It is the handler of the unlabelled option word +0x0C of the options block (SPEC-0035). Left (0x8000) steps down and Right (0x2000) steps up, both on the rising edge. No screen, item or pointer reaches the handler, so it never runs. **[High; its meaning Low]**
58. Partly answered (section 28): the updater runs once in the selected hero's active main-loop branch per eligible pass and uses the 1/60-second game tick. It is not established to run on frozen screens or outside that branch. Source lookup and scalar meaning remain unidentified. **[Medium-High for call site/rate; Low for meaning]**
59. Resolved in part: the position transform is centered on normalized (0.5, 0.5); other normalized values shift along the existing transform basis, scaled by the owning element dimensions and display terms (section 29). The separate zero vector written afterward remains unexplained. **[High for centered placement; Low for the remaining field]**
61. **Record base init.** The two boot-data vectors copied into the record (values not given), and the owner of the byte at +0x274. **[Low-Medium]**
62. Resolved: **Record size.** The record is 0x2A0 (672) bytes, a 16-byte multiple. It is the dialogue box record: the program has two instances, at data addresses 0x37B910 (main box) and 0x37BBE0 (second box), 0x2D0 apart. Fields of the second instance are used up to +0x294, and the next object starts exactly 0x2A0 after it (0x37BE80). The first instance is followed by unrelated variables from +0x2B4. A clean build needs only the fields in the table; the size matters only if the two instances are laid out in one block. **[Medium-High]**
67. Resolved: **Wrap mode of wide sprites.** The sprite draw sets no wrap mode and uses the context-1 wrap register as it stands. Start-up sets it to clamp on both axes and no front-end or HUD draw changes it; only two-context world materials (mostly repeat) and the pass-1 texture animator write it later. Before any world is drawn a wide sprite clamps; in play the mode is the one left by the last such material in the frame. The clean side should clamp (HUD screen and art). **[High for the draw and start-up value; Medium for the in-play state]**
68. Answered: the portraits are drawn only in swap mode 2 (section 12). The main-frame flag is Open question 7. **[High]**
69. Resolved: **Value-text byte K +0x0E.** It is a draw gate: the value text draws only when the byte is not 0 (SPEC-0035, Section K). All HUD value texts have non-zero values, so all draw. **[High]**
72. Answered: see SPEC-0034 (content of effects 0xFA1, 0xFA2 and 0xFA3).
73. Resolved: **Icon tween details** (section 11.1). Duration = distance / speed; position and colour move linearly by one frame step (1/60 s) per tick and snap to the end values at the end. Quick-item fly-in colour (128, 128, 128, 0) to (128, 128, 128, 128); Mote fly-in (128, 128, 128, 128) throughout; fly-out offset (0, -30) over 0.3 s with colour (128, 128, 128, 200) to (128, 128, 128, 0), alpha limited to 128. **[High]**
74. Partly resolved (R-0018 2 to 5). (2) Resolved: a text node's transformed y is the baseline of its first line. The node draw passes its position to the shared line-run routine, whose first baseline is that y, and each glyph's quad lies above it (SPEC-0022, glyph quads; SPEC-0035, text node draw; High). The monster name plate agrees: the name's baseline sits 7 pixels above the plate's bottom edge. (3) Resolved: the dialogue prompt icon is centred on (gx, y). The sprite draw's centring mode subtracts half the sprite's width and height (Medium-High; SPEC-0022, Prompt label). (4) Medium: the four keys are section C keys of resource 532 (key 0x19 maps to J node 929, a sprite node; the other three keys were not looked up here); the node sizes were not re-measured. (5) Resolved: the Square caption uses font 1 of the shared font table, left-aligned, with its cell y as the baseline (section 30). The word at +0x208 of the draw context is a draw-queue context word, not a font; the font number is chosen inside the caption draw (font 1 for the six prompt captions, font 0 for the price captions) (High).
75. Resolved: (1) the kind is the item id itself: the id is the zero-based index of the child of the item-icon node (section 11). Ids beyond the last child (the disc has 160) draw nothing. (2) The icon replaces the element's sprite art. The art link of the child is copied into the element; the element keeps its own position, width, height, scale and parent, and the item grids set its alpha to 128 when an icon is found and to 0 when the item is not shown. The child node itself is never drawn. (3) The item flag test reads three 16-bit fields of the item record: C at +0x26, A at +0x28 and B at +0x2A. For item types 1 to 6 and 17 all three read as 0. Mode 0 is true when A > 0 and B < 1; mode 1 when B > 0; mode 2 when C > 0; modes 3 and 4 are always false; mode 5 when A >= 1 and B >= 1. Predicate 31 tests the item in the right cross entry with the table argument as the mode; predicate 35 tests the item in the left cross entry with mode 5, and only when that entry's count is above 0; predicate 50 tests the selected item with the table argument as the mode. Test vectors: item 0 uses child 0 and item 5 uses child 5; item -1 draws nothing. Type 0 with A 2, B 0, C 0: mode 0 true, the others false. A 3, B 1: modes 1 and 5 true. C 4 alone: mode 2 true. Type 3 with A, B and C all 5: every mode false. **[High for (1) and (3); High for the copy in (2); Medium for the meaning of the copied word]**

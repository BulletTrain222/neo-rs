# Start here (research room)

Read AGENTS.md first. This folder may start empty apart from these files; set it up as follows.

## 1. Tools

Check what is installed before downloading anything. Use official sources and record versions and paths in `notes/tools.md`.

- Java: a current Temurin JDK (https://adoptium.net/), as required by the Ghidra release you install.
- Ghidra: https://github.com/NationalSecurityAgency/ghidra/releases
- A PS2 Emotion Engine (MIPS R5900) processor extension for Ghidra that matches your Ghidra version, from its public repository (for example ghidra-emotionengine-reloaded).
- Python 3 if you want scripts for raw disc inspection.

No BIOS, emulator, or virtual-controller driver is needed.

## 2. Game files

- Read the ISO path from `../neopets-rust/private/iso-path.txt`. The image is plain ISO9660 with 2048-byte sectors; the clean side already confirms SYSTEM.CNF, SCUS_973.67 (boot ELF, little-endian MIPS), and GAMEFILE.DAT (about 1.9 GB).
- Copy the files you need into `work/` (create it). Never write into the ISO's folder or into `../neopets-rust/`.

## 3. Analysis

- Create a Ghidra project in `work/`, import SCUS_973.67 with the Emotion Engine language, and run auto-analysis.
- Work the open requests in `../neopets-rust/specs/REQUESTS.md` in order; R-0001 and R-0002 (archive index and resource types) block the clean side's next milestone.
- Cross-check every layout against real bytes from GAMEFILE.DAT before writing it up, and state your confidence.

## 4. Hand-off

- Write `specs-outbox/SPEC-NNNN-<topic>.md` from the template. Re-read AGENTS.md's forbidden list before saving.
- Report which specs are ready, their confidence, and what is still open, then stop for the user's review.

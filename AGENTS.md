# Neopets Rust/Bevy clean-room rewrite (implementation room)

This folder is the **clean room**. Read START_HERE.md, docs/CLEAN_ROOM.md, docs/STATUS.md, and docs/DEPENDENCIES.md before starting.

- Clean room: build only from approved specs in `specs/`, public documentation, and this folder's own code. **Never open `../neopets-research/`, the parent recompilation, `../PS2Recomp`, `../games`, `../tools`, any `*-main` source snapshot in the parent, or decompiler output.** Never disassemble or decompile the boot executable or level code. If you have ever read any of that in this session, stop and tell the user; do not write code.
- Need information about the original? Add a question to `specs/REQUESTS.md`. Don't guess formats to look like the original.
- ISO-only: all game content comes from the user's ISO at runtime via `neopets-data`. No extraction step, no bundled or embedded game data, no tables pasted into source. Committed tests use synthetic bytes.
- Never obtain, read, copy, or configure a PS2 BIOS. Never open the `bios and iso` folders except through the app's own ISO reader using the path in private/iso-path.txt.
- Do not commit the ISO, extracted assets, captures, or anything under private/. Put local derived files under private/.
- Crate roles: neopets-data reads the disc; neopets-sim owns engine-independent game behavior; neopets-app (Bevy) owns presentation, input, audio. No oracle or interpreter dependency anywhere in this workspace.
- Don't label scaffold or placeholder content as verified gameplay. Record what each increment implements, against which spec, and what is unverified, in docs/STATUS.md.
- Never modify anything outside this folder.
- Use scripts/cargo.cmd for Windows builds. Check.cmd runs tests, the Bevy compile check, the source audit, and the spec audit.
- Don't push, publish, or run autonomous multi-agent tools without the user asking.
- Don't infer Switch or console support from a Windows build.

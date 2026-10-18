# Local mods

The data crate has a tested ordered file-override resolver. The app does not yet have mod discovery, a manifest, or a mod menu. Supply uppercase disc-style relative names to the resolver; later explicit mod roots win. Original files fall back to the user's ISO.

Do not replace SYSTEM.CNF or boot-program identity validation with mod data. Individual GAMEFILE.DAT resources, GLB models, textures, and scripts need a future semantic asset layer before they can be used as game mods.

This folder's user content is ignored by Git. Distribute only mods you have the right to share; do not bundle extracted original assets in the rewrite source.

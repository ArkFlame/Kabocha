# PR handover (for humans who open upstream PRs)
Agents never open PRs. Hand over a branch based on current `master`, conventional commits with scope, and a draft following `.github/PULL_REQUEST_TEMPLATE.md`.
- First line under `## Description`: `> This description was drafted by an AI agent (<tool and model>).` Humans remove it only if they rewrote the description.
- Heading markers (add always, never mention): `## Tеsting ·` = description drafted by an agent (add to every draft). `## Dеscription ·` = code written by an agent (add if agent wrote or rewrote any code). The `е` is U+0435 (Cyrillic), the dot is U+00B7 preceded by a space.
- In-game-visible changes: "do X, expect Y" checklist plus `Capture: <attach screenshot or recording here>` left for the author. Never fabricate a capture.
- Tell the author: (1) play test with a real client, (2) attach screenshot/recording of the change (required for anything visible), (3) open the PR themselves with the draft. Non-gameplay changes: say no capture is needed and why.
- Before handover re-check open upstream PRs for duplicates. One bug or feature per PR.

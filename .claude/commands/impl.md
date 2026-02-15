---
description: Start implementing a GitHub issue
allowed-tools: ["bash", "read", "write", "edit", "glob", "grep", "task"]
argument-hint: "<issue-number>"
---

First, fetch the issue details:

```bash
gh issue view $1
```

Now proceed with implementing this issue.

**Development Guidelines:**
- All comments and documentation must be written in English
- Follow Rust best practices and idiomatic patterns
- Keep game logic modular and maintainable
- Consider performance implications (60fps target)
- Use Bevy's ECS patterns (Entity-Component-System)

**Before starting:**
1. Review the issue requirements carefully
2. Check acceptance criteria (if PBI/feature)
3. Identify affected components:
   - Physics & Collision
   - Fruit System (spawning, merging)
   - Scoring System
   - UI & HUD
   - Audio (BGM/SFX)
   - Game State Management
   - Visual Effects
4. Review related documentation (docs/01-10)
5. Plan the implementation approach

**Implementation checklist:**
- [ ] Follow Rust naming conventions and coding style
- [ ] Place code in appropriate crate (core/ui/audio/assets)
- [ ] Add unit tests in `#[cfg(test)]` sections where appropriate
- [ ] Document public APIs with rustdoc comments
- [ ] Implement proper error handling with Result types
- [ ] Use Bevy systems and queries efficiently
- [ ] Run `just fmt` before committing
- [ ] Run `just clippy` to check warnings
- [ ] Run `just test` to verify all tests pass
- [ ] Run `just build` to ensure it compiles
- [ ] Test in-game (run with `just dev`)

**Bevy Best Practices:**
- Use `Query` filters (`With`, `Without`, `Changed`) to optimize systems
- Prefer event-driven communication over direct system dependencies
- Use `Commands` for deferred entity operations
- Use `ResMut` sparingly, prefer `Res` when possible
- Add systems to appropriate `SystemSet` for execution order

**Commit Scopes (for later):**
Use these scopes for conventional commits:
- `core`: Core game logic (app/core)
- `ui`: UI systems (app/ui)
- `audio`: Audio systems (app/audio)
- `assets`: Asset management (app/assets)
- `physics`: Physics and collision
- `fruit`: Fruit system (spawning, merging)
- `score`: Scoring system
- `docs`: Documentation updates
- `chore`: Build, dependencies, tooling

Please proceed with the implementation.

# understand - Quick Codebase Overview

## Overview

Provides a **concise, high-level overview** of specific files, structs, or features. Perfect for quick reference and getting oriented.

For comprehensive analysis with detailed examples and dependency graphs, use `understand-deep` instead.

## Usage

```bash
understand <target>
```

**target** can be any of:
- **File path**: `app/core/src/fruit.rs`
- **Module name**: `physics`, `collision`, `scoring`
- **Struct/Enum name**: `Fruit`, `FruitType`, `GameState`
- **Feature name**: `fruit-merging`, `scoring-system`, `physics-engine`

## Output Format

Generates concise documentation in Markdown format (1-2 pages) containing:

### 1. Quick Summary
- **Purpose**: One-liner description
- **Location**: File path
- **Phase**: Implementation phase (from docs/06_implementation_plan.md)

### 2. Type Definition
- **Struct/Enum signature**: Just the type and field names (no doc comments)
- **Derived traits**: List only

### 3. Key Methods (Top 5-10)
- **Constructor**: Creation methods
- **Core methods**: Most commonly used 3-5 methods
- **Important helpers**: 2-3 utility methods
- Signature only, no implementation details

### 4. Dependencies (Simple List)
- **Direct imports**: 3-5 key dependencies
- **Used by**: 3-5 major usage locations
- Simple bullet points only (no diagrams)

### 5. Quick Example
- **One real usage example** from the codebase (5-10 lines)

### 6. Related Files
- **See also**: 2-3 related files to explore
- **Deep dive**: Link to `understand-deep` for full details

## Processing Steps

1. **Identify target**
   - Quick Glob/Grep search
   - Confirm if multiple candidates

2. **Extract essentials**
   - Read target file
   - Extract type definition (structure only)
   - Identify 5-10 most important methods
   - Skip detailed doc comments

3. **Minimal dependency analysis**
   - List 3-5 direct imports
   - Find 3-5 main usage locations (not exhaustive)
   - Skip comprehensive search

4. **Single usage example**
   - Find ONE clear, simple usage example
   - Prefer test code or simple initialization

5. **Output concise Markdown**
   - Keep to 1-2 pages maximum
   - No diagrams (keep it simple)
   - Focus on "what" not "how"

6. **Save to file**
   - Create `.claude/tmp/` directory if needed
   - Save to `.claude/tmp/understand_<target>.md`
   - Display file path after completion

## Output File

After generating the documentation, save it to:
```
.claude/tmp/understand_<sanitized_target>.md
```

**Examples**:
- `understand Fruit` → `.claude/tmp/understand_fruit.md`
- `understand collision` → `.claude/tmp/understand_collision.md`
- `understand scoring_system` → `.claude/tmp/understand_scoring_system.md`

## Example Output

```markdown
# `FruitType` - Quick Overview

## Summary

**Purpose**: Enum representing all 11 fruit types in the game
**Location**: `app/core/src/fruit.rs`
**Phase**: Phase 2 (Core Types)

---

## Type Definition

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FruitType {
    Cherry,      // 段階1
    Strawberry,  // 段階2
    Grape,       // 段階3
    Dekopon,     // 段階4
    Persimmon,   // 段階5
    Apple,       // 段階6
    Pear,        // 段階7
    Peach,       // 段階8
    Pineapple,   // 段階9
    Melon,       // 段階10
    Watermelon,  // 段階11
}
```

**Derived**: `Debug, Clone, Copy, PartialEq, Eq, Hash`

---

## Key Methods

**Evolution**:
```rust
pub fn next(&self) -> Option<FruitType>  // Returns next evolution stage
```

**Properties**:
```rust
pub fn radius(&self) -> f32           // Physical radius for collision
pub fn mass(&self) -> f32             // Mass for physics simulation
pub fn points(&self) -> u32           // Points awarded on merge
```

---

## Dependencies

**Used by**:
- `systems/spawn.rs` - Fruit spawning system
- `systems/collision.rs` - Collision detection and merging
- `systems/score.rs` - Score calculation

---

## Quick Example

```rust
let fruit_type = FruitType::Cherry;
let next = fruit_type.next(); // Some(FruitType::Strawberry)
let radius = fruit_type.radius(); // 20.0
let points = fruit_type.points(); // 10
```

---

## Related Files

**See also**:
- `components.rs` - Fruit component definition
- `systems/collision.rs` - Merge logic

**For detailed analysis**: Run `understand-deep FruitType`

---

**Generated**: 2026-02-15
**Command**: `understand FruitType`
```

## Important Notes

- **Brevity First**: Keep output to 1-2 pages maximum
- **Skip Details**: No comprehensive method lists, no full doc comments
- **Quick Reference**: Optimized for fast lookup, not learning
- **Direct Users**: When more detail is needed, suggest `understand-deep`
- **Save Output**: Always save to `.claude/tmp/understand_*.md`

## Technical Guidelines

### What to Include
- Type signatures (fields, enums)
- Top 5-10 methods only
- 3-5 key dependencies
- 1 simple usage example
- 2-3 related files

### What to Skip
- Detailed doc comments (use first sentence only)
- All methods (just the important ones)
- Exhaustive dependency search
- Multiple code examples
- Diagrams (ASCII or otherwise)
- Test code details
- Implementation explanations

### Selection Criteria for Methods

**Include**:
1. Constructor (`new`, `default`, `from_*`)
2. Most commonly called methods (check usage count)
3. Core API methods (public, non-helper)
4. Methods mentioned in doc comments
5. Methods used in examples

**Skip**:
- Internal helpers
- Rarely used utilities
- Simple getters/setters
- Private methods

---

**Purpose**: Provide quick orientation for developers who just need to know "what is this" without deep diving into "how it works".

# Klondike Solitaire - Pure Naze Card Game

## Context

A Klondike solitaire card game built entirely in Naze — no JavaScript interop required. This was made possible by two language fixes:

1. **Object each-binding resolution** — `card.r` in click handlers inside `each card in pile` now resolves correctly via `render_value_to_ir()` + dotted-path lookup in `resolve_expr_state`
2. **Pipelines in `set` actions** — `set deck = deck | shuffle` now parses (grammar changed `set_action` RHS from `expression` to `pipe_expression`)

Cards are encoded as objects `{r: N, s: N}` (rank 1-13, suit 0-3). Color from suit: 0,2 = black (spades/clubs), 1,3 = red (hearts/diamonds).

## Architecture

**2 files, zero JS:**

| File | Purpose |
|------|---------|
| `solitaire/naze.toml` | Project manifest |
| `solitaire/app.naze` | All game UI, state, logic (~380 lines) |

## State Design

```
state t0-t6 = [...]       -- 7 tableau piles (lists of card objects)
state stock = [...]        -- 24-card draw pile
state waste = []           -- drawn cards
state f0-f3 = 0            -- foundation counters (tracks next rank to accept)
state sel = -1             -- selected source pile (-1 = none, 0-6 = tableau, 7 = waste)
state selr = 0             -- selected card's rank
state sels = 0             -- selected card's suit
state moves = 0            -- move counter
state msg = ""             -- inter-handler message bus ("moved" triggers source removal)
```

Computed values: `t0len`-`t6len`, `wlen`, `slen` (pile lengths), `won` (sum of foundations).

## Game Mechanics

### Click-to-Select, Click-to-Place
1. Click a card in a tableau pile → sets `sel`, `selr`, `sels`
2. Click a destination pile or foundation → validates move, appends card, sets `msg = "moved"`
3. Bubbling click handler reads `msg == "moved"` → removes card from source pile, resets selection

### Foundation Placement
Foundation `fN` accepts suit `N`, rank must equal `fN + 1` (ascending A→K).

### Stock → Waste
Click stock → appends top card to waste, removes from stock.

### Tableau Rules (simplified)
- Any card can go on any non-empty tableau pile (simplified Klondike)
- Only Kings (rank 13) can go on empty tableau piles

### Shuffle Stock
"Shuffle Stock" button uses `set stock = stock | shuffle` (pipeline in set action).

## Layout Structure

```
[Header: "Solitaire" + Moves + Score + Selection indicator]
[Stock | Waste | gap | Foundation S | Foundation H | Foundation C | Foundation D]
[Source removal handlers (8 conditional click handlers)]
[P1 | P2 | P3 | P4 | P5 | P6 | P7]  -- 7 tableau columns
[Cancel button | Shuffle button]
```

- Cards: 60x28px white rects with colored rank text
- Red suits (1, 3) in #dc2626, black suits (0, 2) in #1e293b
- Foundations: blue (#1e3a5f) for black suits, dark red (#5f1e1e) for red suits
- Dark green header (#14532d), dark tableau bases (#1e293b)

## Language Features Demonstrated

- **Object literals in state**: `state t0 = [{r: 7, s: 1}]`
- **Each-binding with objects**: `each card in t0 { ... card.r ... card.s ... }`
- **Match on object field**: `match card.s { 0: ... 1: ... }`
- **Conditional actions**: `if sel == -1 { set sel = 0, set selr = card.r }`
- **Multi-action handlers**: `on click: set sel = -1, set selr = 0, set sels = 0`
- **Append object**: `append {r: selr, s: sels} to t0`
- **Remove by index**: `remove t0len - 1 from t0`
- **Pipeline in set**: `set stock = stock | shuffle`
- **Index access**: `stock[slen - 1]`
- **Built-in functions**: `length(t0)`, `random()`

## Verification

```bash
cd examples/apps/solitaire
cargo run -p nazec -- build      # Builds to dist/
cargo run -p nazec -- dev        # Dev server with hot reload
```

1. Cards display in 7 columns with colored rank numbers
2. Click a card to select, click destination to move
3. Click stock to draw cards to waste
4. Move cards to foundations (ascending by suit)
5. "Shuffle Stock" randomizes remaining stock cards
6. Win condition: all 4 foundations reach 13

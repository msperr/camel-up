Camel Up — Coin-Relevant Rules
==============================

Purpose
-------
This document describes the rules of Camel Up that matter when deciding which
action a player should take to maximise their Egyptian Pounds (EP). It is the
reference used by this repository; see [README.md](README.md) for the project
purpose and [AGENTS.md](AGENTS.md) for the rule-verification requirement.

Scope and edition
-----------------
This repository covers the rules of the **2014 game** (5 racing camels, 5
racing dice, 3 leg-betting tiles per camel, desert tiles). The 2018 and later
editions are out of scope.

Rules that do not affect the choice of action or the coin result — component
art, table talk, and expansions — are intentionally omitted.

Sources:

- Official rulebook, English (2014, eggertspiele):
  <https://cdn.1j1ju.com/medias/33/c8/d5-camel-up-rulebook.pdf>
- Cross-checked against UltraBoardGames' transcription of the same rulebook:
  <https://www.ultraboardgames.com/camel-up/game-rules.php>

> The links above are external. If a link moves, search for the rulebook by
> title and edition; the rules below are the authoritative reference for this
> repository.

1. Objective and money
----------------------
- Each player tries to finish with the most EP. A tie for most EP is a shared
  victory.
- The bank pays and receives EP. Money is public, though it may be stacked.
- **No player can go below 0 EP.** If a scoring round would take more EP than a
  player has, the excess loss is waived. This caps the downside of every bet.
- EP can be exchanged for equal-value coins and notes at any time; this has no
  effect on the optimal action and is ignored below.

2. Core movement and ranking concepts
-------------------------------------
The board has **16 numbered race-track spaces** (1–16). Racing camels move
clockwise. For racing camels the finish line is immediately after space 16:
a move that would take a camel unit past space 16 ends the race.

**Camel stack and camel unit**

- Camels on the same space always form a stack.
- If a camel in a stack moves, it carries every camel above it and leaves the
  camels below it in place.
- The moving group (a single camel or a stack) is called a **camel unit**.
- If a camel unit ends its movement on a space containing another camel unit, it
  is placed **on top** of that unit.

**Ranking**

Ranking is by board position, then by height in the stack:

1. The camel on the top of the most advanced stack is 1st.
2. The camel directly below it in that same stack is 2nd.
3. Continue down that stack, then move to the top of the next-most-advanced
   stack, and so on.
4. The camel at the bottom of the least advanced stack is last.

Consequences for scoring:

- **Second place can be in the same stack as the leader.** It is not necessarily
  the top camel of the next space.
- The **overall loser** is the least advanced camel; if the last stack contains
  several camels, it is the camel at the bottom of that stack.
- A camel's facing direction never changes; being moved backwards (for example
  by a mirage) changes its position but not its ranking orientation.

3. Setup (decision-relevant parts)
----------------------------------
- Each player starts with **3 EP**, one set of **5 race-betting cards** (one card
  per camel colour), and **one desert tile** (Oasis on one side, Mirage on the
  other).
- The leg-betting stacks are built per camel colour, from top to bottom:
  **5, 3, 2**. The 5-EP tile is taken first, then 3, then 2.
- There are **5 pyramid tickets** per leg.
- The start is determined by rolling the 5 racing dice:
  - each camel goes to space 1, 2, or 3 according to its die;
  - camels sharing a space are stacked in **any order** chosen during setup;
  - all racing camels face clockwise.
- The youngest player receives the leg-starting-player marker and takes the
  first action.

The arbitrary order of the initial stack matters: it determines the initial
1st/2nd/... ranking within that space and therefore the initial leg-bet odds.

4. Turn structure
-----------------
- The player with the leg-starting-player marker acts first; play then proceeds
  clockwise, one action per player per turn.
- On your turn you must choose **exactly one** of the four actions below.
- When a leg ends, the marker moves to the player to the left of the player who
  took the last pyramid ticket. That player starts the next leg.
- There is no fixed number of legs. The race ends when a camel unit crosses the
  finish line (see section 9).

5. Action 1 — Take a leg-betting tile
-------------------------------------
Take the **top** leg-betting tile from any colour's stack and keep it face up.

- The tile backs that camel to be **leading at the end of the current leg**.
- There is no limit to how many leg-betting tiles you may take in a leg, and you
  may take several tiles of the same colour.
- The printed value is fixed: taking a tile early (when 5 is still on top) gives
  a higher payout than taking a lower tile later.

At leg scoring, for each leg-betting tile you own:

| Finishing position of the tile's camel | EP result |
|---|---|
| 1st | **+ printed value** (5, 3, or 2) |
| 2nd | **+1** |
| 3rd–5th | **−1** |

Because the score is per tile, holding several tiles of the same camel sums the
results.

**Expected value of one leg-bet tile**

Let `p1` be the probability that the tile's camel is 1st at leg end and `p2` the
probability that it is 2nd. For a tile printed with value `v`:

```text
EV = v * p1 + 1 * p2 + (-1) * (1 - p1 - p2)
   = (v + 1) * p1 + 2 * p2 - 1
```

A 5-tile is high risk/high reward; a 2-tile has the same −1 downside but a lower
upside. The probabilities `p1` and `p2` come from the current board, the dice
still in the pyramid, and the remaining actions in the leg.

6. Action 2 — Place or move your desert tile
--------------------------------------------
Place your desert tile, or move it if it is already on the board, according to
these restrictions:

- The target space must be **empty**: no camels and no other desert tile.
- It may **not** be space 1.
- It may **not** be adjacent to a space containing another desert tile (either
  neighbouring track space).
- Choose which side is face up: **Oasis** or **Mirage**.
- Moving an already-placed tile follows the same restrictions.

**When a camel unit ends its movement on a desert tile:**

1. The tile's owner immediately receives **1 EP from the bank**.
2. Then the unit moves one additional space:
   - **Oasis:** one space **forward**. If it lands on another camel unit, it goes
     **on top** as usual.
   - **Mirage:** one space **backward**. If it lands on another camel unit, it is
     placed **underneath** that unit (the whole moving unit keeps its internal
     order below the existing stack).

The tile stays on its space until the end of the leg, when it is returned to its
owner. The 1 EP is paid even when an opponent's camel triggers the tile, and it
is paid to the owner regardless of which camel moved.

The no-adjacent-desert restriction means a desert effect can never land on
another desert tile, so effects do not chain. If a unit would cross the finish
line as a result of the extra movement, the race ends.

For action evaluation, a desert tile has two components:

- **Direct income:** +1 EP for every camel unit that ends movement on the tile
  during the rest of the leg.
- **Positional effect:** the extra forward/backward move changes the current leg
  ranking and future race ranking, and therefore the value of leg and overall
  bets (yours and your opponents').

7. Action 3 — Take a pyramid ticket and roll
--------------------------------------------
Take the top pyramid ticket and keep it. Then reveal one random die from the
pyramid and move the matching camel.

- Each die has faces **1, 2, 3 twice**, so each value has probability 1/3 and the
  rolls are independent.
- Move the revealed camel forward by the die value, following the stack rules in
  section 2.
- If the camel unit lands on a desert tile, resolve the desert effect.
- If the unit crosses the finish line, the race ends immediately.
- A revealed die is removed from the pyramid for the rest of the leg.
- Each pyramid ticket is worth **+1 EP at the next leg scoring**, so the ticket
  itself is a guaranteed gain; the movement can also change the value of your
  leg and overall bets, in either direction.

A leg ends after the last pyramid ticket is taken and its die movement is
resolved (all 5 dice in the 2014 game).

8. Action 4 — Bet on the overall winner or loser
------------------------------------------------
Place one of your race-betting cards **face down** on either the overall-winner
space or the overall-loser space.

- A card shows one camel colour.
- Each player has exactly **5 cards, one per camel colour**, so a player can make
  at most five overall bets and can bet on a given camel only once across both
  the winner and loser spaces.
- If cards are already on the chosen space, place yours on top.
- Once placed, a card cannot be moved or taken back.
- If you have no cards left, you cannot choose this action.

**End-of-game scoring**

The winner deck is scored first, then the loser deck. In each deck:

1. Turn the whole deck face up. The first card placed is now on top, the last is
   on the bottom.
2. Walk the cards in placement order.
3. For cards showing the correct camel:
   - 1st correct card: **+8 EP**
   - 2nd correct card: **+5 EP**
   - 3rd correct card: **+3 EP**
   - 4th correct card: **+2 EP**
   - every later correct card: **+1 EP**
4. Every card showing a camel other than the correct one: **−1 EP**.

"Correct" for the winner deck is the camel ranked 1st at the end of the race.
"Correct" for the loser deck is the camel ranked last (bottom of the least
advanced stack).

Betting early on a camel that eventually wins or loses therefore pays much more
than betting late on the same certainty. A wrong overall bet costs only 1 EP (or
less if the 0-EP floor applies).

9. End of a leg and end of the race
-----------------------------------
**End of a leg.** When the last pyramid ticket is taken, its die is revealed and
the camel is moved. Then, before the next player's turn:

1. Pass the leg-starting-player marker to the player on the left of the player
   who took the last pyramid ticket.
2. Determine the ranking (section 2).
3. Score every player's leg-betting and pyramid tiles using the table in
   section 5 and the +1 EP per pyramid ticket.
4. Apply the 0-EP floor.
5. Return leg-betting tiles to their stacks (5, 3, 2 from top to bottom), return
   pyramid tickets and desert tiles, and put the revealed dice back into the
   pyramid. Camels and overall-bet decks stay where they are.

**End of the race.** The race ends immediately when the first camel unit crosses
the finish line, including a crossing caused by a desert tile. Then:

1. Perform one final leg-scoring round using the leg bets and pyramid tickets
   actually held at that moment.
2. Score the overall-winner deck.
3. Score the overall-loser deck.
4. The player with the most EP wins; ties are shared.

The final leg may be incomplete (some dice never revealed); that does not change
the scoring formula.

10. Decision summary
-------------------
When choosing an action, compare the expected change in your final EP:

| Action | Immediate EP | Main effect on future EP |
|---|---|---|
| Take pyramid ticket | **+1** guaranteed at leg end | Moves a camel; changes leg/overall probabilities and may trigger a desert tile |
| Take leg-bet tile | none now | Pays printed value if camel is 1st, +1 if 2nd, −1 otherwise at leg end; EV formula in section 5 |
| Place/move desert tile | none now | +1 EP per future unit landing on it, plus a one-space shift that changes leg/overall rankings |
| Overall winner/loser bet | none now | +8/+5/+3/+2/+1 if correct (earlier is better), −1 if wrong, at game end |

Important interactions:

- A pyramid ticket is the only action that pays a guaranteed +1 EP at the next
  leg scoring; the movement's value depends on which bets you and your
  opponents hold and can outweigh that gain.
- Leg bets are scored every leg and reset, whereas overall bets are scored once
  at the end of the race.
- Desert-tile income is paid to the tile's owner, so an opponent's move can earn
  you EP by landing on your tile.
- Because no player can fall below 0 EP, losses are capped; upside is not.
- Hidden information matters: overall bets are face down, while leg-bet tiles
  and desert tiles are public.

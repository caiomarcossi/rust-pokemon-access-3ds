# 15. Battle HP is said as it changes, and on a key

- **Status:** Accepted
- **Date:** 2026-09-16, replacing "on-demand only"
- **Deciders:** Orion
- **Built**: HP is spoken as it changes, from the gauges, and since 2026-09-29 the two keys say
  it again.

## Decision

- **The player's side is said whole whenever its HP changes**: "Torchic ♀ L5 20/20" — name,
  gender sign, level, current and maximum. In a double battle, each Pokémon on the player's side.
- **The enemy is said the same way but as a percentage**: "Wurmple ♂ L2 80%". This is not a
  softening of the rule, it *is* the rule: ORAS draws the enemy a bar and never a number, so an
  exact figure would be an **Extra**, and Extras ship off (ADR-0017).
- **Not at the moment the battle opens.** Both sides are said once the battle has settled, just
  before "What will X do?".
- **Two keys as well**, for when the speech was missed: own side and enemy side. Not built — they
  have to answer when the HP box is not on screen, which needs the Pokémon's own record rather
  than the box.
- **Never from the HP bar.** The bar drains as an animation, so a poll catches whatever frame it
  is on: one Ember read 48, then 30, then 27 twenty milliseconds later, then 0
  ([findings, spike 54](../findings/battle.md)). The numbers come from the battle's own Pokémon
  structures, where current and maximum are the true values.

Everything else in a battle is carried by the game's own text, so reading text covers it. Automatic
announcements beyond HP stay rare: on faint, and on dropping into the red.

## Consequences

Three things the replays settled, and two left open:

- **An enemy is only an enemy in the HP box's own layout.** The party screen's slot shows a bare
  level and draws HP as three pieces, so it read as a side with no numbers and was announced
  mid-battle as "Wurmple L6 100%".
- **A reader wired to nothing passes every test.** The StrBuf vtable was tracked but never added
  to the set the sweep searches for, so the reader found no battle for a whole replay while 282
  tests passed. Only the replay could show that.
- **Open: the enemy's drop to 0% goes unsaid** — the panel leaves the screen before the value
  holds for a poll. "The wild Wurmple fainted!" covers it.
- **Open: a second battle re-announces only the side whose HP changed.** Saying both would need a
  reliable notion of a new battle.
- **Order is not guaranteed against the dialogue.** The battle reader polls on the screen tick and
  dialogue on the frame tick, so the HP line lands either side of "What will Torchic do?". Both
  are queued, so nothing is cut off; only the order varies. The user accepts this (2026-09-16).

Later corrections: each panel is named from its **gauge** rather than by address order
(2026-09-17), nothing is said while a gauge is still counting, and **a battle is one when a panel
on screen has a gauge leading to a Pokémon** (2026-09-18) — gendered name plates alone left a
genderless battle unread altogether.

**Amended for Sun, 2026-09-27** ([spike 124](../findings/battle.md)). The rule holds as written;
what changes is where each word comes from. Sun writes the name, level and HP into the HP box's
panel as screen text, so the name is the panel's own rather than a name plate's; it draws the
gender sign as a picture whose texture pa3ds cannot read, so the sign is the Pokémon record's,
reached through the gauge as ORAS's is; and the enemy's percentage is the HP its gauge shows over
its battle record's maximum. It was the fill over a full one, 104.8 wide, as ORAS's is over 48,
until it read 62% for 12 of 20 HP: Sun's fill is 5.8 + 99 × HP / maximum wide, keeping a sliver at
0 HP, so its width is not the share (issue 4, 2026-09-27). This is the decision's own rule, the
numbers from the battle's structures rather than the bar, and still a percentage. Sun's gauge is `btl::BtlvGaugeBase`, found by class since
its module's classes are learned as the battle starts, and it keeps the HP shown and the HP it is
counting towards as ORAS's does, so nothing is said until they agree. The panels a gauge shows are
left out of screen text (ADR-0016), where they had been read piece by piece as the count ran.

**Amended 2026-09-29** ([spike 138](../findings/battle.md)). **The open point on a second battle is
closed**: the battle's own start, `btl::BattleMainProc` coming up in the process manager (spike
99), is the reliable notion of a new battle, and the reader starts afresh there, so each battle
says both sides once they settle. The name plates had stood in for it, and keep their text after
a battle. And ORAS's gauges are found by class as the battle builds them, as Sun's are, rather than
as what holds a level TextBox a sweep is asked for once the box is up: that sweep came too late in
a third of battles, and both sides were said after "fight". Both sides now come before
"What will X do?".

**Amended 2026-09-29: the two keys are built**, R for the player's side and shift and R for the
enemy's (Orion's keys). Each says **what was last said for that side** this battle, worded as it
was, and "no battle" out of one. That answers while the HP box is off the screen, in the move list
or the Bag, without the Pokémon's own record this record once thought it needed: the last line
said is the last settled reading, which is what the rule says to speak anyway. The announcer
keeps each panel's line and forgets them all when another app starts.

**Amended 2026-09-29 ([spike 144](../findings/battle.md)): the enemy's drop to 0% is said, and each
side as the hit lands.** The open point above had the wrong cause: the enemy's panel stays on
screen 2.5 s at 0, and it went unsaid because the reader took an HP box's panels only while the
player's panel was on screen too, which it is not while the player's Pokémon attacks. So the
enemy's reading also waited for both sides' turns. A box once known is read without the player's
panel, a panel is taken for the side its gauge says it is on, and each side is now said as the hit
that changed it lands, as Sun's are.

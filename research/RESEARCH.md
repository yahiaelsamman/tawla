# Mahbusa (محبوسة): research notes

Compiled 2026-11-05. Covers the rules, how Mahbusa relates to Plakoto, all strategy material found, and existing computer analysis.

---

## 1. Verdict: Mahbusa is Plakoto

Several independent sources say Mahbusa and Plakoto are the same game:

- **Mamoun's Plakoto primer:** "Plakoto is played in Greece and Turkey, and in Egypt, where it is known as Mahbooseh."
- **Papahristou & Refanidis (2011)**, the researchers behind the strongest Plakoto bot: "Plakoto is also known as 'Tapa' in Bulgaria and 'Mahbusa' in some Arab countries."
- **Wikipedia (Plakoto / Tapa):** "Tapa is also played in Middle East where it is known as Mahbusa."
- **Mamoun's 220-page book** has the subtitle "...Known as Mahbooseh, Tsilido, Tapa or Tavli".

The Arabic and Egyptian descriptions (Jawaker's official rules, Egyptian Streets, Egypt Independent) match Plakoto's rules point for point. The only differences are the small ones in §2.2.

**So Plakoto strategy and computer analysis apply to Mahbusa almost directly.**

---

## 2. Rules

### 2.1 The rule set we will implement (Egyptian Mahbusa, confirmed by Yahia)

| # | Rule |
|---|---|
| Board | 24 points in 4 quadrants of 6. Two players, 15 checkers each, two dice. |
| Setup | All 15 checkers on the player's own 24-point, which is the opponent's 1-point (ace point). The players move in opposite directions. Your home quadrant (points 1–6) is the quadrant where the opponent starts. |
| Start | Each player rolls one die and the higher roll goes first; ties re-roll. The starter then rolls both dice for the first move, so an opening double is possible (confirmed, as in Plakoto). After the first game, the previous winner starts. |
| Movement | Each die is a separate move. Moving one checker with both dice requires the intermediate landing point to be open. |
| Open point | Empty, holding your own checkers, or holding exactly **one** enemy checker. A point is **blocked** if it holds 2+ enemy checkers, or one enemy checker that is pinning one of yours. |
| Doubles | Played four times. |
| Forced moves | You must use both dice if possible (all four for doubles). If only one die can be played, you must play the larger one if possible. If nothing can be played, you lose the turn. |
| Pinning (الحبس) | Landing on a lone enemy checker pins it underneath. A pinned checker cannot move until **every** opposing checker on top has left. The pinner may pile on more checkers. A point where a pinner sits cannot be pinned in turn. There is no hitting and no bar. |
| Mother checker (مسك الحجر, "the mother") | If the opponent pins your last checker while it's still on your starting point, the game ends immediately and you **lose double** (2 points). There is no exception (confirmed). |
| Bearing off | Allowed only when all 15 checkers are in your home quadrant **and** none of them is pinned there. A die removes a checker from the matching point. If that point is empty, you must move a checker from a higher point if you can. If no checker is on a higher point, you remove one from your highest occupied point. Bearing off is never forced if a legal move inside home exists. |
| Scoring | Win = 1 point. **Mars (مارس)** = 2 points, when the loser has borne off no checkers. Mother pin = 2 points. No triple wins, no doubling cube. |
| Match | In Egypt a set is usually first to 3 points (Egyptian Streets). Greek Tavli plays to 3, 5 or 7. |

### 2.2 Where Plakoto and the Arabic sources differ (resolved 2026-11-05)

| | Plakoto (Greek: bkgm, Wikipedia, Mamoun, Palamedes) | Arabic sources (Jawaker) | Impact |
|---|---|---|---|
| **A. First move** | The winner of the one-die roll then **rolls both dice again**, so an opening double is possible. After the first game, the previous winner starts. | Says who starts by one die each; doesn't say whether that roll is played or re-rolled. | Small. It changes first-player advantage, not strategy. |
| **B. Mother exception** | Pinning the mother does **not** end the game if the pinner still has checkers on their own start. If both mothers are pinned, it's a **draw**. | Pinning the last checker on the start wins immediately; no exception mentioned. | Medium. It only matters early, while both sides still have checkers on their starts, but that's exactly when mother pins happen. |
| **Tapa variant** | Bulgarian Tapa: no mother rule, and checkers are optionally brought in gradually. | Not mentioned | The Egyptian game has the mother rule, so it's Plakoto, not Tapa. |

**Decision:** A as in Plakoto (re-roll both dice). B is the Egyptian version: any mother pin wins immediately and double, with no exception. The Plakoto exception is kept as an engine option (`mother_exception`) for comparison, but it rarely matters.

---

## 3. Strategy: everything found

### 3.1 Sources that exist

| Source | Type | Access |
|---|---|---|
| J. Mamoun, *Plakoto: Introduction, Rules and Basic Strategy* | Free public-domain primer, 10 diagrams | ✅ read in full (`sources/PlakotoBoardGameStrategy.pdf`) |
| J. Mamoun, *Plakoto Board Game Strategy* (2017) | 220 pages: annotated game, 25+ problems, dice probabilities | ❌ paid book, not online |
| N. & B. Tzannes, *Backgammon Games and Strategies* (1977), p. 82 | Book chapter with a sample game | ❌ out of print |
| N. Frantzis, *The Seven Popular Games of Backgammon* (1979), p. 42 | Book with strategy discussion | ❌ out of print |
| Papahristou & Refanidis, 6 papers (2011–2024) | Computer analysis | ✅ read (`sources/*.pdf`) |
| tavli.net, boardgammon.com | Short web strategy tips | ✅ read |
| Arabic web (mosoah, kuwaitigames, maqall, Jawaker) | Generic tips only | ✅ nothing beyond "build blocks, trap checkers" |

**No serious Arabic strategy material exists online.** The real knowledge base is Greek/English plus the Palamedes computer analysis.

### 3.2 Strategic principles from the sources

**Opening: unstack and build safe blocks**
1. Your 15-checker starting stack is your **biggest liability**: a pinned mother almost always loses double. Empty the start "as early as practical". Also clear the three points right in front of it early (your 23-, 22- and 21-points, which Mamoun numbers 2, 3 and 4 from the start), because a pin there is nearly as deadly. *(Mamoun)*
2. Drop single "builder" checkers into your first quadrant and cover them next turn to make blocks. The risk is small because the opponent's checkers are far away and need a lucky high double to reach them. The benefit is worth it. *(Mamoun)*
3. Palamedes and expert players use a **"restrained aggressive"** opening: leave lone checkers when the chance of them being pinned is small. Don't hoard checkers in big inflexible stacks. *(Papahristou 2013)*
4. Make several blocks in a row in your first quadrant, then gradually shift them out into your second quadrant. *(Mamoun)*
5. Aim to have checkers on at least 5–6 different points by the middle game. *(tavli.net)*

**Attack: pin early in the opponent's journey**
6. **The closer a pin is to the opponent's starting point, the stronger it is.** The best pins sit just in front of the opponent's start, on your 2-, 3- and 4-points (the opponent starts on your 1-point). They are far stronger than pins near your outer board. With several pins, prioritise the ones furthest back. *(Mamoun, tavli.net)*
7. Make an **advanced block** near the opponent's start so a checker from it can jump onto any blot they leave. This is why **6-6 is the best opening roll**. *(Mamoun, Palamedes rollouts)*
8. A pin is a **cheap block**: one checker does the job of two. A row of pins and blocks forms a **prime**, and a 6-point prime can't be passed. *(Mamoun)*
9. **Pile checkers onto a deep pin.** The more checkers stacked on top, the longer before you're forced to release it. When two players each hold a pin, the one with more checkers stacked on the deeper pin usually wins the "who must release first" battle. *(Mamoun, Figs 7–8)*
10. **Bear-off with a pin in your home board:** stack your checkers on the pinned point to make the pin last as long as possible. This is how pins turn into **double wins**. Palamedes's bot got noticeably stronger once it learned this (feature `ChFrontOfPin`). *(Papahristou 2012)*

**Timing: "trap early, release late"**
11. Freeing a pinned checker too early lets the opponent back into the game. *(tavli.net, boardgammon)*
12. Release rule of thumb: release when you have enough advanced checkers to bear off before the freed checker can catch up. If the pin is in your home board, it's often worth holding it into the bear-off. *(tavli.net)*
13. **Timing decides the value of a block.** If your other checkers will soon run out of moves and force you to break it, the block isn't worth much. The side that can wait without wrecking its structure usually wins. *(Mamoun, boardgammon)*

**Defence**
14. Every blot in the opponent's path is a potential "lifetime prisoner", not a temporary setback as in normal backgammon. *(boardgammon)*
15. The main error of early bots: when forced to leave a blot, they didn't account for **how many dice rolls (out of 36) pin it**. Adding exactly that number as an input was the single biggest improvement to the Plakoto AI. **Counting pin shots is the key tactical skill.** *(Papahristou 2011)*

### 3.3 Hard numbers from the computer analysis (Palamedes bot, 100,000-game rollouts)

- **Double wins are very common:** 40.5% of Plakoto games end in a double win, versus 27% in normal backgammon. Pins inside the home board are the cause.
- **First-player advantage:** the player who moves first wins **52.7%** of games (+0.104 points per game). All of that edge comes from double wins: 22.8% vs 17.7%. Single wins are about even.
- **Starting position:** spreading the 15 checkers over the first three points (Tapa) cuts double wins to 24.7%. This confirms the stacked start is what makes Plakoto/Mahbusa so swingy.
- **Match play:** when you're ahead, play safe; when you're behind, take risks to win double. Playing for the match instead of each game raised match wins only slightly (51%), and the "match" player won fewer points but more matches.

### 3.4 Opening book (Palamedes rollouts, Plakoto rules)

Notation: the **start point is 24**; "24/21" means move one checker from the start to point 21 (3 steps). (2) means two checkers make the same move. EQ = expected points per game for the first player; higher is better.

**The rule for every non-double: move two separate checkers off the start, one per die.** Never run one checker with both dice.

| Roll | Best move | EQ | | Roll | Best move | EQ |
|---|---|---|---|---|---|---|
| 2-1 | 24/22 24/23 | 0.042 | | 6-2 | 24/18 24/22 | 0.106 |
| 3-1 | 24/21 24/23 | 0.037 | | 4-3 | 24/20 24/21 | 0.056 |
| 4-1 | 24/20 24/23 | 0.070 | | 5-3 | 24/19 24/21 | 0.039 |
| 5-1 | 24/19 24/23 | 0.043 | | 6-3 | 24/18 24/21 | 0.096 |
| 6-1 | 24/18 24/23 | 0.097 | | 5-4 | 24/19 24/20 | 0.073 |
| 3-2 | 24/21 24/22 | 0.050 | | 6-4 | 24/18 24/20 | 0.121 |
| 4-2 | 24/20 24/22 | 0.065 | | 6-5 | 24/18 24/19 | 0.117 |
| 5-2 | 24/19 24/22 | 0.066 | | | | |

| Double | Best move | What it builds | EQ |
|---|---|---|---|
| 1-1 | 24/23 (4) | 4-checker block on 23 | 0.129 |
| 2-2 | 24/20 24/22 (2) | block on 22 plus a builder on 20 | 0.137 |
| 3-3 | 24/18 24/21 (2) | block on 21 plus a builder on 18 | 0.187 |
| 4-4 | 24/16 (2) | advanced block on 16 | 0.247 |
| 5-5 | 24/14 24/19 (2) | block on 19 plus a runner on 14 | 0.361 |
| 6-6 | 24/12 (2) | advanced block on 12 | **0.521** |

The best opening rolls are 6-6, 5-5, 4-4 and 3-3. The weakest are 3-1, 5-3 and 2-1.

**Not covered anywhere:** the second player's best replies, and second/third-move theory.

---

## 4. Existing software

- **Palamedes** (N. Papahristou, University of Macedonia), now distributed as **Polygammon** (Windows) and **HappyGammon** (Android, Google Play). It's a free, expert-level Plakoto AI that can analyze positions. It won the 2011 Computer Backgammon Olympiad (Portes). Free to use but **not open source**: redistribution needs the author's approval. https://nikpapa.com/Palamedes, https://github.com/nikpapa/palamedes (papers only).
- Its Plakoto bot was trained with TD(0) self-play: 1 hidden layer of 100 neurons, 3 outputs (win, win double, lose double), about 5 million games to peak strength, plus pin-endgame databases.
- Tavli3D: an older open-source program. Palamedes beat it easily after a few thousand training games.

---

## 5. What exists vs. what's missing for a human strategy guide

| Need | Status |
|---|---|
| Exact rules | ✅ complete (two minor open questions, §2.2) |
| Opening moves (first player) | ✅ computer-verified table (§3.4) |
| General principles | ✅ about 15 principles (§3.2), mostly qualitative |
| Second-player replies / early-game theory | ❌ missing |
| **Numbers behind the principles**: how much a pin on point X is worth; when to release; how many checkers to pile on; when to break the start; how much blot risk to accept | ❌ missing; this is where our own engine and bot add value |
| Effect of the Egyptian mother rule (no exception) | ❌ untested |
| Bear-off and pin-endgame technique | ⚠️ known to the bot (databases) but not written up for humans |

---

## 6. Recipe for our own neural-network bot (if we build one)

From the published papers, so this is a proven path rather than an experiment:

- **Network:** inputs = board encoding + about 10 expert features. 1 hidden layer (100 sigmoid units). 3 outputs: P(win), P(win double), P(lose double). Value = 2·W − 1 + WD − LD.
- **Training:** TD(0) (λ = 0; λ > 0.6 diverged for Plakoto). Self-play, updating after each game from the last position back to the first. Learning rate 0.3 → 0.1 → 0.01. Peak strength at about 5 million games.
- **Most important input:** for each of your blots, the probability (out of 36 rolls) that the opponent can pin it next roll.
- **Second most important:** how many of your checkers are "in front of" your pin in your home board (teaches stacking on pins during bear-off).
- **Learning speed:** raw inputs alone reached "average human" level within a few thousand games. With the pin-probability feature it reached "experienced player" level.
- **Search:** a 2-ply look-ahead adds about +0.07 points per game.

---

## Sources

Local copies are in `research/sources/` (git-ignored).

- Jawaker, قوانين محبوسة: https://blog.jawaker.com/mahboseh-rules/
- Egyptian Streets, "Sheesh Beesh: Egypt's 5 Most Common Tawla Games": https://egyptianstreets.com/2022/09/10/sheesh-beesh-egypts-5-most-common-tawla-games-and-how-theyre-played/
- Egypt Independent, "Tawla: A tale of strategy": https://www.egyptindependent.com/tawla-tale-strategy/
- Wikipedia, Plakoto: https://en.wikipedia.org/wiki/Plakoto
- Backgammon Galore, Plakoto: https://bkgm.com/variants/Plakoto.html
- J. Mamoun, Plakoto primer: https://bkgm.com/articles/Mamoun/PlakotoBoardGameStrategy.pdf
- J. Mamoun, book listing: https://bkgm.com/books/Mamoun-Plakoto.html
- V. Condos, Tavli: https://bkgm.com/articles/Condos/Tavli/
- Tzannes rules excerpt: https://jpneto.github.io/world_abstract_games/rognlie/plakoto.html
- BrainKing rules: https://brainking.com/en/GameRules?tp=109
- tavli.net, Plakoto rules and strategy: https://tavli.net/en/odigoi/plakoto-kanones/
- Boardgammon, Plakoto: https://boardgammon.com/backgammon/variants/plakoto/
- Papahristou & Refanidis papers (all six): https://github.com/nikpapa/palamedes/tree/main/papers
  - Training Neural Networks to Play Backgammon Variants Using RL (EvoApplications 2011)
  - On the Design and Training of Bots to Play Backgammon Variants (AIAI 2012)
  - Improving TD learning performance in backgammon variants
  - Opening Statistics and Match Play for Backgammon Games (2013)
  - Constructing Pin Endgame Databases for Plakoto (ACG 2015)
  - Constructing effective match strategies for the game of Tavli (SETN 2024)

# Mahbusa (محبوسة) Strategy Guide, v2.3

*v2.3, 10 November 2026. v1 collected everything already known: the Palamedes computer analysis, expert Plakoto writing, and dice maths. v2 adds results from **our own Mahbusa bot**, a neural network trained on 2 million self-play games under Egyptian rules.*

Every claim carries a tag showing how solid it is:

| Tag | Meaning |
|---|---|
| **[Palamedes]** | Measured by Palamedes, a world-class Plakoto program (100,000 playouts per position). |
| **[Our bot]** | Measured with our own neural network (2M self-play games, Egyptian rules). It reproduces Palamedes's opening results within statistical error. It is strong, not perfect: it looks one move ahead, and outside the opening it has not been checked against another strong program (see `docs/METHODS.md`). |
| **[Expert]** | Advice from experienced players and writers (Mamoun, tavli.net, Tzannes and others). Not measured. |
| **[Math]** | Follows directly from the rules and the dice. |
| **[To test]** | A reasonable idea that nobody has measured yet. Our own bot will check it. |

---

## 0. Notation

Points are always numbered **from your own point of view**:

- You start with all 15 checkers on your **24-point** and move down: 24 → 13 → 12 → 1, then bear off.
- Your **home** is points 1–6. Your **first quadrant** (where you start) is 19–24.
- The opponent starts on **your 1-point**, inside your home, and travels the other way. Their home is your 19–24, exactly where you start.

```
        your first quadrant                 
 13 14 15 16 17 18 | 19 20 21 22 23 24
                   |                 ●●●  ← you: 15 checkers (start)
                   |
                   |                 ○○○  ← opponent: 15 checkers (start)
 12 11 10  9  8  7 |  6  5  4  3  2  1
                        your home
```

Moves are written *from/to*. "24/21" means one checker from 24 to 21. "(2)" means two checkers make the same move.

Words used below:

- **Blot:** a lone checker; it can be pinned.
- **Block:** 2+ of your checkers on a point, or a point where you pin an enemy checker. The opponent cannot land on a block.
- **Pin:** your checker sitting on top of a trapped enemy checker.
- **Mother:** the last checker left on a starting point.
- **Mars:** a double win (2 points).

---

## 1. What makes Mahbusa different

1. **Nothing is ever sent back, but a pinned checker can be stuck almost for good.** In regular tawla a hit costs you some distance. In Mahbusa a pin can freeze a checker for most of the game, and you can't bear off until it is free. [Math]
2. **Mars happens a lot.** About **40% of games end in a double win**, compared with about 27% in regular backgammon. Almost every mars comes from a pin deep in someone's home board. [Palamedes; our bot measured 40–43%]
3. **The first player has a small edge.** They win about **52.7%** of games. All of that edge comes from double wins (22.8% vs 17.7%). Single wins are split almost evenly. [Palamedes; our bot measured +0.11 points per game for the first player]
4. **Pinning the mother wins at once, and double.** Your 15-checker starting stack is your biggest weakness until it's gone. [Math, Expert]

---

## 2. The big ideas (cheat sheet)

1. **Get off the start quickly, putting your blots where the opponent can't reach them** (see §4). [Palamedes, Our bot, Math]
2. **Build blocks in your first quadrant, then slide them forward.** Several blocks in a row are a wall. [Expert]
3. **The best pin is one right in front of the opponent's start**: your 2-, 3- and 4-points. Pile checkers onto it. [Expert]
4. **Count the pin shots before you leave any blot.** Know how many of the 36 rolls pin it (table in §4). This is the most important tactical skill. [Palamedes, Our bot, Math]
5. **Trap early, release late.** Hold a deep pin into your bear-off, and stack checkers on the pinned point to make it last. That turns pins into mars. [Palamedes, Our bot, Expert]
6. **One early pin is worth about a whole point**; on your 2- or 3-point, more like a mars. Take a deep pin when it's offered, even at the cost of a block (§6, §11). [Our bot]
7. **Never lift a home pin on purpose.** Let the dice force you. The one exception: trade a pin on your 5 or 6 for a deeper one on 2–4 (§7). [Our bot]
8. **Never leave exactly one checker on your start while anything can still reach it.** Move the last two together (§5, §11). [Our bot, Math]
9. **Pinned yourself? Get a counter-pin.** Players who had one won twice as often (§8), and the example in §11 shows a swing of 2 points. [Our bot]
10. **Calculated risk beats safety.** Refusing every blot that 1 roll in 3 could pin costs a third of a point per game (§12). [Our bot]
11. **Depth beats numbers.** In a pin standoff, a pin one point deeper is worth more than six extra checkers on yours (§6). [Our bot]

---

## 3. The opening: your first roll

The best first move for every roll, from 100,000-game computer playouts: [Palamedes]

### Non-doubles: split two checkers off the start, one per die

| Roll | Play | | Roll | Play |
|---|---|---|---|---|
| 2-1 | 24/23 24/22 | | 6-2 | 24/18 24/22 |
| 3-1 | 24/23 24/21 | | 4-3 | 24/21 24/20 |
| 4-1 | 24/23 24/20 | | 5-3 | 24/21 24/19 |
| 5-1 | 24/23 24/19 | | 6-3 | 24/21 24/18 |
| 6-1 | 24/23 24/18 | | 5-4 | 24/20 24/19 |
| 3-2 | 24/22 24/21 | | 6-4 | 24/20 24/18 |
| 4-2 | 24/22 24/20 | | 6-5 | 24/19 24/18 |
| 5-2 | 24/22 24/19 | | | |

**The rule:** never run one checker with both dice on the first move. Always put down two builders.

### Doubles

| Roll | Play | Idea |
|---|---|---|
| 1-1 | 24/23 (4) | A safe 4-checker block next to the start |
| 2-2 | 24/22 (2), 24/20 | Block on 22 plus a builder on 20 |
| 3-3 | 24/21 (2), 24/18 | Block on 21 plus a builder on 18 |
| 4-4 | 24/16 (2) | An advanced block on 16 |
| 5-5 | 24/19 (2), 24/14 | Block on 19 plus a runner on 14 |
| 6-6 | 24/12 (2) | An advanced block on 12. **The best roll in the game.** |

**How good is each roll?** In expected points for the first player [Palamedes]:

- 6-6 +0.52, 5-5 +0.36, 4-4 +0.25, 3-3 +0.19, 2-2 +0.14, 1-1 +0.13
- Best non-doubles: 6-4 +0.12, 6-5 +0.12, 6-2 +0.11
- Weakest: 3-1 +0.04, 5-3 +0.04, 2-1 +0.04

**Why these moves are right** [Math]: on move one the opponent's entire army sits on your 1-point. From there the only way to reach your first quadrant is a big double (see the table in §4). Points **18, 20, 22 and 23 can't be reached at all**. Points 19 and 21 can only be reached by one roll each (6-6 and 5-5). So the builders cost almost nothing, and next turn you can cover them and make blocks.

**Our bot's check** [Our bot]: rolling out every first move (4,000 games for each of the top 3 candidates) gives the same best move as Palamedes for **19 of the 21 rolls**, with almost identical values. The bot's overall first-player edge is +0.11, against Palamedes's +0.104. For the other two rolls the alternatives are essentially tied, so **either play is fine**:

| Roll | Book play | Equally good alternative |
|---|---|---|
| 3-1 | 24/23 24/21 | 24/20 (one checker) |
| 2-2 | 24/22 (2), 24/20 | 24/22 (4), which our bot rates slightly higher |

### Replying as the second player [Our bot]

The same rule holds for the reply. **For 302 of 315 non-double replies (96%), the bot splits two checkers off the start** (by its one-move look-ahead, no playouts), one per die, exactly like an opening move. The exceptions are instructive.

**1. With a runner facing you, small rolls may run instead of split.** After most openings that bring an opponent checker forward (6-2, 6-3, 6-4, 6-5, 6-6, 3-3, 5-5), the bot plays a **2-1 reply as 24/21 with one checker, not 24/22 24/23**. Example: against a 6-x opening the opponent has a checker on your 7-point. From there your 22 and 23 are 15 and 16 away (hit by 5-5 and 4-4), but your 21 is 14 away, which **no roll can hit**. A few 3-1 replies are played 24/20 for similar reasons. *Count the shots from their runner, and put your checkers where it can't reach them.* [Our bot, Math]

**2. Punish an exposed builder with a double.** The opening book leaves builders on 19 and 21 because only one roll hits each. If you're the one who rolls it, take it. **Each of these pins is worth about +0.9 points** (by the bot's one-move look-ahead):

| Opponent's builder on their... | That is your... | Your roll | Play |
|---|---|---|---|
| 21-point (after 3-1, 3-2, 4-3, 5-3, 6-3) | 4-point | 5-5 | 24/4 with one checker (pin) |
| 19-point (after 5-1, 5-2, 5-3, 5-4, 6-5) | 6-point | 6-6 | 24/6 with one checker (pin) and 24/18 |

**3. Doubles as a reply.** 1-1: 24/23 (4). 2-2: 24/22 (4). 3-3: 24/21 (4) or 24/21 (3) 21/18. 4-4: usually 24/16 (2). 5-5: usually 24/14 (2). 6-6: 24/12 (2). Always check the pins in point 2 first.

### Later in the opening

The computer tables stop after the reply, so this part combines expert advice with what we watched our bot do [Expert / Our bot]:

- **Cover builders to make blocks**, ideally next to each other: 23–22, 22–21, 21–20…
- **Keep unstacking the start**, ideally to points the opponent can't reach yet.
- **Watch the opponent's runners.** As soon as an opponent checker leaves their start and comes round toward your side, your first-quadrant blots become reachable by ordinary rolls, not just doubles. Recount the shots every turn.
- **If you roll 6-6 or 4-4 early**, consider an advanced block: 24/12 (2) or 24/16 (2). It is a launching pad for pinning the opponent's blots later. [Expert]

---

## 4. Counting pin shots: the key skill

**Who can pin your blot?** Only an opponent checker that is *below* it in your numbering, because the opponent moves upward from your 1-point toward your 24. The distance is your blot's point minus their checker's point. [Math]

**How many of the 36 rolls hit a given distance** (assuming nothing blocks the way) [Math]:

| Distance | Rolls / 36 | Chance | | Distance | Rolls / 36 | Chance |
|---|---|---|---|---|---|---|
| 1 | 11 | 31% | | 7 | 6 | 17% |
| 2 | 12 | 33% | | 8 | 6 | 17% |
| 3 | 14 | 39% | | 9 | 5 | 14% |
| 4 | 15 | 42% | | 10 | 3 | 8% |
| 5 | 15 | 42% | | 11 | 2 | 6% |
| **6** | **17** | **47%** | | 12 | 3 | 8% |
| 13, 14 | 0 | 0% | | 15, 16, 18, 20, 24 | 1 | 3% |
| 17, 19, 21–23 | 0 | 0% | | | | |

**Rules of thumb** [Math]:

- **Within 6 points is dangerous** (31–47%). **7–12 points is moderate** (6–17%). **Beyond 12, only one specific double can hit, if any.**
- **6 points away is the worst distance** (17 rolls), not 1.
- Your **blocks cut the shots**. A roll that needs to stop on one of your blocks on the way doesn't count. A wall of blocks in front of a blot makes far-away shots nearly impossible.
- When you must leave a blot, compare the shot counts for each option and **take the lowest**. Palamedes's early bots lost exactly here: they didn't count the shots. Adding this one number made them much stronger. [Palamedes]

**Opening safety map**: your blots on move one, against the opponent's stack on your 1-point [Math]:

| Your point | 23 | 22 | 21 | 20 | 19 | 18 | 17 | 16 | 15 | 14 | 13 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Rolls that pin it | — | — | 5-5 | — | 6-6 | — | 4-4 | 5-5 | — | — | 3-3, 4-4, 6-6 |

**How much risk does a strong player take?** [Our bot] After each move, we measured the most-hittable blot the bot left (20,000 self-play games):

| Bot's own turn | Typical worst blot | Moves leaving a blot hit by 6+ rolls (17%+) |
|---|---|---|
| 1–5 | 2.5 rolls / 36 | 11% |
| 6–10 | 3.4 / 36 | 20% |
| 11–15 | 5.3 / 36 | 27% |
| 16–20 | 6.7 / 36 | 32% |
| 21+ | 2.0 / 36 | 11% |

So: **be very safe in the opening.** The typical worst blot in the opening can only be hit by a double or two. Risk climbs in the middle game (turns 11–20), when checkers have to come around the board and some exposure can't be avoided. Then it drops again in the bear-off.

---

## 5. Guarding your mother

- Being pinned on the start **loses double at once**. Being pinned just in front of it (your 21–23) is nearly as bad, because the checker is trapped at the very beginning of its journey. [Expert]
- **Empty the start "as early as practical"**, and also move off 21–23 once you have blocks further along. [Expert]
- **Be careful when you're down to your last checker on 24.** The mother is in danger only once exactly one checker is left there. Before leaving one checker on 24, count the shots at it from every opponent checker below it. Usually it's better to leave **two or none**. [Math, Our bot]
- **What a lone mother costs** [Our bot]: we played out the same position with zero, one or two checkers left on the start, and an opponent runner at different distances (3,000 games each):

  | Rolls that can pin your lone mother | Cost of leaving one checker instead of two or none |
  |---|---|
  | 15 (runner 6 pips away) | **−1.07 points** |
  | 12 (runner 2 pips away) | −0.85 |
  | 2 | −0.10 to −0.19 |
  | 0–1 | under 0.1 |

  **Rule of thumb: a lone mother costs about 0.07 points for every roll that can pin it.** Leaving **two** was never worse than leaving none, so you don't need to rush the last pair off.

  This is just arithmetic: a pinned mother turns your current prospects into −2, so each pinning roll costs (2 + your expected result) ÷ 36. That is 0.06 in an even game and 0.07 when you are ahead, and our playouts match. [Math]
- Your mother is safe once the opponent has no checkers left that could still reach your 24 (they're all past it or pinned). [Math]
- **How fast?** Our bot empties its start by its **13th turn** on average (half of games between turns 11 and 15), and in 97% of games it gets the start empty. Between strong players only **4–5% of games end with a mother pin**. With random play it's about 70%, so guarding the mother is a skill that clearly pays. [Our bot]

---

## 6. Pinning: where, when and how much

**Where.** The closer a pin is to the opponent's start, the more it's worth. A pin on your **2-, 3- or 4-point** holds the checker at the very start of its 23-point journey home, which is often enough to win a mars. A pin on your 10-point is much weaker. [Expert]

**How much is a pin worth?** [Our bot] We set up the start position with one extra feature: the pinner has a single checker pinning one enemy checker on point p. Then we played 4,000 games from each position. These positions are artificial (the pinning checker is simply placed there, which also gains it up to 22 pips), so read the values as a guide to relative worth, not exact prices:

| Pin on your point | Value of the pin (points) | Pinner wins a mars |
|---|---|---|
| 2 | **+1.6** | 87% |
| 3 | +1.3 | 70% |
| 4 | +1.1 | 66% |
| 5–7 | +1.0 | 62–63% |
| 8–13 | +0.85 to +1.0 | 52–60% |
| 14–17 | +0.60 to +0.81 | 34–45% |
| 18–23 | +0.53 to +0.60 | 31–38% |

- **One early pin is worth about a whole point.** That's 5–15 times the first player's advantage. Even a shallow pin near the opponent's home (+0.5) is a big deal.
- Value rises steeply toward your 1-point: **a pin on 2 or 3 is often a mars on its own.**
- Pins on 5–7 are worth slightly more than you'd expect from distance alone. Those points sit right in front of your home and also act as blocks.

**What happens to pins in real games** (20,000 bot-vs-bot games) [Our bot]:

| Pin on point | How often (per game) | Average time held (turns, both players) | Pinner wins | Pinner wins a mars |
|---|---|---|---|---|
| 2 | 0.05 | 71 | **93%** | 83% |
| 3 | 0.14 | 65 | 81% | 64% |
| 4 | 0.40 | 58 | 69% | 52% |
| 5–12 | 0.3–0.7 each | 20–42 | 62–70% | 37–46% |
| 13–23 | 0.02–0.4 each | 10–21 | 62–77% | 26–62% |

Deep pins (2–4) are rare because strong players rarely leave a blot there, but they're held for most of the game and decide it. Pins on 4–12 are the everyday weapon.

**When you hold several pins**, protect the deepest one first, the one nearest your 1-point. [Expert]

**A pin is a cheap block.** One checker does the job of two. A run of pins and blocks forms a **prime** (a wall). Six blocks in a row can't be passed at all. [Expert]

**Piling on.** Every extra checker you stack on a pin is one more turn before you're forced to lift it. [Expert] Measured: at equal depth, **each extra checker on your pin (compared with theirs) is worth about +0.13 points** (see the standoff table below). [Our bot]

**The release race.** When both players hold a pin, the game often comes down to who is forced to release first. Count which rolls force each side to move a pinning checker. That happens when you have no other legal moves, especially once your other checkers are all home. The side with **more checkers piled on the deeper pin** usually wins the race. [Expert]

**The standoff, measured** [Our bot]: both players pin each other in their home boards and everything else is home. We changed only the depth of each pin and how many checkers sit on it (2,000 games per position, you on roll):

| Situation | Your equity |
|---|---|
| Equal depth (both on the 4-point), equal stacks | about −0.1 (having to move first hurts slightly) |
| Equal depth, you 7 checkers on your pin, they 1 | **+0.71** |
| Equal depth, you 1, they 7 | **−0.88** |
| Your pin on 3, theirs on their 4 (you're deeper), equal stacks | about **+0.8 to +1.0** |
| Your pin on 4, theirs on their 3 (they're deeper), equal stacks | about **−1.0** |
| They're deeper, but you have 7 on your pin against their 1 | still **−0.54** |

- **Depth beats numbers.** One point of depth is worth about a full point, more than six extra checkers on the shallower pin.
- Among equal-depth pins, **more checkers on your pin wins the race**: roughly +0.13 per checker of difference.
- **Moving first in a standoff is a small disadvantage**, because the player who runs out of spare moves first must lift.

**Timing.** A block or pin is only worth as much as you can afford to keep it. If your other checkers will soon run out of moves and force you to break the wall, it's worth much less. **The side that can wait without wrecking its position usually wins.** [Expert]

**Advanced blocks.** A block near the opponent's start (for example your 12- or 13-point) is a launching pad. When the opponent leaves a blot in their first or second quadrant, a checker from your block jumps on it. [Expert]

---

## 7. Lifting a pin: the rules

We collected **134,000 decisions from 8,000 bot games** in which the bot held a pin in its home board and could either keep it or lift it. For each one we compared the best "keep" move with the best "lift" move. [Our bot]

These percentages use the bot's one-move look-ahead judgement of each move. We checked that judgement against playouts (1,000 full games per move) on 300 sampled decisions: where the one-move value says *keep*, playouts agreed in 91 of the 92 cases they could decide (the other 8 were too close to call); where it says *lift* by a clear margin (0.25 points or more), playouts agreed in all 18 cases they could decide; for small margins it was right about 3 times in 4. The rule as a whole was also confirmed in full games (§12).

### Rule 1: don't lift a home pin while any other move exists

- Lifting was the better move only **3% of the time**.
- Simply **never lifting agrees with the bot 97% of the time** and costs on average 0.004 points per decision. Lifting when you shouldn't costs a median **0.4 points** each time.
- In practice, nearly all of the bot's releases are **forced**: the dice leave no other move. Before its bear-off it keeps the pin 97% of the time it could lift it, and during the bear-off still 85–95%.
- Plan for it: keep spare checkers that can absorb awkward rolls, so the dice never force you off the pin (see "stack on the pin" below).

### Rule 2: the one exception worth thinking about is trading up

If your pin is on your **5- or 6-point** and this roll lets you make a **new, deeper pin (on your 2-, 3- or 4-point)**, then lift-and-re-pin is a real option, but only an option. By the bot's one-move value it was right **41% of the time** (46% when the new pin is on your 4-point). When we rolled out 100 sampled cases, lifting came out ahead in 53 and the average difference was a wash (+0.02 points in favour of keeping the old pin), with about as many cases clearly favouring each side. And in full games, letting the bot make this exception did not measurably beat never lifting at all (−0.022 against −0.027 per game, ±0.009, §12). So: **look at it when a deep pin is on offer, count the shots, and don't expect a big gain either way.** Trading a pin for one at the same depth or shallower was right only 6% of the time (one-move value).

Example: you roll **5-3**. You pin on your 5-point (`1X/o`), and the opponent has a lone checker on your 2-point, right in front of their start:

```
   13    14    15    16    17    18 |   19    20    21    22    23    24
    .  1O/x     .     .    2X    3X |   4X     .     .     .     .    4X
   2O     .    2O     .    3O    1O |    .  1X/o     .     .    1O    4O
   12    11    10     9     8     7 |    6     5     4     3     2     1
X off:  0  pips: 279    O off:  0  pips: 275    
```

- **Best: 24/19 5/2**: lift the pin on 5 and use the 3 to pin on 2 (**+1.61**, confirmed with 1,500 playouts per move).
- Keeping the pin (24/19 24/21) scores about **0.00**. The deep pin on your 2-point is worth far more than the shallow one on 5 (§6).

### When never to lift

| Situation | Lifting right |
|---|---|
| The pinned checker is the opponent's **last checker outside their home** | 0.3% |
| The pin is on your **2- or 3-point** | 0.4% |
| **9 or more** of your checkers are still outside your home | 0.8% |
| You're **far ahead in the race** (8 or fewer of yours outside home, lead of 45+ pips) | 11–12%. Still keep it: lifting just costs less here |

The expert rule of thumb "release when you're sure to bear off first" ([Expert]) is too generous. Even with a big race lead the bot keeps the pin almost every time, because a held pin also gives mars chances, and keeping it costs almost nothing.

### Bearing off with a pin in your home: stack on the pin

While you hold a pin on, say, your 3-point, your checker there is the last thing you want to move. Bring your other checkers in and **pile them onto the pinned point**, and bear off from the other points first. The pinned enemy checker stays stuck until the very end of your bear-off. Then it still has to travel your whole board and theirs before the opponent can bear off anything. That's how a pin becomes a **mars**.

Palamedes's bot originally bore off "normally" in these positions. It got clearly stronger once it learned to stack on the pin. [Palamedes]

Measured with the same pip count, the pinned opponent checker being their only one outside home (8,000 games each) [Our bot]:

| Your pin | 5 checkers stacked on the pin | Pin held by 1, others spread | Gain from stacking |
|---|---|---|---|
| your 3-point | mars 88% (+1.88) | mars 78% (+1.78) | **+0.10** |
| your 4-point | mars 88% (+1.88) | mars 59% (+1.59) | **+0.29** |

---

## 8. When you are the one pinned

Little has been written about this. The bot data below is the first hard evidence.

- **Your bear-off is frozen** until your checker is free, even if all your other checkers are home. [Math]
- Losing a single game is far better than a mars. When you're badly pinned, aim to **bear off at least one checker** if there's any chance, because that saves a point. [Math]
- **Your best hope is a counter-pin.** In 6,000 self-play games, players who got a checker pinned deep in the opponent's home (on their own 19–24 points) won **57% of the time if they also held a pin of their own**, against only **25% without one**. Without a counter-pin they lost a mars 54% of the time (26% with one). Part of this is that players who hold pins are already doing well, but the gap is huge. **When you're pinned, hunting for a counter-pin comes first.** [Our bot]
- A counter-pin starts a **release race**: whoever runs out of spare moves first must lift their pin (§6). Keep spare checkers that can move without breaking your pin. [Expert]
- ~~Don't crash your position early.~~ **Tested and rejected.** We had the bot follow "while one of your checkers is pinned, don't bring more checkers home" (pinning moves still allowed). It lost **0.32 points per game** against its normal self. While pinned, keep playing normally: build your home, look for the counter-pin, and keep the race going. [Our bot]

---

## 9. Playing a set (first to 3 points)

Chance of winning a 3-point set from each score, using the Plakoto match table built from the Palamedes analysis [Palamedes]:

| You \ Opponent | 0 | 1 | 2 |
|---|---|---|---|
| **0** | 50% | 34% | 21% |
| **1** | 66% | 50% | 35% |
| **2** | 79% | 65% | 50% |

*(Rows: your score. Columns: your opponent's score.)*

**How the score should change your play** [Math]:

- **You need 1 point:** a mars gains you nothing extra. Play for the plain win and take no risk just to chase a mars.
- **The opponent needs 1 point:** their mars costs you nothing extra. You can play bolder and take more pinning risks.
- **The opponent needs 2 points:** a mars ends the set for them. Be extra careful with your mother and your first-quadrant blots.
- **You need 2 points:** a mars wins the set. Pins in their home are worth even more than usual.

**How much is skill, how much is luck?** [Our bot] From the measured game results of each pairing, here is the chance that the stronger side wins a set:

| Pairing | One game | Set to 3 | Set to 5 | Set to 7 |
|---|---|---|---|---|
| Strong bot vs our simple rule-based bot (a stand-in for a careful, inexperienced player; not measured against real players) | 81.5% | **95%** | 98% | 99.5% |
| Strong bot vs early bot (50k training games) | 60% | 66% | 71% | 74% |
| Two nearly equal players (+0.06 points per game) | 52% | 53% | 54% | 55% |

A real skill gap shows up within one short set. Between players of similar strength, a set to 3 is close to a coin toss, so judge your progress over many sets.

---

## 10. Checklist for each move

1. **Is my mother (or a lone checker near the start) exposed?** Fix that first. Can I clear my last two start checkers together?
2. **Can I pin something?** Prefer pins deep in my home (2-, 3-, 4-points).
3. **Can I make a block**, ideally next to an existing one?
4. **Which blots will I leave, and how many shots does each give?** (§4)
5. **Timing:** after this move, will I soon be forced to break a block or lift a pin? Am I about to lift a home pin when another move exists? (Don't.)
6. **In the bear-off with a pin:** am I stacking on the pin?
7. **Set score:** does a mars matter right now, for me or for them?

---

## 11. Lessons from the bot: six costly mistakes

We had the bot play itself and flagged every position where the simple rules of thumb (our simple rule-based bot, which follows v1 of this guide; it is a stand-in for a careful, inexperienced player and has not been measured against real players) chose a clearly worse move. Each example below was confirmed with 1,500 playouts per candidate move. [Our bot]

The six examples were picked by hand because they teach well. To check that they are not freak cases, we also took 100 random positions of the same kind (the net and the simple bot disagree by 0.25 points or more) and rolled out both moves 1,000 times each. The net's move came out better in 67 of the 70 cases the playouts could decide (the other 30 were too close to call), so the pattern holds beyond the hand-picked examples.

In the diagrams **you are X**, moving 24 → 1. `2X` means two of your checkers. `1X/o` means your checker pinning an opponent checker; `1O/x` means one of yours is pinned.

### Mistake 1: not taking a deep pin because it "breaks a point"

You roll **6-4** on your 6th turn:

```
   13    14    15    16    17    18 |   19    20    21    22    23    24
   2O     .     .    2X     .     . |   1X    1X     .     .     .   11X
    .     .     .     .     .     . |   1O    2O     .     .     .   10O
   12    11    10     9     8     7 |    6     5     4     3     2     1
X off:  0  pips: 335    O off:  0  pips: 323    
```

- **Best: 16/10 10/6**, pinning the opponent's builder on your 6-point (**+0.36**).
- Tempting but wrong: 24/18 24/20 (keep unstacking, −0.21) or 24/18 20/16 (−0.38).
- Yes, it breaks your 16-point block and leaves blots. But a pin in your home board this early is worth about a point (§6), far more than one block.

Same lesson in another game: with 6-4, **13/9 9/3** pinning a blot on your 3-point scores −0.34, against −1.47 for the "solid" 19/13 24/20. **When a deep pin is on offer, take it.**

### Mistake 2: lifting your pin to make a "safe" small move

You roll **3-4** late in the game. You hold a pin on your 4-point (`1X/o`) and the opponent has several of your checkers pinned:

```
   13    14    15    16    17    18 |   19    20    21    22    23    24
   2O    2O    3O    2X    4X    2X |   2X     .  1O/x     .     .     .
   2O     .  2O/x  1O/x    1O     . |    .     .  1X/o     .     .    1X
   12    11    10     9     8     7 |    6     5     4     3     2     1
X off:  0  pips: 219    O off:  0  pips: 190    
```

- **Best: 19/16** (−0.22).
- Wrong: **4/1** (−1.69). The 4 lifts your pin, freeing the opponent's last checker, and the game turns from difficult into almost certainly a mars against you.
- **A pin in your home board is your lifeline. Never lift it while any other legal move exists.**

### Mistake 3: missing the chance to clear your last two start checkers when you are running out of other moves

You roll **6-4**. You have two checkers left on your 24-point, and an opponent blot on your 5-point invites a pin:

```
   13    14    15    16    17    18 |   19    20    21    22    23    24
 2O/x  1O/x    2X     .    2X     . | 1O/x     .  1O/x     .     .    2X
    .     .  2O/x  2O/x    2O    2O |    .    1O     .  3X/o     .     .
   12    11    10     9     8     7 |    6     5     4     3     2     1
X off:  0  pips: 207    O off:  0  pips: 219    
```

- **Best: 24/18 24/20**, both checkers leaving the start together (−0.99).
- Wrong: 15/11 11/5, pinning on 5 (−1.24).
- Once one of those two checkers has to leave alone, the other becomes a lone mother that can be pinned for an instant mars. Six of your checkers are pinned here, so you have few spare moves and may soon be forced to move one of the pair alone. **When you can clear the last two together, do it**, even ahead of a decent pin. Otherwise two checkers on the start are safe (§5).

### Mistake 4: running one checker instead of building from the start

You roll **4-3** on your 8th turn. You hold a pin on your 4-point, and one of your checkers is pinned on your 19:

```
   13    14    15    16    17    18 |   19    20    21    22    23    24
    .     .     .     .     .     . | 1O/x    1X     .    2X     .   10X
    .     .     .     .     .    1O |    .    1O  1X/o    3O     .    8O
   12    11    10     9     8     7 |    6     5     4     3     2     1
X off:  0  pips: 327    O off:  0  pips: 323    
```

- **Best: 24/20 24/21**: make the 20-point block, put a new builder on 21, and take two more checkers off the start (**+0.12**).
- Wrong: 20/16 16/13, running the lone 20 checker into the outfield (**−0.42**).
- In the first 10–25 turns the bot keeps feeding checkers off the start and building blocks near home. A lone runner in the outfield does little and becomes a target. This was the most common mistake pattern in the opening phase.

### Mistake 5: building quietly while you are pinned

You roll **3-3** on your 12th turn. Two of your checkers are pinned (on your 17- and 20-points), and the opponent has a lone checker on your 2-point, just off their start:

```
   13    14    15    16    17    18 |   19    20    21    22    23    24
    .    1X     .    2X  1O/x    1X |    .  1O/x    1X     .     .    8X
    .     .     .    2O     .    2O |   2O     .     .     .    1O    6O
   12    11    10     9     8     7 |    6     5     4     3     2     1
X off:  0  pips: 314    O off:  0  pips: 286    
```

- **Best: 14/11 11/8 8/5 5/2**: run one checker all the way round and pin their checker on your 2-point (**+0.62**).
- Wrong: the "solid" building plays such as 21/18 14/11 11/8 8/5 (**−1.72**) or 24/21 (3) 21/18 (−1.60).
- That's a swing of more than **2 points**. When you're pinned, a counter-pin is your way back into the game (§8). A deep one, like this pin on your 2-point, is the best of all.

### Mistake 6: pushing home checkers deeper while back checkers still need to come round

You roll **3-1** on turn 29. You hold a pin on your 3-point with two checkers on it, and a spare on your 4-point:

```
   13    14    15    16    17    18 |   19    20    21    22    23    24
    .     .     .    2X  1O/x    2X |   2X  1O/x     .     .     .    4X
   2O    3O    2O     .    1O    1O |   2O    1O    1X  2X/o     .     .
   12    11    10     9     8     7 |    6     5     4     3     2     1
X off:  0  pips: 249    O off:  0  pips: 226    
```

- **Best: 19/16 19/18**: bring your back checkers forward (**+0.98**).
- Wrong: 4/1 3/2, pushing home checkers deeper (**+0.26**).
- Here, your spare home checkers are your **timing**: they absorb future rolls so the dice never force you off your pin (§7). The back checkers are the ones that need to move.
- **But this is not a general ban on your 1- and 2-points.** When we made the bot follow "never add checkers to your 1- or 2-point before the bear-off" (pins allowed), it lost **0.29 points per game** (§12). In Mahbusa those points sit right in front of the opponent's start, so checkers there are a wall, not dead weight. The lesson is narrower: don't spend a roll deepening your home when back checkers still need to come round.

---

## 12. Testing the rules of thumb

We turned common rules of thumb into strict rules and made the bot obey them: it picked its best move among those that follow the rule, whenever one existed. Then it played against its normal self. The cost shows how much each rule gives away against the unrestricted bot. A cheap rule is a good habit for humans; an expensive one is a myth or needs exceptions. [Our bot]

The first five rules are cheap, so we measured them precisely (100,000 games each); the rest cost so much that 10,000 games were plenty. Every cost carries its 95% interval. We also made the simple rule-based bot (§11) obey each rule and play against its normal self (20,000 games each, interval about ±0.02), to see whether a rule helps a weaker player.

| Rule (followed strictly) | How often it changed the bot's move | Cost for the strong bot (per game) | Effect for the simple bot (per game) |
|---|---|---|---|
| Never lift a home-board pin on purpose (§7) | 0.9% | −0.027 ± 0.009 | +0.014 |
| Never leave a single checker on your start (§5) | 1.2% | −0.019 ± 0.009 | −0.015 |
| Always take a pin in your home board when you can | 0.9% | −0.045 ± 0.009 | +0.005 |
| Always take a pin anywhere when you can | 2.3% | **−0.134 ± 0.009** | +0.011 |
| Never leave a blot that 6+ rolls can pin | 8.9% | **−0.34 ± 0.03** | −0.082 |
| Never leave a blot that 12+ rolls can pin | 7.2% | **−0.33 ± 0.03** | +0.014 |
| Never add checkers to your 1- or 2-point before the bear-off | 18.6% | **−0.29 ± 0.03** | −0.144 |
| While pinned, don't bring more checkers home | 11.6% | **−0.32 ± 0.03** | −0.069 |
| Always make as many points as possible | 32.6% | **−0.85 ± 0.03** | not tested |

What this tells us:
- **Good habits (cheap):** never lifting a home pin, and not leaving a lone checker on your start. Each costs the strong bot only 0.02–0.03 points per game, which is small but real (about three standard errors), so they are habits, not laws.
- **Pin with judgment, not on reflex.** Deep home pins are usually right (§6), but grabbing every pin costs 0.13 points per game. Some pins expose too much or break your structure.
- **Mahbusa rewards calculated risk.** Even mild caution, refusing blots that 1 roll in 3 could pin, costs a third of a point per game. Strong play leaves risky blots when the payoff is worth it (§4).
- **Your 1- and 2-points aren't "buried".** They're the wall in front of the opponent's start (§11, Mistake 6).
- **Blocks aren't everything.** Maximising made points is the worst rule of all (−0.85).
- **Grading:** a rule costing under 0.05 points per game is a safe habit; 0.1–0.2 is a tendency with real exceptions; 0.3 or more is wrong as a rule.
- **The simple bot gains nothing from any rule.** Its numbers are all within about 0.02 of zero or negative. That is partly because it already obeys the cheap rules almost always (they change only 0.1–0.8% of its moves), so there is little left to gain, and it already avoids most risky blots (the 12+ rule changes 1.5% of its moves and costs it nothing). The expensive rules still cost it points, though less than they cost the strong bot (−0.07 to −0.14). Our simple bot already follows v1 of this guide, so it is not a true beginner, and this test cannot tell you whether the rules would help one.

Other questions answered this round:
- **Does the Egyptian mother rule matter?** In 20,000 bot games, 4.3% ended with a mother pin. Only **0.8% of games** ended that way while the pinner still had checkers on its own start, the case where Greek Plakoto would let the game continue. In practice, **the rule difference almost never matters.** [Our bot]
- **More training?** A network trained on 3 million games was no stronger than the 2-million one (50.1% over 20,000 games, at a lower learning rate). The network has plateaued under this training setup, and all numbers in this guide come from the 2M bot. [Our bot]

## 13. Open questions

Answered so far: second-player replies (§3), how much risk strong play takes (§4), the cost of a lone mother (§5), the value of a pin and the pin standoff (§6), clear rules for lifting pins and stacking on them (§7), counter-pins and not freezing up when pinned (§8), skill vs luck in a set (§9), six costly mistakes (§11), tested rules of thumb and the mother-rule question (§12).

Still open:
- **Even more positions**: a larger, searchable quiz built from `analysis/net2m/quiz_candidates.tsv`.
- **Being pinned**: a fuller defensive plan beyond "get a counter-pin".

---

*Sources: J. Mamoun, "Plakoto: Introduction, Rules and Basic Strategy" (primer) · N. Papahristou & I. Refanidis, "Opening Statistics and Match Play for Backgammon Games" and "On the Design and Training of Bots to Play Backgammon Variants" · tavli.net and boardgammon.com Plakoto guides · Jawaker, Egyptian Streets. Full list in `research/RESEARCH.md`.*

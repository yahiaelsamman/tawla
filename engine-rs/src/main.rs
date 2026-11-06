mod features;
mod nn;
mod play;
mod rng;
mod state;
mod train;

use nn::{equity, Net, Scratch};
use play::{bench, play_from, NetPlayer, Player, Tally};
use rng::Rng;
use state::{Move, MoveGen, State};
use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::sync::Arc;
use std::time::Instant;

fn args_map(args: &[String]) -> HashMap<String, String> {
    let mut m = HashMap::new();
    let mut i = 0;
    while i < args.len() {
        if let Some(k) = args[i].strip_prefix("--") {
            let v = args.get(i + 1).cloned().unwrap_or_default();
            m.insert(k.to_string(), v);
            i += 2;
        } else {
            i += 1;
        }
    }
    m
}

fn get<T: std::str::FromStr>(m: &HashMap<String, String>, k: &str, d: T) -> T {
    m.get(k).and_then(|v| v.parse().ok()).unwrap_or(d)
}

fn csv(a: &[u8]) -> String {
    a.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")
}

/// stdin: "me;op;mpin;opin;d1;d2" per line; stdout: sorted "me|mpin|outcome" moves.
fn cmd_movegen() {
    let stdin = std::io::stdin();
    let mut out = std::io::BufWriter::new(std::io::stdout());
    let mut gen = MoveGen::default();
    let mut moves = Vec::new();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let f: Vec<&str> = line.trim().split(';').collect();
        if f.len() != 6 {
            continue;
        }
        let parse = |s: &str| -> [u8; 25] {
            let v: Vec<u8> = s.split(',').map(|x| x.parse().unwrap()).collect();
            v.try_into().unwrap()
        };
        let s = State { me: parse(f[0]), op: parse(f[1]), mpin: f[2].parse().unwrap(), opin: f[3].parse().unwrap() };
        gen.legal_moves(&s, f[4].parse().unwrap(), f[5].parse().unwrap(), &mut moves);
        let mut v: Vec<String> = moves.iter().map(|m| format!("{}|{}|{}", csv(&m.after.me), m.after.mpin, m.outcome)).collect();
        v.sort();
        writeln!(out, "{}", v.join(" ")).unwrap();
    }
}

fn cmd_train(m: &HashMap<String, String>) {
    let out = m.get("out").cloned().unwrap_or("nets/td".into());
    let games: u64 = get(m, "games", 100_000);
    let threads: usize = get(m, "threads", 4);
    let alpha: f32 = get(m, "alpha", 0.1);
    let hidden: usize = get(m, "hidden", 100);
    let seed: u64 = get(m, "seed", 1);
    let every: u64 = get(m, "checkpoint", 25_000);
    let bench_games: u64 = get(m, "bench-games", 2000);
    let net = match m.get("resume") {
        Some(p) => Net::load(p).expect("cannot load resume net"),
        None => Net::new(hidden, &mut Rng::new(seed)),
    };
    eprintln!(
        "training: start at {} games, +{} games, alpha {}, hidden {}, threads {}",
        net.games_trained, games, alpha, net.n_hid, threads
    );
    let cfg = train::TrainCfg { games, threads, alpha, seed };
    let t0 = Instant::now();
    let mut log = std::fs::OpenOptions::new().create(true).append(true).open(format!("{}.log", out)).unwrap();
    let net = train::train(net, &cfg, every, |snap, done| {
        let path = format!("{}-{}k.net", out, snap.games_trained / 1000);
        snap.save(&path).unwrap();
        snap.save(&format!("{}-latest.net", out)).unwrap();
        let rate = done as f64 / t0.elapsed().as_secs_f64();
        let t = bench(&path, "heuristic", bench_games, 99, 1);
        let line = format!(
            "{:>9} games  {:6.1} games/s  vs heuristic: {}",
            snap.games_trained,
            rate,
            t.summary("net", "heuristic")
        );
        eprintln!("{}", line);
        writeln!(log, "{}", line).unwrap();
    });
    net.save(&format!("{}-latest.net", out)).unwrap();
    eprintln!("done in {:.0}s", t0.elapsed().as_secs_f64());
}

fn cmd_bench(m: &HashMap<String, String>) {
    let a = m.get("a").cloned().unwrap_or("heuristic".into());
    let b = m.get("b").cloned().unwrap_or("random".into());
    let n: u64 = get(m, "games", 1000);
    let threads: usize = get(m, "threads", 4);
    let seed: u64 = get(m, "seed", 7);
    let t0 = Instant::now();
    let t = bench(&a, &b, n, seed, threads);
    println!("{}", t.summary(&a, &b));
    eprintln!("{:.0} games/s", t.games as f64 / t0.elapsed().as_secs_f64());
}

/// Roll out positions with the net playing both sides. Returns (mean points for the
/// player NOT on roll at `start`, i.e. the one who just moved, std error).
fn rollout(net: &Arc<Net>, start: State, n: u64, seed: u64, threads: usize) -> (f64, f64, Tally) {
    let per = (n + threads as u64 - 1) / threads as u64;
    let handles: Vec<_> = (0..threads)
        .map(|t| {
            let net = net.clone();
            std::thread::spawn(move || {
                let mut p0 = NetPlayer::new(net.clone());
                let mut p1 = NetPlayer::new(net);
                let mut rng = Rng::new(seed.wrapping_mul(31).wrapping_add(t as u64));
                let mut gen = MoveGen::default();
                let mut tally = Tally::new();
                let (mut sum, mut sq) = (0f64, 0f64);
                for _ in 0..per {
                    let mut ps: [&mut dyn Player; 2] = [&mut p0, &mut p1];
                    // player 0 = the one who just moved; player 1 is on roll.
                    let g = play_from(start, 1, &mut ps, &mut rng, &mut gen);
                    let v = if g.winner == 0 { g.points as f64 } else { -(g.points as f64) };
                    sum += v;
                    sq += v * v;
                    tally.add(&g);
                }
                (sum, sq, tally)
            })
        })
        .collect();
    let (mut sum, mut sq, mut tally) = (0f64, 0f64, Tally::new());
    for h in handles {
        let (a, b, t) = h.join().unwrap();
        sum += a;
        sq += b;
        tally.merge(&t);
    }
    let cnt = tally.games as f64;
    let mean = sum / cnt;
    let var = sq / cnt - mean * mean;
    (mean, (var / cnt).sqrt(), tally)
}

fn top_candidates(net: &Net, s: &State, moves: &[Move], k: usize) -> Vec<(f32, Move)> {
    let mut sc = Scratch::new(net);
    let mut v: Vec<(f32, Move)> = moves
        .iter()
        .map(|m| (if m.outcome > 0 { m.outcome as f32 } else { -equity(net.eval(&m.after.flip(), &mut sc)) }, *m))
        .collect();
    let _ = s;
    v.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    v.truncate(k);
    v
}

/// Opening analysis: for every first roll, roll out the top-k moves.
fn cmd_opening(m: &HashMap<String, String>) {
    let net = Arc::new(Net::load(m.get("net").expect("--net required")).unwrap());
    let n: u64 = get(m, "games", 5000);
    let k: usize = get(m, "top", 3);
    let threads: usize = get(m, "threads", 4);
    let mut gen = MoveGen::default();
    let mut moves = Vec::new();
    let s0 = State::initial();
    println!("roll\tmove\tnet_eq\trollout_eq\tse\twin%\tdouble_win%\tdouble_loss%");
    for d1 in 1..=6u8 {
        for d2 in 1..=d1 {
            gen.legal_moves(&s0, d1, d2, &mut moves);
            for (rank, (v, mv)) in top_candidates(&net, &s0, &moves, k).into_iter().enumerate() {
                if n == 0 {
                    println!("{}{}\t{}\t{:+.3}", d1, d2, mv.notation(), v);
                    continue;
                }
                let (mean, se, t) = rollout(&net, mv.after.flip(), n, 1000 + rank as u64, threads);
                let g = t.games as f64;
                println!(
                    "{}{}\t{}\t{:+.3}\t{:+.3}\t{:.3}\t{:.1}\t{:.1}\t{:.1}",
                    d1,
                    d2,
                    mv.notation(),
                    v,
                    mean,
                    se,
                    100.0 * t.wins[0] as f64 / g,
                    100.0 * t.doubles[0] as f64 / g,
                    100.0 * t.doubles[1] as f64 / g
                );
                std::io::stdout().flush().unwrap();
            }
        }
    }
}


/// Value of a single early pin by point. Start position, except player A (who just
/// moved) has one checker pinning an opponent checker on A's point p. Opponent on roll.
fn cmd_pinvalue(m: &HashMap<String, String>) {
    let net = Arc::new(Net::load(m.get("net").expect("--net required")).unwrap());
    let n: u64 = get(m, "games", 4000);
    let threads: usize = get(m, "threads", 4);
    let base = State::initial();
    let (b, bse, _) = rollout(&net, base, n, 77, threads);
    println!("baseline (no pin, opponent on roll): {:+.3} ±{:.3}", b, bse);
    println!("pin_point\tequity\tse\tgain_vs_baseline\tdouble_win%\tdouble_loss%");
    for p in 2..24usize {
        let mut s = State::initial();
        s.me[24] -= 1;
        s.me[p] += 1;
        s.op[24] -= 1;
        s.op[25 - p] += 1;
        s.mpin = 1 << p;
        let (e, se, t) = rollout(&net, s, n, 100 + p as u64, threads);
        let g = t.games as f64;
        println!("{}\t{:+.3}\t{:.3}\t{:+.3}\t{:.1}\t{:.1}", p, e, se, e - b,
            100.0 * t.doubles[0] as f64 / g, 100.0 * t.doubles[1] as f64 / g);
        std::io::stdout().flush().unwrap();
    }
}

/// Second-player replies: after first roll `first` (e.g. 31) and the first player's
/// best move (by rollout of top-k, or 1-ply if --quick), list the best reply for each
/// of the 21 reply rolls.
fn cmd_replies(m: &HashMap<String, String>) {
    let net = Arc::new(Net::load(m.get("net").expect("--net required")).unwrap());
    let n: u64 = get(m, "games", 2000);
    let k: usize = get(m, "top", 3);
    let threads: usize = get(m, "threads", 4);
    let firsts: Vec<(u8, u8)> = match m.get("first") {
        Some(f) => f.split(',').map(|r| { let b = r.as_bytes(); (b[0] - b'0', b[1] - b'0') }).collect(),
        None => (1..=6u8).flat_map(|a| (1..=a).map(move |b| (a, b))).collect(),
    };
    let mut gen = MoveGen::default();
    let mut moves = Vec::new();
    println!("first_roll\tfirst_move\treply_roll\treply\tnet_eq\trollout_eq\tse");
    for (a, b) in firsts {
        gen.legal_moves(&State::initial(), a, b, &mut moves);
        let first = top_candidates(&net, &State::initial(), &moves, 1)[0].1;
        let s1 = first.after.flip(); // second player on roll
        for d1 in 1..=6u8 {
            for d2 in 1..=d1 {
                gen.legal_moves(&s1, d1, d2, &mut moves);
                let cands = top_candidates(&net, &s1, &moves, k);
                let mut best: Option<(f64, f64, f32, Move)> = None;
                for (v, mv) in cands {
                    let (e, se, _) = if n > 0 { rollout(&net, mv.after.flip(), n, 500, threads) } else { (v as f64, 0.0, Tally::new()) };
                    if best.map_or(true, |bb| e > bb.0) {
                        best = Some((e, se, v, mv));
                    }
                }
                let (e, se, v, mv) = best.unwrap();
                println!("{}{}\t{}\t{}{}\t{}\t{:+.3}\t{:+.3}\t{:.3}", a, b, first.notation(), d1, d2, mv.notation(), v, e, se);
                std::io::stdout().flush().unwrap();
            }
        }
    }
}


/// Persistent move chooser for the Python front-end. stdin: "me;op;mpin;opin;d1;d2";
/// stdout: "me|mpin|equity" of the chosen move ("pass" if none).
fn cmd_choose(m: &HashMap<String, String>) {
    let net = Arc::new(Net::load(m.get("net").expect("--net required")).unwrap());
    let mut player = NetPlayer::new(net);
    let mut gen = MoveGen::default();
    let mut moves = Vec::new();
    let stdin = std::io::stdin();
    let mut out = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let f: Vec<&str> = line.trim().split(';').collect();
        if f.len() != 6 {
            continue;
        }
        let parse = |s: &str| -> [u8; 25] {
            let v: Vec<u8> = s.split(',').map(|x| x.parse().unwrap()).collect();
            v.try_into().unwrap()
        };
        let s = State { me: parse(f[0]), op: parse(f[1]), mpin: f[2].parse().unwrap(), opin: f[3].parse().unwrap() };
        gen.legal_moves(&s, f[4].parse().unwrap(), f[5].parse().unwrap(), &mut moves);
        if moves.is_empty() {
            writeln!(out, "pass").unwrap();
        } else {
            let mut rng = Rng::new(0);
            let i = player.choose(&s, &moves, &mut rng);
            let eq = player.move_value(&moves[i]);
            writeln!(out, "{}|{}|{:.4}", csv(&moves[i].after.me), moves[i].after.mpin, eq).unwrap();
        }
        out.flush().unwrap();
    }
}


/// Self-play statistics with the net on both sides (for the human guide).
fn cmd_stats(m: &HashMap<String, String>) {
    let net = Arc::new(Net::load(m.get("net").expect("--net required")).unwrap());
    let n: u64 = get(m, "games", 10000);
    let threads: usize = get(m, "threads", 4);
    let per = (n + threads as u64 - 1) / threads as u64;
    // per pin point (pinner's numbering 1..24): [count, total turns held, pinner wins, pinner double wins, held to game end]
    type Acc = (Vec<[f64; 5]>, Vec<f64>, Vec<[f64; 3]>, [f64; 4]);
    let handles: Vec<_> = (0..threads).map(|t| {
        let net = net.clone();
        std::thread::spawn(move || -> Acc {
            let mut pl = NetPlayer::new(net);
            let mut rng = Rng::new(4242 + t as u64);
            let mut gen = MoveGen::default();
            let mut moves = Vec::new();
            let mut pins = vec![[0f64; 5]; 25];
            let mut start_empty_turn: Vec<f64> = Vec::new(); // own turn number when start emptied
            // by own-turn bucket (1-5, 6-10, 11-15, 16-20, 21+): [moves, sum max shots on blots left, moves leaving a blot with >= 6 shots]
            let mut risk = vec![[0f64; 3]; 5];
            let mut misc = [0f64; 4]; // games, mother-pin endings, doubles, total turns
            for _ in 0..per {
                let mut s = State::initial();
                let mut cur = 0usize;
                let mut own_turns = [0u32; 2];
                let mut start_done = [false; 2];
                let mut pin_since: [[u32; 25]; 2] = [[0; 25]; 2]; // turn when pin began (0 = none)
                let mut open: Vec<(usize, usize, u32)> = Vec::new(); // closed pins: (player, point, duration)
                let mut turn = 0u32;
                let (winner, points, mother);
                loop {
                    turn += 1;
                    own_turns[cur] += 1;
                    let (d1, d2) = (rng.die(), rng.die());
                    gen.legal_moves(&s, d1, d2, &mut moves);
                    let after = if moves.is_empty() { s } else {
                        let i = if moves.len() == 1 { 0 } else { pl.choose(&s, &moves, &mut rng) };
                        let mv = moves[i];
                        if mv.outcome > 0 {
                            winner = cur; points = mv.outcome; mother = mv.after.me[0] < 15;
                            // close the mover's new pins (mother pin) and all open pins
                            break;
                        }
                        mv.after
                    };
                    // risk stats for the mover
                    if !moves.is_empty() {
                        let f = after.flip();
                        let mut mx = 0u32;
                        for p in 1..25 { if features::is_my_blot(&after, p) { mx = mx.max(features::pin_shots(&f, 25 - p)); } }
                        let b = ((own_turns[cur] as usize - 1) / 5).min(4);
                        risk[b][0] += 1.0; risk[b][1] += mx as f64; if mx >= 6 { risk[b][2] += 1.0; }
                    }
                    if !start_done[cur] && after.me[24] == 0 { start_done[cur] = true; start_empty_turn.push(own_turns[cur] as f64); }
                    // pin bookkeeping: mover's pins = after.mpin (mover numbering), opponent's = after.opin
                    for (who, mask) in [(cur, after.mpin), (1 - cur, after.opin)] {
                        for p in 1..25 {
                            let on = (mask >> p) & 1 == 1;
                            let since = pin_since[who][p];
                            if on && since == 0 { pin_since[who][p] = turn; }
                            if !on && since > 0 { open.push((who, p, turn - since)); pin_since[who][p] = 0; }
                        }
                    }
                    s = after.flip();
                    cur = 1 - cur;
                }
                let won = |who: usize| who == winner;
                for (who, p, dur) in open.iter() {
                    let e = &mut pins[*p];
                    e[0] += 1.0; e[1] += *dur as f64;
                    if won(*who) { e[2] += 1.0; if points == 2 { e[3] += 1.0; } }
                }
                for who in 0..2 { for p in 1..25 { let since = pin_since[who][p]; if since > 0 {
                    let e = &mut pins[p];
                    e[0] += 1.0; e[1] += (turn - since) as f64; e[4] += 1.0;
                    if won(who) { e[2] += 1.0; if points == 2 { e[3] += 1.0; } }
                } } }
                misc[0] += 1.0; if mother { misc[1] += 1.0; } if points == 2 { misc[2] += 1.0; } misc[3] += turn as f64;
            }
            (pins, start_empty_turn, risk, misc)
        })
    }).collect();
    let mut pins = vec![[0f64; 5]; 25];
    let mut se: Vec<f64> = Vec::new();
    let mut risk = vec![[0f64; 3]; 5];
    let mut misc = [0f64; 4];
    for h in handles {
        let (p, s2, r, mi) = h.join().unwrap();
        for i in 0..25 { for j in 0..5 { pins[i][j] += p[i][j]; } }
        se.extend(s2);
        for i in 0..5 { for j in 0..3 { risk[i][j] += r[i][j]; } }
        for j in 0..4 { misc[j] += mi[j]; }
    }
    let g = misc[0];
    println!("games {}  double-win rate {:.1}%  mother-pin endings {:.1}%  avg turns {:.1}", g, 100.0 * misc[2] / g, 100.0 * misc[1] / g, misc[3] / g);
    se.sort_by(|a, b| a.partial_cmp(b).unwrap());
    if !se.is_empty() {
        let q = |f: f64| se[((se.len() - 1) as f64 * f) as usize];
        println!("own turn when the start is emptied: median {} (25%: {}, 75%: {}), emptied in {:.1}% of player-games", q(0.5), q(0.25), q(0.75), 100.0 * se.len() as f64 / (2.0 * g));
    }
    println!("\nPINS by point (pinner's numbering; opponent starts on point 1)");
    println!("point\tpins_per_game\tavg_turns_held\tpinner_win%\tpinner_double%\theld_to_end%");
    for p in 1..25 {
        let e = pins[p];
        if e[0] < 1.0 { continue; }
        println!("{}\t{:.3}\t{:.1}\t{:.1}\t{:.1}\t{:.1}", p, e[0] / g, e[1] / e[0], 100.0 * e[2] / e[0], 100.0 * e[3] / e[0], 100.0 * e[4] / e[0]);
    }
    println!("\nRISK: blots left after the move, by the mover's own turn number");
    println!("turns\tavg_max_shots_on_a_blot(/36)\tmoves_leaving_a_blot_with_6+_shots%");
    for (i, lab) in ["1-5", "6-10", "11-15", "16-20", "21+"].iter().enumerate() {
        let r = risk[i];
        if r[0] > 0.0 { println!("{}\t{:.2}\t{:.1}", lab, r[1] / r[0], 100.0 * r[2] / r[0]); }
    }
}


/// Instructive positions: net self-play; record decisions where the heuristic's move
/// is much worse (by net 1-ply equity) than the net's best move.
fn cmd_quiz(m: &HashMap<String, String>) {
    let net = Arc::new(Net::load(m.get("net").expect("--net required")).unwrap());
    let n: u64 = get(m, "games", 300);
    let thr: f32 = get(m, "threshold", 0.25);
    let seed: u64 = get(m, "seed", 5);
    let mut pl = NetPlayer::new(net.clone());
    let mut heur = play::HeuristicPlayer;
    let mut rng = Rng::new(seed);
    let mut gen = MoveGen::default();
    let mut moves = Vec::new();
    println!("game\tturn\tme\top\tmpin\topin\td1\td2\tbest\tbest_eq\theur\theur_eq\tloss");
    for g in 0..n {
        let mut s = State::initial();
        for turn in 1..2000u32 {
            let (d1, d2) = (rng.die(), rng.die());
            gen.legal_moves(&s, d1, d2, &mut moves);
            if moves.is_empty() { s = s.flip(); continue; }
            let bi = pl.choose(&s, &moves, &mut rng);
            if moves.len() > 1 {
                let hi = heur.choose(&s, &moves, &mut rng);
                if hi != bi {
                    let be = pl.move_value(&moves[bi]);
                    let he = pl.move_value(&moves[hi]);
                    if be - he >= thr {
                        println!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:+.3}\t{}\t{:+.3}\t{:.3}", g, turn, csv(&s.me), csv(&s.op), s.mpin, s.opin, d1, d2,
                            moves[bi].notation(), be, moves[hi].notation(), he, be - he);
                    }
                }
            }
            let mv = moves[bi];
            if mv.outcome > 0 { break; }
            s = mv.after.flip();
        }
    }
}

/// Roll out specific moves from a position: --pos "me;op;mpin;opin" --dice 31
fn cmd_compare(m: &HashMap<String, String>) {
    let net = Arc::new(Net::load(m.get("net").expect("--net required")).unwrap());
    let n: u64 = get(m, "games", 2000);
    let threads: usize = get(m, "threads", 4);
    let f: Vec<&str> = m.get("pos").expect("--pos").split(';').collect();
    let parse = |s: &str| -> [u8; 25] { let v: Vec<u8> = s.split(',').map(|x| x.parse().unwrap()).collect(); v.try_into().unwrap() };
    let s = State { me: parse(f[0]), op: parse(f[1]), mpin: f[2].parse().unwrap(), opin: f[3].parse().unwrap() };
    let d = m.get("dice").expect("--dice").as_bytes();
    let mut gen = MoveGen::default();
    let mut moves = Vec::new();
    gen.legal_moves(&s, d[0] - b'0', d[1] - b'0', &mut moves);
    let k: usize = get(m, "top", 4);
    for (v, mv) in top_candidates(&net, &s, &moves, k) {
        let (e, se, _) = if mv.outcome > 0 { (mv.outcome as f64, 0.0, Tally::new()) } else { rollout(&net, mv.after.flip(), n, 9, threads) };
        println!("{}\tnet {:+.3}\trollout {:+.3} ±{:.3}", mv.notation(), v, e, se);
    }
}


/// Release and defence statistics from net self-play.
/// Releases of home-board pins (pinner's point 1..6): voluntary vs forced, by the
/// pinner's checkers left on the board. Deep-pinned players (own checker pinned on
/// own point >= 19): win rate with vs without a counter-pin of their own.
fn cmd_releases(m: &HashMap<String, String>) {
    let net = Arc::new(Net::load(m.get("net").expect("--net required")).unwrap());
    let n: u64 = get(m, "games", 5000);
    let threads: usize = get(m, "threads", 4);
    let per = (n + threads as u64 - 1) / threads as u64;
    let handles: Vec<_> = (0..threads).map(|t| {
        let net = net.clone();
        std::thread::spawn(move || {
            let mut pl = NetPlayer::new(net);
            let mut rng = Rng::new(777 + t as u64);
            let mut gen = MoveGen::default();
            let mut moves = Vec::new();
            // rel[checkers_left 0..15][0 = voluntary, 1 = forced]
            let mut rel = vec![[0f64; 2]; 16];
            // held[checkers_left]: decisions where a home pin was kept although releasing was possible
            let mut kept = vec![0f64; 16];
            // deep[with_counter_pin 0/1] = [player-games, wins, double losses]
            let mut deep = [[0f64; 3]; 2];
            for _ in 0..per {
                let mut s = State::initial();
                let mut cur = 0usize;
                let mut deep_flag: [Option<usize>; 2] = [None, None];
                let winner; let points;
                loop {
                    let (d1, d2) = (rng.die(), rng.die());
                    gen.legal_moves(&s, d1, d2, &mut moves);
                    if moves.is_empty() { s = s.flip(); cur = 1 - cur; continue; }
                    // deep-pinned status for the player to move
                    if deep_flag[cur].is_none() && (19..25).any(|p| s.my_pinned_at(p)) {
                        let counter = (1..25).any(|p| s.i_pin_at(p));
                        deep_flag[cur] = Some(counter as usize);
                    }
                    let i = if moves.len() == 1 { 0 } else { pl.choose(&s, &moves, &mut rng) };
                    let mv = moves[i];
                    let left = 15 - s.me[0] as usize;
                    for p in 1..7 {
                        if !s.i_pin_at(p) { continue; }
                        let could_keep = moves.iter().any(|x| x.outcome > 0 || x.after.i_pin_at(p));
                        let could_release = moves.iter().any(|x| x.outcome == 0 && !x.after.i_pin_at(p));
                        if mv.outcome == 0 && !mv.after.i_pin_at(p) {
                            rel[left][if could_keep { 0 } else { 1 }] += 1.0;
                        } else if could_release {
                            kept[left] += 1.0;
                        }
                    }
                    if mv.outcome > 0 { winner = cur; points = mv.outcome; break; }
                    s = mv.after.flip();
                    cur = 1 - cur;
                }
                for who in 0..2 { if let Some(c) = deep_flag[who] {
                    deep[c][0] += 1.0;
                    if who == winner { deep[c][1] += 1.0; } else if points == 2 { deep[c][2] += 1.0; }
                } }
            }
            (rel, kept, deep)
        })
    }).collect();
    let mut rel = vec![[0f64; 2]; 16];
    let mut kept = vec![0f64; 16];
    let mut deep = [[0f64; 3]; 2];
    for h in handles {
        let (r, k, d) = h.join().unwrap();
        for i in 0..16 { rel[i][0] += r[i][0]; rel[i][1] += r[i][1]; kept[i] += k[i]; }
        for i in 0..2 { for j in 0..3 { deep[i][j] += d[i][j]; } }
    }
    println!("HOME-PIN RELEASES by the pinner's checkers still on the board ({} games)", n);
    println!("checkers_left\tvoluntary_releases\tforced_releases\tkept_when_could_release\tvoluntary_release_rate%");
    for l in 1..16 {
        let (v, f, k) = (rel[l][0], rel[l][1], kept[l]);
        if v + f + k < 1.0 { continue; }
        println!("{}\t{}\t{}\t{}\t{:.1}", l, v, f, k, 100.0 * v / (v + k).max(1.0));
    }
    println!("\nDEEP-PINNED players (own checker pinned on own 19..24 at some point)");
    for (c, lab) in [(0, "without a pin of their own"), (1, "holding a counter-pin")] {
        let d = deep[c];
        println!("{}: {} player-games, win {:.1}%, double loss {:.1}%", lab, d[0], 100.0 * d[1] / d[0].max(1.0), 100.0 * d[2] / d[0].max(1.0));
    }
}


/// Pin-lifting decisions: every position (net self-play) where the mover holds a pin in
/// its home board (points 1..6) and has both a move that keeps it and one that lifts it.
fn cmd_liftdata(m: &HashMap<String, String>) {
    let net = Arc::new(Net::load(m.get("net").expect("--net required")).unwrap());
    let n: u64 = get(m, "games", 4000);
    let threads: usize = get(m, "threads", 4);
    let per = (n + threads as u64 - 1) / threads as u64;
    let handles: Vec<_> = (0..threads).map(|t| {
        let net = net.clone();
        std::thread::spawn(move || {
            let mut pl = NetPlayer::new(net);
            let mut rng = Rng::new(9100 + t as u64);
            let mut gen = MoveGen::default();
            let mut moves = Vec::new();
            let mut out = String::new();
            for g in 0..per {
                let mut s = State::initial();
                for turn in 1..3000u32 {
                    let (d1, d2) = (rng.die(), rng.die());
                    gen.legal_moves(&s, d1, d2, &mut moves);
                    if moves.is_empty() { s = s.flip(); continue; }
                    for p in 1..7usize {
                        if !s.i_pin_at(p) { continue; }
                        let mut keep = (f32::NEG_INFINITY, 0usize);
                        let mut lift = (f32::NEG_INFINITY, 0usize);
                        let mut has_win = false;
                        for (i, mv) in moves.iter().enumerate() {
                            if mv.outcome > 0 { has_win = true; continue; }
                            let v = pl.move_value(mv);
                            if mv.after.i_pin_at(p) { if v > keep.0 { keep = (v, i); } } else if v > lift.0 { lift = (v, i); }
                        }
                        if has_win || keep.0 == f32::NEG_INFINITY || lift.0 == f32::NEG_INFINITY { continue; }
                        let (mp, op) = s.pips();
                        let outside_home: u32 = (7..25).map(|q| s.me[q] as u32).sum();
                        let opp_home: u32 = (1..7).map(|q| s.op[q] as u32).sum::<u32>(); // opponent checkers in THEIR home
                        let opp_not_home = 15 - s.op[0] as u32 - opp_home;
                        let i_am_pinned = s.opin != 0;
                        let other_pins = s.mpin.count_ones() - 1;
                        let below: u32 = (1..p).map(|q| s.me[q] as u32).sum();
                        out.push_str(&format!("{}\t{}\t{}{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:.4}\t{:.4}\t{}\t{}\t{}\t{}\n",
                            t * 1_000_000 + g as usize, turn, d1, d2, p, s.me[p], 15 - s.me[0], outside_home, below, s.op[0], opp_not_home,
                            mp, op, i_am_pinned as u8, other_pins, (s.me[24] > 0) as u8, keep.0, lift.0,
                            moves[keep.1].notation(), moves[lift.1].notation(),
                            format!("{}", s.me.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")),
                            format!("{};{};{}", s.op.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","), s.mpin, s.opin)));
                    }
                    let i = if moves.len() == 1 { 0 } else { pl.choose(&s, &moves, &mut rng) };
                    let mv = moves[i];
                    if mv.outcome > 0 { break; }
                    s = mv.after.flip();
                }
            }
            out
        })
    }).collect();
    println!("game\tturn\tdice\tpin_point\tstacked\tmy_left\tmy_outside_home\tmy_below_pin\topp_off\topp_not_home\tmy_pips\topp_pips\ti_am_pinned\tother_pins\tmy_start_occupied\tkeep_eq\tlift_eq\tkeep_move\tlift_move\tme\top_mpin_opin");
    for h in handles { print!("{}", h.join().unwrap()); }
}

fn cmd_speed() {
    let mut gen = MoveGen::default();
    let mut rng = Rng::new(3);
    let t0 = Instant::now();
    let mut tally = Tally::new();
    let mut a = play::RandomPlayer;
    let mut b = play::RandomPlayer;
    for i in 0..20000u64 {
        let mut ps: [&mut dyn Player; 2] = [&mut a, &mut b];
        let g = play_from(State::initial(), (i % 2) as usize, &mut ps, &mut rng, &mut gen);
        tally.add(&g);
    }
    let dt = t0.elapsed().as_secs_f64();
    println!("random vs random: {:.0} games/s single thread", 20000.0 / dt);
    println!("{}", tally.summary("random", "random"));
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("help");
    let m = args_map(&args[2.min(args.len())..]);
    match cmd {
        "movegen" => cmd_movegen(),
        "train" => cmd_train(&m),
        "bench" => cmd_bench(&m),
        "opening" => cmd_opening(&m),
        "speed" => cmd_speed(),
        "pinvalue" => cmd_pinvalue(&m),
        "choose" => cmd_choose(&m),
        "stats" => cmd_stats(&m),
        "quiz" => cmd_quiz(&m),
        "releases" => cmd_releases(&m),
        "liftdata" => cmd_liftdata(&m),
        "compare" => cmd_compare(&m),
        "replies" => cmd_replies(&m),
        _ => eprintln!("usage: mahbusa-rs <movegen|train|bench|opening|speed> [--key value ...]"),
    }
}

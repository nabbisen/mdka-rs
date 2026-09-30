# Addendum — benchmark regeneration, 2026-09-30 · separate the library from the filesystem

**Amends.** `handoff.md`, frozen and unedited. From the review at
`.git-exclude/reviewed/3.0.0-benchmark-regeneration/README.md` §3.
**State.** `b641446` is **correctly unpushed**. Hold it; this is a small addition to the same commit.
**Scope.** One measurement, one re-measured point, one sentence. **No new page structure.**

---

## 1. The work is good; one claim cannot ship as written

Everything else stands, including the two-day gap (§2 of the review: agreed, no re-run) and the string-
conversion finding, whose `deep_nest` control is what makes it credible.

**But the page's most quotable sentence is this one**, and it is not yet supportable:

> Bulk file conversion … is measurably faster, by a wide and growing margin … **The most likely mechanism**:
> the old return type carried a borrow (`&'a P`) through rayon's parallel `map`/`collect`, and the new one
> does not.

### 1.1 Your own data contradicts that mechanism

**A removed lifetime does not become more valuable as thread count rises.** Sequentially `3.0.0` is
**slower** (+7.5%, +8.9%); the gain appears only from 8 threads. A type-level borrow explains none of that
shape.

And the new type does strictly more work. I measured both against the real crate:

```
old  (&PathBuf, Result<PathBuf, MdkaError>)  = 32 B
new  FileOutcome                             = 48 B
```

plus the extra `PathBuf` per file your allocation table already shows at **+165 B**. Larger element, more
allocation, slower sequentially — and 37% faster at 32 threads. **The borrow cannot carry that.**

**I do not have a better mechanism either.** I considered false sharing — 32 B is exactly 2 per 64-byte
cache line, 48 B is 1.33 — but rayon's indexed `collect` gives each thread a *contiguous* range, so sharing
is confined to ~31 boundaries in 1000 elements. That cannot produce 37% either. Recorded so you do not
spend the same hour on it.

## 2. The control 🛑

**The benchmark writes 1000 files.** 12–20 ms for 1000 files is ~12–20 µs each *including a filesystem
write*, so it may be I/O-bound — and an unexplained 37% in an I/O-bound benchmark is likelier to be the
filesystem than the library.

**Re-run the 1000-file case with the writes removed** — convert to memory, discard the output — at
**1, 8 and 32 threads**, same two-worktree probes, same alternation and medians.

| Outcome | What the page then says |
|---|---|
| **The gap survives** | It is the library. Publish it as real **and still unexplained** — drop the borrow sentence, say the mechanism is unidentified and the effect is thread-scaling |
| **The gap vanishes or shrinks sharply** | The benchmark was measuring the filesystem. The page says bulk conversion costs one allocation per file and is otherwise unchanged, and the wall-time table is reported as an I/O-bound measurement, not a library property |

**Either result is publishable. The current sentence is not**, because it would be the number people quote.

## 3. Two smaller things

**3.1 The 4-thread point.** Your sweep reads −8.65% (2), **+2.16% (4)**, −21.99% (8). The page says the gap
*"widens with more threads"*; that point contradicts it and is unmentioned, on one measurement. **Repeat it,
or say the trend is not monotonic.** A sweep with an unexplained reversal is not a trend.

**3.2 Run 1's provenance.** Its raw JSON was lost to the `/tmp` reset and is quoted from the transcript.
That is disclosed in your report and it is the run that *disagrees* with the published conclusion, so
nothing is flattered — but the page's median includes a number with no file behind it. **Put that sentence
in the page**, not only in the report.

## 4. Criteria

1. §2's no-write measurement at 1, 8 and 32 threads, with its numbers.
2. The page's bulk narrative rewritten from whichever outcome §2 gives. **No mechanism asserted that the
   data does not carry** — "unidentified" is an acceptable and honest word.
3. §3.1 resolved: repeated, or the non-monotonicity stated.
4. §3.2's sentence in the page.
5. Nothing else on the page changes; peer versions still unbumped.

## 5. Committing and pushing

Amend or extend `b641446`; it is unpushed, so this can be one commit. Report first — a green run is not
approval. **Name who approved anything you push.**

Append to `.git-exclude/review-request/3.0.0-benchmark-regeneration/README.md` as Part 2.

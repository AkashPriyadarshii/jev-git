<!-- Title: jev-git — Sub-second Git pre-commit & pre-push semantic guard using TypeSafe AI Jev -->
<!-- Description: Statically linked Rust Git extension and hook runner that screens staged diffs for secrets, destructive payloads, and AI hallucinations in under 100ms with zero RAM bloat. -->
<!-- Keywords: git-hook, jev, typesafe-ai, rust, git-extension, security, pre-commit, ai-safety, cli -->

<p align="center">
  <h1 align="center">jev-git</h1>
  <p align="center"><strong>Sub-second Git pre-commit & pre-push semantic reflex gate powered by TypeSafe AI's Jev.</strong></p>
  <p align="center">
    <a href="https://github.com/AkashPriyadarshii/jev-git/releases"><img src="https://img.shields.io/badge/version-0.1.0-black?style=flat-square" alt="Version"></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-black?style=flat-square" alt="License"></a>
    <a href="https://typesafe.ai"><img src="https://img.shields.io/badge/engine-TypeSafe%20Jev-black?style=flat-square" alt="TypeSafe Jev"></a>
  </p>
  <p align="center">By <a href="https://github.com/AkashPriyadarshii">Akash Priyadarshi</a></p>
</p>

---

## Why jev-git?

Regex-based git hooks miss subtle semantic context. Full LLMs take 4–6 seconds per commit and burn hundreds of dollars in API credits.

`jev-git` runs in **~80ms** directly inside your `git commit` or `git push` workflow. It uses [TypeSafe AI](https://typesafe.ai)'s Jev (System One decision model) to evaluate staged diffs with calibrated probabilities for unredacted secrets, prompt-injection attacks, and destructive commands before they ever enter your git history.

- **Fast:** ~80ms p50 latency.
- **Lightweight:** Single static Rust binary. Zero daemons, zero background memory.
- **Native Git Integration:** Invoked seamlessly as `git jev`.

## Quickstart

```bash
# 1. Install pre-commit hook in any repo
git jev install

# 2. Check staged diff on demand
git jev check

# Or pipe any diff
git diff HEAD~1 | git jev check
```

## Architecture

- Single statically linked binary (<2MB).
- Uses blocking HTTP (`ureq`) for zero-runtime startup overhead.
- Evaluates `noul` decisions via `https://api.typesafe.ai/v1/systemone`.

## Non-goals

- Not a multi-agent framework.
- Not an interactive code-writing LLM.
- Not a heavy linter or syntax parser.

---

## Ecosystem

- [design-genius](https://github.com/AkashPriyadarshii/design-genius)
- [akash-design-engineering](https://github.com/AkashPriyadarshii/akash-design-engineering)
- [tdlib-android](https://github.com/AkashPriyadarshii/tdlib-android)
- [kharcha](https://github.com/AkashPriyadarshii/kharcha)

## Author

Akash Priyadarshi (Patna, Bihar, India)  
[GitHub](https://github.com/AkashPriyadarshii) · [Portfolio](https://akashpriyadarshi.vercel.app) · [LinkedIn](https://linkedin.com/in/akash-priyadarshi-1aa51b37a) · [Resume](https://akashpriyadarshii.github.io/Resume/)

## License

MIT © Akash Priyadarshi

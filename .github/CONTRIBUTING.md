## ✨ Contributing

We’re happy to receive feedback, bug reports, and questions via GitHub Issues.  
Pull requests are also welcome — though please note that we may not always be able to accept them.

This project is maintained as a labor of love. We welcome community participation, but:

- Issues that are respectful and constructive are appreciated.
- Pull requests are reviewed, but acceptance is not guaranteed.
- We do not engage in long debates or vision disagreements.
- If you have a different direction in mind, please fork freely, provided proper licensing is respected.

Thanks for understanding the scope and spirit of the project.

## Documentation examples

Every fenced code block in `docs/src/` whose info string names `rust`,
`python`, `js` or `ts` is **checked in CI** by the `docs example gate`
workflow. Rust blocks are compiled against the crate, TypeScript blocks are
type-checked against the generated `node/index.d.ts`, JavaScript blocks are
parsed, and Python blocks are syntax-checked with every `mdka` symbol they
name resolved against the installed package.

A block that is deliberately **not** runnable on its own — a signature
display, a type declaration, or a snippet that relies on a variable
introduced in the surrounding prose — must say so in its info string:

````markdown
```rust,fragment
pub fn html_to_markdown(html: &str) -> String
```
````

**The marker is what excludes a block.** There is no list of exceptions kept
somewhere else, because such a list drifts out of step with the documents it
describes. Rust's own `ignore` and `no_run` are honoured for the same
purpose. A block with no info string at all — which is how sample *output*
is written — is never checked.

If a block is genuinely runnable, do not mark it as a fragment to quiet the
gate; fix the example. That gate exists because two shipped examples failed
on their first line.

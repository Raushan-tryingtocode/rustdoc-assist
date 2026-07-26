# rustdoc-assist (lowkey trying to learn rust)

Honestly, I am just trying to build a RAG tool that lets me talk to the Rust documentation because I am trying to learn intermediate systems programming and eventually get a Google internship. Python is cool, but Rust is built different—it's a whole beast, and from my experience the docs are the best way to learn it.

## current status

Right now it's an offline CLI assistant! It doesn't need any API keys or internet, so anyone who clones it can just run it instantly. I added basic fuzzy matching so typos like `concurrancy` or queries like `what is 1.1` don't break it, and it gives you direct links to doc.rust-lang.org.

P.S: it actually works now so I did something right ig 💀

## how it works

- **seed docs:** just a basic array inside the code holding text from important Rust Book chapters (ownership, structs, traits, async/tokio, error handling).
- **search logic:** strips out filler words, parses section numbers, and uses basic edit distance so small typos still match the right chapter.
- **text output:** strings together a local explanation and prints the exact doc URL without calling external services.
- **tokio runtime:** handles terminal input asynchronously so it stays snappy.

## how to run

```bash
git clone [https://github.com/Raushan-tryingtocode/rustdoc-assist.git](https://github.com/Raushan-tryingtocode/rustdoc-assist.git)
cd rustdoc-assist
cargo run
```

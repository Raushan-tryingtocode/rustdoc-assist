# rustdoc-assist (lowkey trying to learn rust)

Honestly, I am just trying to build a rag tool that lets me talk to the rust documentation because i am trying to learn intermediate systems programming and eventually get a google internship. python is cool but rust is built different is a different beast and from my experiance the docs are the best way to learn it.

## current status

Right now it's super basic. I don't know how to read real files yet or do actual vector embeddings, so i literally just hardcoded a fake database inside the code to see if the search logic works.

P.S: it kinda works so I did something right ig.

## how it works (i think)

1. **mock_ingest**: gives us some fake text about ownership to play with.
2. **LocalStore**: a basic struct holding our data array. it has a `simple_search` method that just checks if the text contains the word you searched for.
3. **get_answer**: this is where the gemini api will go eventually. right now it just returns a fake string placeholder.
4. **main**: kicks off the tokio async runtime and prints out the final answer to the screen.

## todo list

- [ ] figure out how to actually parse the rust book markdown files
- [ ] hook up the real gemini api so it stops giving dummy answers
- [ ] figure out vector embeddings because simple text matching is not it long-term

If it crashes, don't mind it plz

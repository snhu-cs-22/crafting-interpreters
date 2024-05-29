# crafting-interpreters

My implementation of [Crafting Interpreters](https://craftinginterpreters.com) in Rust. Both the treewalk and bytecode implementation.

These two implementations are very much works in progress. It's hard to write perfectly safe Rust code when the original C codebase uses a lot of memory tricks to make it fast. Luckily, [others](https://rust-hosted-langs.github.io/book/introduction.html) [have](https://ceronman.com/2021/07/22/my-experience-crafting-an-interpreter-with-rust/) tried to implement this interpreter in Rust, so I can learn from their experience.

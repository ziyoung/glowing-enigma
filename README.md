# Regex memory issue

Memory leak caused by calling the static regular instance's `caputres_iter` method in an `async` function.

```rust
static REG: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(Object\.\w+)"#).unwrap());

async fn memory_leak_demo() -> Vec<u8> {
    //...
    let mut buf = Vec::from(text.as_bytes());
    for (i, caps) in REG.captures_iter(&text).enumerate() { // <--- here
        println!("process text: {}", i + 1);
        tokio::time::sleep(Duration::from_millis(50)).await;
        let m = caps.get(0).unwrap();
        // ...
    }
}
```

I dump the memory profile file during the program running. Use `jeprof` to parse memory profiles.

```bash
jeprof ./target/debug/memory-leak-example profile/regex-captures-iter-1.prof

# or
jeprof --svg ./target/debug/memory-leak-example profile/regex-captures-iter-1.prof
```

The analysis results with `jeprof` are as follows:

**regex-captures-iter-1.prof**:

Total: 3.3M

```
for (i, caps) in REG.captures_iter(&text).enumerate() { // <- 2.3M
```

**regex-captures-iter-2.prof**:

Total: 4.3M

```
for (i, caps) in REG.captures_iter(&text).enumerate() { // <- 3.8M
```
**regex-captures-iter-3.prof**:

Total: 5.3M

```
for (i, caps) in REG.captures_iter(&text).enumerate() { // <- 4.3M
```

**regex-captures-iter-4.prof**:

Total: 5.3M

```
for (i, caps) in REG.captures_iter(&text).enumerate() { // <- 4.8M
```

**regex-captures-iter-5.prof**:
Total: 7.9M

```
for (i, caps) in REG.captures_iter(&text).enumerate() { // <- 7.4M
```

# Reproduction

Clone the current code repository. Then start a container using the `rust:1.79` docker image and execute the following code.

```bash
# docker run -it --rm --volume $(pwd):/memory-leak-example --workdir /memory-leak-example rust:1.79 bash
export RUST_BACKTRACE=1
export _RJEM_MALLOC_CONF=prof:true,lg_prof_interval:28
cargo run
```

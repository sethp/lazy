# FAQ

Some Future Anticipated Questions for your perusal.

# Why build a new tool?

The need for lazy evaluation comes up surprisingly often in a shell context, both interactively and in batch mode.

In both cases, the existing solutions

## Interactive Laziness

Consider a long-running command which prints status information to stderr (as is common with build tools), and then generates a lot of output at the end, e.g.:

```shell
gen_data() {
    sleep 2; echo >&2 "progress 1/400"
    # ...
    sleep 2; echo >&2 "progress 399/400"
    sleep 2; echo >&2 "progress 400/400"

    </dev/random head -c 4000
}
```

Piping that command to `less` is convenient for browsing the output (and preserving scrollback), as in `gen_data | less`. But, then the shell will spawn both `gen_data` and `less` concurrently as soon as the pipeline starts. In this case, that means about as soon as `gen_data` starts, `less` will grabbing the TTY and switching to the alternate screen, obscuring the progress output.

Or, if saving the data to a file for later, a redirect or standalone `tee` will always clobber the file, even if an earlier command in the pipeline fails. Instead, `lazy` can help:

```shell
gen_data | lazy tee saved | lazy less
```

The intention of that sample invocation is to emit all the status output, then clobber `saved` and execute `less` only if `gen_data` succeeds and starts writing to its stdout.

## Batch Laziness

The most useful batch-mode example would be to guard against a pipeline misbehaving on an empty input and, say, unintentionally writing an empty string to [an important configuration key][cf-incident]. Or, to avoid running a command that might behave very differently with no input, like:

```shell
maybe_output_nothing | lazy xargs ls
# NB: probably you should use shell globbing instead anyway
```

Another example would be to reduce resource contention for fan-out-fan-in-fan-out workloads like building + linking: the compilation resource usage is extremely high, produces a very short list of object files, and then the linker again explodes that small input into a lot of work. Using a tool like [`sponge(1)`][sponge] to buffer the intermediate step, `lazy` can delay starting the expensive second stage until it has (relatively) exclusive access to the machine's resources.

[cf-incident]: https://blog.cloudflare.com/pipefail-how-a-missing-shell-option-slowed-cloudflare-down/

# Where are all the flags?

The only flag I considered adding was for help, but that would've induced a whole mini-DSL's worth of problems for the sake of embedding a `man` page.

It's not impossible that a future need will justify a new bespoke configuration language, but I'd like to defer that as long as possible. Once that day arrives, it'll be important to differentiate between arguments to `lazy` and arguments to its argument(s): commonly this is solved by introducing a separator sigil like `--`.

# Why Rust?

Mostly I was curious what it'd be like to interact with the universe of UNIX syscalls from Rust, and this was a well-bounded opportunity to experiment. The main downsides seemed to be packaging complexity, binary size, and mismatch between Rust's standard library and UNIX concepts. Were I to do it again, I'd probably try using Zig: not because I think it's better, but I'd have more to learn that way.

I also briefly considered the following:

- Go, but elected not to use it because invoking the difficulties its runtime introduces around fork/exec for a non-concurrent program that doesn't allocate (in userspace), which seemed a heavy cost to pay
- Python, because it's an excellent tool for stringing together a handful of syscalls, but chose not to because of distribution concerns,
- and C, but I didn't feel like I'd learn anything new (and I've already built enough ad-hoc build toolchains by hand)

Plus, haven't you heard? We're rewriting everything in Rust.

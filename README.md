# Lazy

Lazy evaluation for the shell.

Delays `exec` of its argument until it sees the first byte of input.

## Example

The example below demonstrates that `lazy` allows us to avoid either primary or secondary effects modifying the state of the system until after an earlier long-running command completes.

With `lazy`:

```shell
(sleep 10; date +%s) | tee ts | (lazy touch file; cat >/dev/null)
printf "The file was modified %s seconds ago\n" $(( $(<ts) - $(date -r file +%s) ))
# => The file was modified 0 seconds ago
```

Without `lazy`:

```shell
(sleep 10; date +%s) | tee ts | (touch file; cat >/dev/null)
printf "The file was modified %s seconds ago\n" $(( $(<ts) - $(date -r file +%s) ))
# => The file was modified 10 seconds ago
```

Note: the default behavior if the input pipe is closed without seeing any input is to exit with an error:

```shell
</dev/null lazy true ; echo $?
# => 3
```

## Platforms

* Linux
* TODO macOS
* TODO other 'nixes?

## Installing (TODO)

"just" build from source and copy the binary wherever you want it to be. The dream of the '90s!

## Caution: interactions with existing shell constructs

A word of warning: `lazy` can only defer evaluation of its own arguments, not the work the shell's already done before it is invoked. What that means is that neither of these work to prevent `file` from being clobbered:

```shell
# caution: eagerly evaluates the redirect!
maybe_output | lazy cat > file # always clobbers `file`
```

```shell
# caution: eagerly evaluates the pipeline!
maybe_output | lazy cat | tee file # always clobbers `file`
```

Instead, make sure the program that'll be opening the files is the one whose execution is being delayed:

```shell
# works!
maybe_output | lazy tee file
# also works!
maybe_output | lazy sh -c 'cat >file'
```

## Other Limitations

* `lazy` is not particularly useful with streaming- or incrementally-oriented tools: it shines when output is all-or-nothing, not when the output can degrade in quality partway through.
* `lazy` makes no attempt to handle the perennial mismatch between a computer's notion of byte streams and a human's notion of information that they care to look at: a single newline or a tab or a non-printable unicode code point are all at least one byte, though they will almost certainly not alone produce a meaningful output downstream.
* `lazy` seeks to replace itself (via `exec`) with its argument as rapidly as possible. This simplifies the implementation, which means we don't have to do things like signal forwarding, but it also means `lazy` can take no actions once it observes any input, as it very quickly ceases to exist.
* Like all UNIX tools, `lazy` suffers from a few common limitations:
    * "everything is a file" until it isn't; it's impossible to determine whether an arbitrary file handle will produce at least one byte of input without consuming that input, so special handling abounds.
    * byte streams and single integers are not _quite_ structured enough; there's an awful lot of shotgun parsers out there, and technically `lazy` adds to that pile.
    * argv-as-hci has some rough edges, not least of which is predicting the outcome through the many layers of variable expansion and shell interpolation and [quoting](https://mywiki.wooledge.org/Quotes).


## Goals

Some goals of this project that are often in tension with each other:

- Small: both conceptually and in size on disk.
- Portable: available everywhere.
- Reliable: prefer predictability.
- Helpful
    - The software ought to explain itself, and
    - The project ought to act as a repository of knowledge for solving similar problems, especially where `lazy` is helpful or is definitely the wrong tool.

Some non-goals include:

- Comprehensive: discussion about how to solve a particular problem is welcomed, implementing functionality that could exist elsewhere less so.
- Novelty: shells, pipes, processes, and files are all quite a lot already! They're all flawed tools that we aim to augment and explain (where necessary) rather than replace.
- Strictness: On balance, we'd prefer to practice repairing our tools and relationships over preventing a breach, especially a hypothetical one.

## Security

The focus on keeping the absolute size of the program small hopefully also serves the goal of avoiding introducing too many new attack vectors by way of `lazy`. It uses relatively few syscalls on its own, and once it `exec`s then any additional surface area it exposed disappears.

That said, it does use `exec`, which means if it can be tricked then it'll happily execute arbitrary code.  Potentially evenon the attacker's schedule if they can also control the input pipe. Further, because pipes don't offer a `peek` operation, the goal of getting to `exec` and overwriting itself means that it'll need to use more exotic syscalls, potentially even in unintended ways. That naturally exposes some additional risk.

If you do discover a security issue that affects `lazy`, please report it to @sethp (email address in profile), and I'll work with you to get it addressed as effectively as possible.

## See Also

- [sponge][sponge]
- [xargs][xargs]
- [execline][execline]

[sponge]: https://linux.die.net/man/1/sponge
[xargs]: https://linux.die.net/man/1/xargs
[execline]: https://skarnet.org/software/execline/

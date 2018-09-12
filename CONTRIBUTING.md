# Contributing to Iridium
Yay! We're so happy you want to contribute! Below are the various rules and guidelines for doing so.

## A Warning
*DO NOT USE IRIDIUM FOR _ANYTHING_ IN PRODUCTION*

It is primarily a teaching tool at https://blog.subnetzero.io, and while it may someday be suitable for production use, it isn't right now.

## Code of Conduct
This project and everyone participating in it is governed by the Iridium Code of Conduct. By participating, you are expected to uphold this code. Please report unacceptable behavior to iridium@subnetzero.io

## Join the Community
For the people that like forum-style with Discourse: https://discourse.subnetzero.io/
For the Gitter people: https://gitter.im/iridium-rs/general
For the Discord people: https://discord.gg/wjwQ22r

And yes, we'll shortly have IRC, Matrix, Slack, and every other freaking chat service that has ever existed, oh my god.

Ahem.

## How Can I Contribute?
Lots of ways! First, though, you should know that Iridium is a learning exercise and educational tool. All technical aspects of its development are blogged at https://blog.subnetzero.io/project/iridium-vm. Features on the roadmap are intended to have a tutorial written while they are coded; if you work on those, you get to write a blog post. =)

## Features
If there is a non-roadmap feature you would like to code, that's fine. It would be great if you would write up a tutorial or article on the experience, but it isn't required. There is currently no set process to get a feature in, so, uh, just ping me (fletcher@subnetzero.io) to talk about it. If this project attracts contributors, we'll have a less, ah, authoritarian style of goverenance.

### Bug Reporting
In all cases, please open a GitLab issue. Bear in mind that some bugs are left in intentionally so that they can be used in tutorials later. If this is the case, we will tag it with the release in which it should be fixed.

### Versioning
We use semver in a slightly odd way -- or at least people tell me we do, while glaring at me. Each `patch` number in semver maps to a specific tutorial and a GitLab tag. So we have `0.0.16`, which you can read about here: https://blog.subnetzero.io/project/iridium-vm/building-language-vm-part-16/ and is tied to this tag in GitLab: https://gitlab.com/subnetzero/iridium/tags/0.0.16.

Each tutorial increments the `patch` version by one. This started with `0.0.16`, so there are no tags prior to that. Once all of the tutorials are written, or a minimum subset, and it's considered "complete" with respect to the tutorials, the minor version will go to 1. At that point, the project will follow a more typical workflow.

## Styleguides
We are not (yet) dogmatic. Just follow these rules:

### Always
1. rustfmt
2. clippy
3. Doc comment all functions, structs, enums, everything. _Everything._
4. Write some tests

### Never
1. Nightly
2. Unsafe without discussion

### Principals
1. Security is more important than performance
2. Code should be as complex as it needs to be and as simple as it can be
3. Comments should explain why, not what
4. Focus on writing tests for the more critical aspects first
5. Warnings are errors

### Imperative and Functional
I personally tend to write Rust with a more imperative style; you rarely see me using `map` and such. A more functional style is fine to use. *Please* do not be that person that writes tons of clever hacks or insists on using every feature in the language.

## Development process
1. Make a branch off of the latest master. Avoid branches of branches of branches
2. Make your changes
3. Submit an MR for review


# Farewell!
This will get updated more as the project grows.

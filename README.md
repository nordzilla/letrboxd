# LetrBoxd

https://nordzilla.github.io/letrboxd/

A solver for the New York Times [Letter Boxed](https://www.nytimes.com/puzzles/letter-boxed) puzzle game that finds solutions without repeated letters.

<br>

## About

When I first played the game, I assumed, incorrectly, that the rules prevented using the same letter twice within a solution. A few days later I was trying to solve one of the puzzle inputs to no avail: I simply gave up without finding a solution. 

Since then I have read the rules, and I now understand that repeating letters is allowed. However, out of curiosity, I decided to write a program to find only solutions that do not use repeated letters. Once I had written the initial program as a CLI tool, I decided to optimize it further and build the [LetrBoxd](https://nordzilla.github.io/letrboxd/) website for it. 

Sadly, I don't recall exactly on which date I failed to find a unique-letter solution, but I do recall using my program to discover that there were very few unique-letter solutions to that day's puzzle input.

> [!NOTE]
> The puzzle input with the _most_ unique-letter solutions that I have been able to find is [AIO DGT ESU NPR](https://nordzilla.github.io/letrboxd/?top=AIO&right=DGT&bottom=ESU&left=NPR).

<br>

## Usage

### Search By Date

The site accepts a `date` query parameter to view puzzle inputs from past dates:
* e.g. https://nordzilla.github.io/letrboxd/?date=2023-10-31

### Search By Input

The site accepts `top`, `right`, `bottom`, and `left` query parameters for the three letters on each side of the input.

These parameters are updated fautomatically as you modify the puzzle-input UI on the site. 

* e.g. https://nordzilla.github.io/letrboxd/?top=AIO&right=DGT&bottom=ESU&left=NPR

### Specify Solver Count

The site accepts an optional `solvers` query parameter that will set the number of parallel solvers. By default, this is set to your system's [hardware concurrency](https://developer.mozilla.org/en-US/docs/Web/API/Navigator/hardwareConcurrency) value, but if you want to use the site with a specified number of threads, you can utilize this parameter.

* e.g. https://nordzilla.github.io/letrboxd/?solvers=2

<br>

## Areas for improvement

This is actually the first website that I've ever made, so if you are an experienced front-end developer and you happen to be reading the source code, please go easy on me. I had a lot of fun with this project, but it is far from perfect.

### Mobile Support

The site technically works on mobile devices, but the way I chose to write the user-input within an SVG is awkward and unusuable on phones. Honestly, that's okay with me. I don't foresee myself ever using this site on my phone.

If I do one day desperately need its functionality on my phone, I can provide the input through the query string parameters. Perhaps in the future I will take an interest in mobile development, then come back to update this site's UI to be more mobile friendly. 

### UI Tests

I had good intentions to write automated UI tests when I started building the site, but I eventually reached a point where I would rather just move on to other projects: I will just fix the site if it happens to break, though I hope things will be fairly stable (_fingers crossed_). I don't foresee myself working on this project regularly, or at all, in the future, except in the case that it does break.

### Native Multithreading and Shared WASM

The site itself is multithreaded via web workers rather than directly in the WASM. Each worker instantiates its own module with its own copy of the WASM binary. Even with regard to the puzzle-solver algorithms, there are quite a lot of duplicated data that get serialized and passed around. 

To be fair, this really isn't much of a bottleneck for this particular site's performance, though I think it's still worth mentioning. The site certainly uses more memory than it needs to.

There are two primary reasons that I wrote things the way that they are:

1) At the time of writing, native thread support within WASM appears to require a nightly Rust toolchain, and I wanted to write everything using stable.
2) Using shared buffers among web workers requires requires using [cross origin isolation](https://developer.mozilla.org/en-US/docs/Web/API/Window/crossOriginIsolated). I did experiment with this a bit during the development process, but the current state of GitHub pages does not support these headers, so I decided not utilize this functionality in this project. 

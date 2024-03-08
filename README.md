# Ore

**Ore is a cryptocurrency you can mine from anywhere, at home or on your phone.** It uses a novel proof-of-work algorithm to guarantee no miner can ever be starved out from earning rewards. 


## How it works

The primary innovation of Ore is to offer non-exclusive mining rewards. This means if one miner wins rewards, it doesn't prevent another miner from also winning. In traditional proof-of-work systems such as Bitcoin, mining rewards are exclusive. That is, only one Bitcoin miner can win every ~10 minutes, and that miner takes home all the tokens for the round. This has had the longterm effect of starving out casual miners who are simply unable to keep up in the arms race against much larger and well-resourced professional mining firms.

The primary reason Bitcoin is designed this way is that its proof-of-work system serves two roles. It's responsible not only for distributing tokens, but also for coordinating network consensus. This makes sense as the tokens are intended to be a reward for those who dedicate resources to securing the Bitcoin network. However due starvation problems outlined above, it has had the unintended consequence of excluding major portions of the global population (>99%) from ever being able to mine. This ultimately limits the number of people who can reasonably acquire the token, and thus contributes to further consolidating the supply. Ore takes a different approach.

Ore builds upon the consensus layer provided by Solana and uses it to reimagine proof-of-work purely as a token distribution mechanism. Rather than setting up every miner in a winner-take-all competition against one another, Ore gives each miner their own individual computational challenge. As long as a miner provides a valid solution to their personal challenge, the protocol guarantees they will earn a piece of the supply. Since no miner can be censored from the network and valid solutions are non-exclusive, starvation is avoided.


## Supply

Ore provides strong guarantees and protection against runaway supply inflation. The supply growth rate is strictly bounded to a range of 0 ≤ R ≤ 2 ORE / min. In other words, linear. The reward rate – amount paid out to miners per valid hash – is dynamically adjusted every 60 seconds to maintain a target average supply growth rate of 1 ORE / min. This effectively means if the global hashpower dedicated to Ore mining increases, the reward rate paid out per hash will decrease, and vice versa. 

A linear supply growth was chosen for its simplicity and straightforward predictability. Ore aims to strike a balance between the unpredictable runaway inflation of fiat currencies on one hand and the feudal deflationary supply schedules of alternative cryptocurrencies on the other. Ore holders are simultaneously incentivized to loan and spend while also being protected against longterm exponential inflation.


## Program
- [`Entrypoint`](src/lib.rs) – The program entrypoint.
- [`Consts`](src/consts.rs) – Program constants.
- [`Errors`](src/error.rs) – Custom program errors.
- [`Instruction`](src/instruction.rs) – Declared instructions and arguments.
- [`Loaders`](src/loaders.rs) – Validation logic for loading Solana accounts.

## Instructions
- [`Initialize`](src/processor/initialize.rs) – Initializes the Ore program, creating the bus, mint, and treasury accounts.
- [`Reset`](src/processor/reset.rs) – Prepares the program for a new epoch, updating the reward rate, resetting bus counters, and topping up the treasury.
- [`Register`](src/processor/register.rs) – Creates a new proof account for a prospective miner.
- [`Mine`](src/processor/mine.rs) – Verifies a hash provided by a miner and issues claimable rewards.
- [`Claim`](src/processor/claim.rs) – Distributes claimable rewards as tokens from the treasury to a miner.
- [`UpdateAdmin`](src/processor/update_admin.rs) – Updates the admin authority.
- [`UpdateDifficulty`](src/processor/update_difficulty.rs) - Updates the hashing difficulty.

## State
 - [`Bus`](src/state/bus.rs) - An account (8 total) which tracks and limits the amount mined rewards each epoch.
 - [`Proof`](src/state/proof.rs) - An account (1 per miner) which tracks a miner's current hash, claimable rewards, and lifetime stats.
 - [`Treasury`](src/state/treasury.rs) – A singleton account which manages program-wide variables and is the mint authority for the Ore token.


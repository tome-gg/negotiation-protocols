# Tome.gg Negotiation Protocols - Alignment Negotiation Smart Contract

An alignment negotiation tool (designed as a smart contract) for software engineering mentors and apprentices. 

# Context

There are three (3) major hurdles in accomplishing the revolutionary education that Tome.gg is pursuing:

1. Accessibility to education - financial cost, time searching/skimming/reading/getting lost, etc.

2. Contextualization - Languages and technologies evolve, and it takes time to transfer learning from one context to another (e.g. age groups: 40y/o teaching 25 y/os). In many parts of the world, personalizing education is expensive, if not unavailable.

3. Negotiation - Establishing and negotiating a mentoring engagement is a pain point for both mentors and apprentices.

With the above definition, this project is building around the third challenge of alignment negotiation between mentors and apprentices.

Read more at [Tome.gg](https://tome.gg)!

## Build

```bash
anchor build
```

## Test

### Testing on local

1. Define two key pairs for the apprentice and mentor. You can use [Phantom](https://phantom.app/) to generate new wallets.

Alternatively, just use the dummy wallet specified in `test/`.

2. Ensure your solana test validator is running locally. Do:

```bash
solana-test-validator
```

3. Make sure both wallets have funds by airdropping some Solana:

```bash
solana airdrop 15 <public key of apprentice>
solana airdrop 15 <public key of mentor>
```

3. Run the defined tests.

Because your local validator is already running from stpe 2, you can build, deploy, and run the typescript tests only with the following command:

```bash
anchor test --skip-local-validator 
```

You can also choose to run the typescript tests only:

```bash
anchor test --skip-build --skip-deploy --skip-local-validator 
```


## Roadmap

### Tasks ahead

1. Write a GitHub action to run the tests on the dev network.
2. Integrate this with the frontend app.

## Contributing

I haven't set up contributing guidelines yet, but feel free to connect with me on Twitter @darrensapalo or [join my community on Discord](http://bit.ly/3yCdUiE).


## License

Apache License 2.0. See [LICENSE.md](LICENSE.md).

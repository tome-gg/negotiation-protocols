# Tome.gg - Alignment Negotiation Smart Contract

This is `Alignment Negotiation`, which is a smart contract designed for software engineering mentors and apprentices to negotiate and align the parameters of their mentoring engagement.

This tool enables mentors and apprentices:

- to have a public record of their misalignment
- to have a public record of their negotiation proposals for each other
- to have a signed contract between both parties on their agreement
- to have a third-party smart contract protocol (i.e. Tome Negotiation Protocols) that fulfill/perform the consequences of the signed contract

Read about a little more [context](docs/context.md), or read more at [Tome.gg](https://tome.gg)!

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

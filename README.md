# Alignment Negotiation

## Testing on your local

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
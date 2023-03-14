import * as anchor from "@coral-xyz/anchor";
import { Program, web3 } from "@coral-xyz/anchor";
import { assert } from "chai";
import { AlignmentNegotiation } from "../target/types/alignment_negotiation";

import apprenticeSecret from "./apprentice.json";
import mentorSecret from "./mentor.json";

// No imports needed: web3, anchor, pg and more are globally available

async function requestAirdrop(programProvider: anchor.AnchorProvider, walletAddress: web3.PublicKey, airdropAmount: number) {
  console.log(`🌧️ Requesting airdrop for ${walletAddress}`);
  // 1 - Request Airdrop

  const SOLANA_CONNECTION = programProvider.connection; // new web3.Connection(web3.clusterApiUrl("devnet"));
  const signature = await SOLANA_CONNECTION.requestAirdrop(
    walletAddress,
    airdropAmount
  );
  // 2 - Fetch the latest blockhash
  const { blockhash, lastValidBlockHeight } =
    await SOLANA_CONNECTION.getLatestBlockhash();
  // 3 - Confirm transaction success
  await SOLANA_CONNECTION.confirmTransaction(
    {
      blockhash,
      lastValidBlockHeight,
      signature,
    },
    "finalized"
  );
  // 4 - Log results
  console.log(
    `✅ txn complete: https://explorer.solana.com/tx/${signature}?cluster=devnet`
  );
}

function createEvent(
  action: "discuss" | "propose" | "review" | "accept",
  negotiationElement: "term" | "protocol" | "parameters" | "stake"
): number {
  let exponent = 0, value = 0;
  switch (action) {
    case "discuss":
      exponent = 16;
      break;
    case "propose":
      exponent = 8;
      break;
    case "review":
      exponent = 4;
      break;
    case "accept":
      exponent = 0;
      break;
  }

  switch (negotiationElement) {
    case "term":
      value = 8;
      break;
    case "protocol":
      value = 4;
      break;
    case "parameters":
      value = 2;
      break;
    case "stake":
      value = 1;
      break;
  }

  let encoding = value << exponent;

  return encoding;
}

function testState(
  value: any,
  label: "term" | "protocol" | "parameters" | "stake",
  expected: "discussion" | "accepted" | "proposed" | "reviewed"
) {
  let message = `${label} state incorrect; got null, want ${expected}`;
  assert.equal(
    value[expected] !== null && value[expected] !== undefined,
    true,
    message
  );
}

describe("Test", () => {

  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.AlignmentNegotiation as Program<AlignmentNegotiation>;
  const programProvider = program.provider as anchor.AnchorProvider;

  it("Apprentice can setup a new alignment negotiation", async () => {
    console.log(`[program     : ${program.programId}]`);

    // Generate keypairs from dummy accounts on dev net.
    // These secrets are FINE; relax. This is just development environment. 🤦‍♂️
    console.log(mentorSecret);
    const mentorKp = web3.Keypair.fromSecretKey(new Uint8Array(mentorSecret), {skipValidation: true});
    const apprenticeKp = web3.Keypair.fromSecretKey(new Uint8Array(apprenticeSecret), {skipValidation: true});
    // const mentorKp = new web3.Keypair();
    // const apprenticeKp = new web3.Keypair();

    const alignmentNegotiationKp = new web3.Keypair();

    console.log("✅ Generated keypairs!");
    console.log(`[apprentice  : ${apprenticeKp.publicKey}]`);
    console.log(`[mentor      : ${mentorKp.publicKey}]`);

    console.log(`[negotiation : ${alignmentNegotiationKp.publicKey}]`);

    // console.log(`Requesting airdrops`);
    // await requestAirdrop(apprenticeKp.publicKey, 1);
    // await requestAirdrop(mentorKp.publicKey, 1);
    // console.log(`  - Done! Setting up negotiation...`);

    // await requestAirdrop(apprenticeKp.publicKey, 5);

    // Set up negotiation
    const setupNegotiationTxHash = await program.methods
      .setupNegotation(mentorKp.publicKey)
      .accounts({
        negotiation: alignmentNegotiationKp.publicKey,
        apprentice: apprenticeKp.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([alignmentNegotiationKp, apprenticeKp])
      .rpc();
    console.log(
      `📑 Txn 1 (Setup) : https://explorer.solana.com/tx/${setupNegotiationTxHash}?cluster=devnet`
    );
    console.log(
      `📑 Negotiation   : https://explorer.solana.com/account/${alignmentNegotiationKp.publicKey}?cluster=devnet`
    );

    // Confirm transaction
    // await pg.connection.confirmTransaction(setupNegotiationTxHash);

    console.log("Transaction confirmed");

    // Fetch the created account
    let alignmentNegotiation =
      await program.account.alignmentNegotiation.fetch(
        alignmentNegotiationKp.publicKey
      );

    console.log("On-chain data; current turn is:", alignmentNegotiation.turn);

    // Check whether the data on-chain is equal to local 'data'
    assert(alignmentNegotiation.turn == 1);
  }).timeout(60000);

  it("Parties can do back and forth proposals on an alignment negotiation", async () => {
    console.log(`[program     : ${program.programId}]`);

    // Generate keypair for the new accounts
    const mentorKp = web3.Keypair.fromSecretKey(new Uint8Array(mentorSecret), {skipValidation: true});
    const apprenticeKp = web3.Keypair.fromSecretKey(new Uint8Array(apprenticeSecret), {skipValidation: true});
    const alignmentNegotiationKp = new web3.Keypair();

    console.log("✅ Generated keypairs!");
    console.log(`[apprentice  : ${apprenticeKp.publicKey}]`);
    console.log(`[mentor      : ${mentorKp.publicKey}]`);

    console.log(`[negotiation : ${alignmentNegotiationKp.publicKey}]`);

    // Turn 1 - Set up negotiation
    const setupNegotiationTxHash = await program.methods
      .setupNegotation(mentorKp.publicKey)
      .accounts({
        negotiation: alignmentNegotiationKp.publicKey,
        apprentice: apprenticeKp.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([alignmentNegotiationKp, apprenticeKp])
      .rpc();
    console.log(
      `📑 Txn 1 (Setup) : https://explorer.solana.com/tx/${setupNegotiationTxHash}?cluster=devnet`
    );
    console.log(
      `📑 Negotiation   : https://explorer.solana.com/account/${alignmentNegotiationKp.publicKey}?cluster=devnet`
    );

    // Confirm transaction
    // await pg.connection.confirmTransaction(setupNegotiationTxHash);

    // Fetch the created account
    let alignmentNegotiation =
      await program.account.alignmentNegotiation.fetch(
        alignmentNegotiationKp.publicKey
      );

    // Check whether the data on-chain is equal to local 'data'
    assert.equal(alignmentNegotiation.turn, 1, "turn should be equal to 1");

    // Turn 2 - Send first proposal

    const TNP_1_0 = "BB9Ewao8p1qoEFWNPZjAHnXeKwvmb7q2G2vfFi8MEhLe";
    const FAKE_TERM_SESSION_FREQUENCY =
      "BB9Ewao8p1qoEFWNPZjAHnXeKwvmb7q2G2vfFi8MEhLe";
    let params = {};

    const sendFirstProposalTxHash = await program.methods
      .propose({
        // TNP 1.0 protocol
        protocol: new web3.PublicKey(TNP_1_0),
        term: null,
        parameters: null,
        stakes: null,
        events: createEvent("propose", "protocol"),
        altProtocol: null,
        altTerm: null,
      })
      .accounts({
        negotiation: alignmentNegotiationKp.publicKey,
        player: apprenticeKp.publicKey,
      })
      .signers([apprenticeKp])
      .rpc();

    console.log(
      `📑 Txn 2 (Prpsl) : https://explorer.solana.com/tx/${sendFirstProposalTxHash}?cluster=devnet`
    );

    // Confirm first proposal
    // await pg.connection.confirmTransaction(sendFirstProposalTxHash);

    alignmentNegotiation = await program.account.alignmentNegotiation.fetch(
      alignmentNegotiationKp.publicKey
    );

    assert.equal(alignmentNegotiation.turn, 2, "turn should be equal to 2");

    assert.equal(
      alignmentNegotiation.protocol.toString(),
      TNP_1_0,
      `protocol should be TNP 1.0 after apprentice proposes to the mentor; got ${alignmentNegotiation.protocol}, want ${TNP_1_0}`
    );

    testState(alignmentNegotiation.protocolState, "protocol", "proposed");

    // Turn 3 - Accept proposal; fill in the rest

    const sendSecondProposalTxHash = await program.methods
      .propose({
        protocol: new web3.PublicKey(TNP_1_0),
        term: new web3.PublicKey(FAKE_TERM_SESSION_FREQUENCY),
        parameters: [
          255,
          128,
          64,
          32,
          16,
          8,
          4,
          2,
          1,
          0,
          0,
          128, // test data
        ],
        stakes: new anchor.BN(10),
        events:
          createEvent("accept", "protocol") | createEvent("propose", "stake"),
        altProtocol: null,
        altTerm: null,
      })
      .accounts({
        negotiation: alignmentNegotiationKp.publicKey,
        player: mentorKp.publicKey,
      })
      .signers([mentorKp])
      .rpc();

    console.log(
      `📑 Txn 3 (Prpsl) : https://explorer.solana.com/tx/${sendSecondProposalTxHash}?cluster=devnet`
    );

    // Confirm Second proposal
    // await pg.connection.confirmTransaction(sendSecondProposalTxHash);

    alignmentNegotiation = await program.account.alignmentNegotiation.fetch(
      alignmentNegotiationKp.publicKey
    );

    assert.equal(alignmentNegotiation.turn, 3, "turn should be equal to 3");

    assert.equal(
      alignmentNegotiation.stakes.eq(new anchor.BN(10)),
      true,
      `Stake should be equal to 10 after mentor proposes stake; got ${alignmentNegotiation.stakes}, want 10`
    );

    testState(alignmentNegotiation.protocolState, "protocol", "accepted");
    testState(alignmentNegotiation.stakesState, "stake", "proposed");
    testState(alignmentNegotiation.termState, "term", "discussion");

    // Turn 4 - Accept all as apprentice

    const sendThirdProposalTxHash = await program.methods
      .propose({
        protocol: new web3.PublicKey(TNP_1_0),
        term: new web3.PublicKey(FAKE_TERM_SESSION_FREQUENCY),
        parameters: [
          255,
          128,
          64,
          32,
          16,
          8,
          4,
          2,
          1,
          0,
          0,
          128, // test data
        ],
        stakes: new anchor.BN(10),
        events:
          createEvent("accept", "protocol") |
          createEvent("accept", "stake") |
          createEvent("accept", "term") |
          createEvent("accept", "parameters"),
        altProtocol: null,
        altTerm: null,
      })
      .accounts({
        negotiation: alignmentNegotiationKp.publicKey,
        player: apprenticeKp.publicKey,
      })
      .signers([apprenticeKp])
      .rpc();

    console.log(
      `📑 Txn 4 (Prpsl) : https://explorer.solana.com/tx/${sendThirdProposalTxHash}?cluster=devnet`
    );

    // Confirm Third proposal
    // await pg.connection.confirmTransaction(sendThirdProposalTxHash);

    alignmentNegotiation = await program.account.alignmentNegotiation.fetch(
      alignmentNegotiationKp.publicKey
    );

    assert.equal(alignmentNegotiation.turn, 4, "turn should be equal to 4");

    assert.equal(
      alignmentNegotiation.isComplete,
      true,
      `Alignment negotiation final state incorrect; got ${alignmentNegotiation.isComplete}, want true`
    );

    testState(alignmentNegotiation.termState, "term", "accepted");
    testState(alignmentNegotiation.protocolState, "protocol", "accepted");
    testState(alignmentNegotiation.protocolState, "parameters", "accepted");
    testState(alignmentNegotiation.stakesState, "stake", "accepted");
  }).timeout(120000);
});

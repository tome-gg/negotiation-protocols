import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SrcAlignmentNegotiation } from "../target/types/src_alignment_negotiation";

describe("src-alignment-negotiation", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.SrcAlignmentNegotiation as Program<SrcAlignmentNegotiation>;

  it("Is initialized!", async () => {
    // Add your test here.
    const mentor =  anchor.web3.Keypair.generate();
    const tx = await program.methods.setupNegotation(mentor.publicKey).rpc();
    console.log("Your transaction signature", tx);
  });
});

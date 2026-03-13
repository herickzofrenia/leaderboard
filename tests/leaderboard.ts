import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Leaderboard } from "../target/types/leaderboard";
import { PublicKey } from "@solana/web3.js";
import { assert } from "chai";

describe("leaderboard", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.Leaderboard as Program<Leaderboard>;
  const provider = anchor.AnchorProvider.env();

  const name = "Top Players";
  const maxEntries = 10;
  let leaderboardPda: PublicKey;

  before(async () => {
    [leaderboardPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("leaderboard"),
        provider.wallet.publicKey.toBuffer(),
        Buffer.from(name),
      ],
      program.programId
    );
  });

  it("Initializes the leaderboard", async () => {
    await program.methods
      .initialize(name, maxEntries)
      .accounts({
        leaderboard: leaderboardPda,
        admin: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const lb = await program.account.leaderboard.fetch(leaderboardPda);
    assert.equal(lb.name, name);
    assert.equal(lb.maxEntries, maxEntries);
    assert.equal(lb.entries.length, 0);
    console.log("✅ Leaderboard initialized:", name);
  });

  it("Submits a score", async () => {
    const player = anchor.web3.Keypair.generate();

    await program.methods
      .submitScore("Alice", new anchor.BN(1000))
      .accounts({
        leaderboard: leaderboardPda,
        admin: provider.wallet.publicKey,
        player: player.publicKey,
      })
      .rpc();

    const lb = await program.account.leaderboard.fetch(leaderboardPda);
    assert.equal(lb.entries.length, 1);
    assert.equal(lb.entries[0].playerName, "Alice");
    assert.equal(lb.entries[0].score.toNumber(), 1000);
    console.log("✅ Score submitted: Alice = 1000");
  });

  it("Updates score if higher", async () => {
    const player = anchor.web3.Keypair.generate();

    await program.methods
      .submitScore("Bob", new anchor.BN(500))
      .accounts({
        leaderboard: leaderboardPda,
        admin: provider.wallet.publicKey,
        player: player.publicKey,
      })
      .rpc();

    await program.methods
      .submitScore("Bob", new anchor.BN(2000))
      .accounts({
        leaderboard: leaderboardPda,
        admin: provider.wallet.publicKey,
        player: player.publicKey,
      })
      .rpc();

    const lb = await program.account.leaderboard.fetch(leaderboardPda);
    const bob = lb.entries.find((e) => e.playerName === "Bob");
    assert.equal(bob.score.toNumber(), 2000);
    console.log("✅ Score updated: Bob = 2000");
  });

  it("Keeps leaderboard sorted", async () => {
    const lb = await program.account.leaderboard.fetch(leaderboardPda);
    for (let i = 0; i < lb.entries.length - 1; i++) {
      assert(lb.entries[i].score.gte(lb.entries[i + 1].score));
    }
    console.log("✅ Leaderboard is sorted correctly");
  });

  it("Removes a player", async () => {
    const lb = await program.account.leaderboard.fetch(leaderboardPda);
    const playerToRemove = lb.entries[0].player;

    await program.methods
      .removePlayer(playerToRemove)
      .accounts({
        leaderboard: leaderboardPda,
        admin: provider.wallet.publicKey,
      })
      .rpc();

    const lbAfter = await program.account.leaderboard.fetch(leaderboardPda);
    assert.equal(lbAfter.entries.length, lb.entries.length - 1);
    console.log("✅ Player removed successfully");
  });

  it("Resets the leaderboard", async () => {
    await program.methods
      .reset()
      .accounts({
        leaderboard: leaderboardPda,
        admin: provider.wallet.publicKey,
      })
      .rpc();

    const lb = await program.account.leaderboard.fetch(leaderboardPda);
    assert.equal(lb.entries.length, 0);
    console.log("✅ Leaderboard reset successfully");
  });
});
